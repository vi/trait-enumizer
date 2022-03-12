# trait-enumizer

![img](doc_header.png)

Generate enum from a trait, with converters between them.

Features:

* Generation of enum based on specified trait where each variant corresponds to a method. Arguments correpond to fields of the variants; return values, correspond to a sender of some channel.
* Generation of a `call`, `call_mut` or `call_once` function with a appropriate `match` generated inside. This allows the enum to be "applied" to an object implementing the trait.
* Generation of a proxy that allows obtaining a sequence of enum values by using modified or original trait. If return values are handled, this proxy also creates temporary channel to deliver return value on each call.

By default, return values in methods are not handled. You need to enable `returnval` Cargo feature, which also needs `#![feature(generic_associated_types)]`. This also changes API of the generated items.

Main piece of library is `enumizer` attribute macro. It should be attached to trait and supports the following parameters and sub-parameters:

* `pub`, `pub_crate` - Mark generated items as `pub` or `pub(crate)` respectively
* `returnval=<macro_class_name>` - Enable more complex mode where return values are handled. `call` functions turn into `try_call`. Input macros is used as a makeshift GAT trait with specialisation. There are a number of built-in channel classes and you can also have custom ones.
* `call`, `call_mut`, `call_once` - Generate enum's inherent impl functions to apply the enum instance to various forms of the object.
    * `allow_panic` - Allow generation of the function with `panic!()` calls inside
    * `extra_arg_type(<type>)` - Add additional argument to the `try_call` function. That argument will appear on all `macro_class_name!(send(...))` callbacks.
* `ref_proxy`, `mut_proxy`, `once_proxy` - Generate proxy struct to "convert" the trait to enum values, accepting `Fn`, `FnMut` and `FnOnce` respectively.
    * `infallible_impl` - Make the proxy also implement the original trait, provided your sink function does not err.
    * `unwrapping_impl` - Make the proxy also implement the original trait, using `unwrap` where needed.
    * `unwrapping_and_panicking_impl` - Force proxy to implement the original trait, using `panic!()` calls where complication would fail because of ownership requirements.
    * `extra_field_type` - Add additional second field to proxy struct. That field will be used as additional argument to `macro_class_name!(create(...))` and `macro_class_name!(recv(...))` callbacks.
    * `name=<ident>` - override the name of the generated struct
    * `traitname=<ident>` - override the name of the generated "resultified" trait
* `enum_attr` - Inject custom attribute (e.g. `enum_attr[derive(serde_derive::Serialize)]`)  into enum declaration. Can be repeated. You need to use square brackets for this.
* `name=<ident>` - override the name of the generated enum
* `inherent_impl` - Base enum on an inherent impl instead of a trait.


Example of attributes syntax:

```rust,ignore
#[trait_enumizer::enumizer(pub_crate, call_mut(allow_panic),ref_proxy(unwrapping_impl))]
```

Pseudo-derive-helpers:

* `#[enumizer_enum_attr[...]]` - Forward specified attribute to generated enum. Example: `#[enumizer_enum_attr[serde(rename="qqq")]]`. Can be attached to functions (which become enum variants) or to function arguments (which become enum variant fields).
* `#[enumizer_return_attr[...]]` - in `returnval` mode, attach custom attribute to the `ret` field of the enum.
* `#[enumizer_to_owned]` - For reference argument type, use owned value instead of trying to put reference to enum (which may not work, unless `'static`).

To understand how the crate functions, you can view some test files:

* [`simple_derive.rs`](crates/trait-enumizer/tests/simple_derive.rs) - Simple demo.
* [`simple_manual.rs`](crates/trait-enumizer/tests/simple_manual.rs) - Expanded implementation of the demo above.
* [`mutable_derive.rs`](crates/trait-enumizer/tests/mutable_derive.rs), [`mutable_manual.rs`](crates/trait-enumizer/tests/mutable_manual.rs), [`move_derive.rs`](crates/trait-enumizer/tests/move_derive.rs), [`move_manual.rs`](crates/trait-enumizer/tests/move_manual.rs) - the same, but for mut and once cases.
* [`mixed.rs`](crates/trait-enumizer/tests/mixed.rs) - Showcases some other features, also uses threading to demonstrates how to interact with channels (using [flume](https://crates.io/crates/flume) as example). Also showcases injecting custom derive into the generated enum.
* [`returnval_derive.rs`](crates/trait-enumizer/tests/returnval_derive.rs), [`returnval_manual_generic.rs`](crates/trait-enumizer/tests/returnval_manual_generic.rs) - Tricker mode for handling return values.
* [`returnval_manual_flume.rs`](crates/trait-enumizer/tests/returnval_manual_flume.rs) - Original version without the channel abstraction, hard coded for [flume](https://crates.io/crates/flume) instead.
* [`rpc.rs`](crates/trait-enumizer/tests/rpc.rs) - Demonstrates advanced usage of custom `returnval=` classes to make remote prodecure call using serde_json and flume. Two primary Flume channels simulate a socket, interim Flume channels route return values back to callers.
* [`toowned_manual`](crates/trait-enumizer/tests/toowned_manual.rs), [`toowned_derive`](crates/trait-enumizer/tests/toowned_derive.rs) - Expanded (manual) and automaitcally derived demonstration of `#[enumizer_to_owned]` feature.
* [`inherent_derive`](crates/trait-enumizer/tests/inherent_derive.rs) - demonstrates `inherent_impl` mode.


# See also

* [spaad](https://crates.io/crates/spaad)
