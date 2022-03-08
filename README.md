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
* `returnval` - Enable more complex mode where return values are handled. `call` functions turn into `try_call`, enum and proxy gain generic parameters, unstable Rust is required.
* `call`, `call_mut`, `call_once` - Generate enum's inherent impl functions to apply the enum instance to various forms of the object.
    * `allow_panic` - Allow generation of the function with `panic!()` calls inside
* `ref_proxy`, `mut_proxy`, `once_proxy` - Generate proxy struct to "convert" the trait to enum values, accepting `Fn`, `FnMut` and `FnOnce` respectively.
    * `infallible_impl` - Make the proxy also implement the original trait, provided your sink function does not err.
    * `unwrapping_impl` - Make the proxy also implement the original trait, using `unwrap` where needed.
    * `unwrapping_and_panicking_impl` - Force proxy to implement the original trait, using `panic!()` calls where complication would fail because of ownership requirements.
* `enum_attr` - Inject custom attribute (e.g. `derive(serde_derive::Serialize)`)  into enum declaration. Can be repeated.

Example of attributes syntax:

```rust,ignore
#[trait_enumizer::enumizer(pub_crate, call_mut(allow_panic),ref_proxy(unwrapping_impl))]
```

To understand how the crate functions, you can view some test files:

* [`simple_derive.rs`](crates/trait-enumizer/tests/simple_derive.rs) - Simple demo.
* [`simple_manual.rs`](crates/trait-enumizer/tests/simple_manual.rs) - Expanded implementation of the demo above.
* [`mutable_derive.rs`](crates/trait-enumizer/tests/mutable_derive.rs), [`mutable_manual.rs`](crates/trait-enumizer/tests/mutable_manual.rs), [`move_derive.rs`](crates/trait-enumizer/tests/move_derive.rs), [`move_manual.rs`](crates/trait-enumizer/tests/move_manual.rs) - the same, but for mut and once cases.
* [`mixed.rs`](crates/trait-enumizer/tests/mixed.rs) - Showcases some other features, also uses threading to demonstrates how to interact with channels (using [flume](https://crates.io/crates/flume) as example).
* [`returnval_derive.rs`](crates/trait-enumizer/tests/returnval_derive.rs), [`returnval_manual_generic.rs`](crates/trait-enumizer/tests/returnval_manual_generic.rs) - Tricker mode for handling return values.
* [`returnval_manual_flume.rs`](crates/trait-enumizer/tests/returnval_manual_flume.rs) - Original version without the channel abstraction, hard coded for [flume](https://crates.io/crates/flume) instead.
