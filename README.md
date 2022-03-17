# trait-enumizer

![img](doc_header.png)

Generate enum from a trait, with converters between them.

# Features

* Generation of enum based on specified trait (or inherent impl) where each variant corresponds to a method. Arguments correpond to fields of the variants; return values, correspond to a sender of some channel.
* Generation call functions function with a appropriate `match` generated inside. This allows the enum to be "applied" to an object implementing the trait.
* Generation of a proxy that allows obtaining a sequence of enum values using method calls from original trait (if possible) or API similar to original. Proxy also helps dealing with "channelizing" return values.
* Handling return values.
* Handling async (inherent impl only).

The library can be used as a synchronisation mechanism or as a building block to build actors or remote procedure calls.

Main piece of library is `enumizer` attribute macro. It should be attached to trait or inherent impl. It copies input trait or impl to proc macro output (sans pseudo-derive-helpers attributes), then also generates the following items:

* Enum where each variant represent a method
* Zero or more "call functions"
* Zero or more "proxy structs"

# Parameters

`#[trait_enumizer::enumizer(...)]` accepts following parameters:

* `name=<ident>` - Set the enum name. Required.
* `pub`, `pub_crate` - Mark all generated items as `pub` or `pub(crate)` respectively
* `returnval=<macro_class_name>` - Enable more complex mode where return values are handled. Affects API of other generated items (call_fns and proxies) as well. Input macros is used as a makeshift GAT trait with specialisation. See a dedicated README section for more info.
* `enum_attr` - Inject custom attribute (e.g. `enum_attr[derive(serde_derive::Serialize)]`)  into enum declaration. Can be repeated. You need to use square brackets for this.
* `inherent_impl` - Base enum on an inherent impl instead of a trait.
* `call_fn()` - See below.
* `proxy()` - See below.

Example of attributes syntax:

```rust,ignore
#[trait_enumizer::enumizer(pub_crate, name=NewEnumName, call_fn(name=call_me,ref_mut,allow_panic), proxy(name=NewEnumNameProxy, Fn,unwrapping_impl))]
```

# Call functions (`call_fn`s)

Call functions are generated when you use `call_fn()` parameter. They use the following subparameters:

* `name=<ident>` - name of the inherent method. Required.
* `ref` or `ref_mut` (alias `mut_ref`) or `once` (alias `move`) - Accept the object by reference, by mutable reference or by value. Required.
* `allow_panic` - Allow generation of the function with `panic!()` calls inside.
* `async` - Generate `async fn`. Use `send_async` pseudomethod from `returnval` macro-class instead of `send`.
* `extra_arg_type(<type>)` - Add additional argument to the `try_call` function. That argument will appear on all `macro_class_name!(send(...))` callbacks.

Those functions are used to "convert" enum value into a method call. Call functions are generated as inherent impl functions of the generated enum. First argument is `self`. Second argument is the value of (or reference to) something implementing the trait you specified (skipping the trait in `inherent_impl` mode). Third argument is required if you specify `extra_arg_type()`. It is passed to returnval's `send` (or `send_async`) pseudomethod for customized handling of return values.

Example:

```rust,ignore
#[enumizer(name=QqqEnum,pub,call_fn(name=the_call,ref_mut))]
trait Qqq {
    ...
}
```

generates

```rust,ignore
enum QqqEnum { ... }
impl QqqEnum {
    pub fn the_call<I: Qqq>(self, o: &mut I);
}
```

# Proxies

Proxies are generated when you use `proxy()` parameter. They use the following subparameters:

* `Fn`, `FnMut`, `FnOnce` - Set type of closure that the proxy will carry. Required.
* `name=<ident>` - name of the generated struct. Required.
* `resultified_trait=<ident>` - Also generate "resultified" trait instead of implementing `try_*` functions inherently.
* `infallible_impl` - Make the proxy also implement the original trait, provided your sink function does not err.
* `unwrapping_impl` - Make the proxy also implement the original trait, using `unwrap` where needed.
* `unwrapping_and_panicking_impl` - Force proxy to implement the original trait, using `panic!()` calls where complication would fail because of ownership requirements.
* `extra_field_type(...)` - Add additional second field to proxy struct. That field will be used as additional argument to `macro_class_name!(create(...))` and `macro_class_name!(recv(...))` callbacks.
* `async` - Expect user-specified closure to return `Future<Output=Result>` instead of just `Result` and use `.await`s inside where appropriate.

A proxy is a generic tuple struct with a public field. That field should implement `Fn`, `FnMut` or `FnOnce`. Second field (also public) is created if you specify `extra_field_type()`. There are two generic parameters: error type (you choose it) and closure type.
Proxies allow "converting" method calls to enum values (which get delivered to your closure). By default all input methods are renamed, having "try_" prepended. Typically they return `Result<(), YourErrorType>`, but in `returnval` mode some of them may return `Result<Result<T, SendError>, YourErrorType>`. There is async mode, which upgrades your function to return `Future` and makes all the `try_*` methods `async`. You can ask Enumizer to also generate "resultified" trait which proxy then implements (unless `async`, of course). `async` also affects `returnval` macro usage.

You can also ask Enumizer to make proxy implement the original trait (also unless `async`). There are two strategies for it: infallible (if return values are not used and your `Fn` opts out of error handling by using `std::convert::Infallible`) and unwrapping.

You can make `async` proxy for non-async original methods and vice versa.

Example (simplified):

```rust,ignore
#[enumizer(name=QqqEnum,proxy(FnMut,name=QqqProxy))]
trait Qqq {
    fn foo(&self,x : i32);
}
```

generates

```text
enum QqqEnum { Foo{x:i32} }
struct QqqProxy<E,F>(pub F)
    where F: FnMut(QqqEnum) -> Result<(), E>;
impl<E,F> QqqProxy where ... {
    fn try_foo(&mut self, x: i32) -> Result<(), E>{
        (self.0)(QqqEnum::Foo{x})
    }
}
```

# Pseudo-derive-helpers

Pseudo-derive-helpers are attributes that are handled by this library. You are supposed to used them inside your input trait or impl before method signature or before argument inside signature. Other (unknown) attributes are passed though unmodified.

* `#[enumizer_enum_attr[...]]` - Forward specified attribute to generated enum. Example: `#[enumizer_enum_attr[serde(rename="qqq")]]`. Can be attached to functions (which become enum variants) or to function arguments (which become enum variant fields).
* `#[enumizer_return_attr[...]]` - in `returnval` mode, attach custom attribute to the `ret` field of the enum.
* `#[enumizer_to_owned]` - For reference argument type, use owned value instead of trying to put reference to enum (which may not work, unless `'static`).

# Returnval pseudotrait

If you want Enumizer to handle return values, you need a channel of some sort. Enumizer is flexible in channel choice. There are built in "classes" for some popular channel types, you may also need to implement a channel class yourself.

You specify channel class as value for `returnval` parameter, e.g. `returnval=trait_enumizer::flume_class`. Early in Enumizer design channel classes were traits using GAT, but now they are special `macro_rules`-based macros.

Here is API of a channel class:

```rust,ignore
macro_rules! my_channelclass {
    (Sender<$T:ty>) => { /* type of the `ret` field in enum variant */ };
    (SendError) => { /* Error type when failed to send to a channel. Must not depend on T */ };
    (RecvError) => { /* Error type when failed to receive from a channel */ };
    (create::<$T:ty>()) => {
         /* Expression returning (tx, rx) channel pair. `tx` must be of type `Sender`. */
         /* Used by proxies */
    };
    (send::<$T:ty>($channel:expr, $msg:expr /*, $extraarg:expr */)) => { 
         /* Expression used to send to the channel (for `call_fn`). You may need to map error type here */
    };
    (recv::<$T:ty>($channel:expr /*, $extrafield:expr */)) => { 
        /* Expression use to recv value from the channel (for proxy) */
     };
    (send_async::<$T:ty>($channel:expr, $msg:expr)) => { 
        /* Expression to send to channel from async `call_fn`s. Should include `.await` and error mapping */
    };
    (recv_async::<$T:ty>($channel:expr)) => { 
        /* Expression to receive from cahnnel in async proxies. Should include `.await`. */
     };
}
```

You are recommended to base your implementation on one of the built-in channel class (e.g. `flume_class`) or to use RPC sample as a template for trickier channel class.

Although returnval mechanism use "channel" terminology, `Sender`s are not required to be actual channels. They may be some internal IDs, with the real channel being supplied as an additional argument.

When Enumizer encountres a method with return value, corresponding enum variant gains additional field named `ret` (a hard coded identifier). Type of this field is controlled by the channel class and may depend on the type of the return value. All interactions with this additional field go though channel class's pseudomethods.


# Tests (also serve as documentation)

To understand how the crate functions, you can view some test files. For most samples there is corresponding "manual" sample, showing expanded version (sometimes slightly simplified) of the same test.

Note that those links may be broken on Docs.rs, use README on Github instead.

* [`simple_derive.rs`](crates/trait-enumizer/tests/simple_derive.rs) - Simple demo.
* [`simple_manual.rs`](crates/trait-enumizer/tests/simple_manual.rs) - Expanded implementation of the demo above.
* [`mutable_derive.rs`](crates/trait-enumizer/tests/mutable_derive.rs), [`mutable_manual.rs`](crates/trait-enumizer/tests/mutable_manual.rs), [`move_derive.rs`](crates/trait-enumizer/tests/move_derive.rs), [`move_manual.rs`](crates/trait-enumizer/tests/move_manual.rs) - the same, but for mut and once cases.
* [`mixed.rs`](crates/trait-enumizer/tests/mixed.rs) - Showcases some other features, also uses threading to demonstrates how to interact with channels (using [flume](https://crates.io/crates/flume) as example). Also showcases injecting custom derive into the generated enum.
* [`returnval_derive.rs`](crates/trait-enumizer/tests/returnval_derive.rs), [`returnval_manual_generic.rs`](crates/trait-enumizer/tests/returnval_manual_generic.rs) - Tricker mode for handling return values.
* [`returnval_manual_flume.rs`](crates/trait-enumizer/tests/returnval_manual_flume.rs) - Original version without the channel abstraction, hard coded for [flume](https://crates.io/crates/flume) instead.
* [`rpc.rs`](crates/trait-enumizer/tests/rpc.rs) - Demonstrates advanced usage of custom `returnval=` classes to make remote prodecure call using serde_json and flume. Two primary Flume channels simulate a socket, interim Flume channels route return values back to callers.
* [`toowned_manual`](crates/trait-enumizer/tests/toowned_manual.rs), [`toowned_derive`](crates/trait-enumizer/tests/toowned_derive.rs) - Expanded (manual) and automaitcally derived demonstration of `#[enumizer_to_owned]` feature.
* [`inherent_derive`](crates/trait-enumizer/tests/inherent_derive.rs) - demonstrates `inherent_impl` mode.
* [`async_derive.rs`](crates/trait-enumizer/tests/async_derive.rs), [`async_manual.rs`](crates/trait-enumizer/tests/async_manual.rs), [`async_returnval_derive.rs`](crates/trait-enumizer/tests/async_derive.rs), [`async_returnval_manual.rs`](crates/trait-enumizer/tests/async_manual.rs), [`async_rpc.rs`](crates/trait-enumizer/tests/async_derive.rs), [`async_rpc.rs`](crates/trait-enumizer/tests/async_manual.rs) - async versions of some of the tests above.
* [`channelclasses_showcase.rs`](crates/trait-enumizer/tests/channelclasses_showcase.rs) - various built-in channel classes.


# See also

* [spaad](https://crates.io/crates/spaad)
* [enum_dispatch](https://crates.io/crates/enum_dispatch)
