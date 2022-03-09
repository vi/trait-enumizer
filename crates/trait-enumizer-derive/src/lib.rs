use convert_case::Casing;
use proc_macro2::TokenStream;
use syn::Ident;


struct Argument {
    name: Ident,
    ty: syn::Type,
    enum_attr: Vec<proc_macro2::Group>,
}


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ReceiverStyle {
    Move,
    Mut,
    Ref,
}

struct Method {
    name: Ident,
    receiver_style: ReceiverStyle,
    args: Vec<Argument>,
    ret: Option<syn::Type>,
    enum_attr: Vec<proc_macro2::Group>,
    return_attr: Vec<proc_macro2::Group>,
}

impl Method {
    fn variant_name(&self) -> proc_macro2::Ident {
        quote::format_ident!(
            "{}",
            self.name
                .to_string()
                .to_case(convert_case::Case::UpperCamel)
        )
    }
}

impl std::fmt::Debug for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Method")
            .field("name", &self.name.to_string())
            .field("receiver_style", &self.receiver_style)
            .field("args", &self.args)
            .finish()
    }
}

struct TheTrait {
    name: Ident,
    methods: Vec<Method>,
}


mod parse_args;
mod util;
mod parse_trait;
mod generate;

impl TheTrait {
}

#[derive(Default)]
struct GenProxyParams {
    gen_infallible: bool,
    gen_unwrapping: bool,
    gen_unwrapping_and_panicking: bool,
}

#[derive(Default)]
struct CallFnParams {
    allow_panic: bool,
}

#[derive(PartialEq, Eq,Copy,Clone)]
enum AccessMode {
    Priv,
    Pub,
    PubCrate,
}
impl Default for AccessMode {
    fn default() -> Self {
        AccessMode::Priv
    }
}

#[derive(Default)]
struct Params {
    ref_proxy: Option<GenProxyParams>,
    mut_proxy: Option<GenProxyParams>,
    once_proxy: Option<GenProxyParams>,
    call_ref: Option<CallFnParams>,
    call_mut: Option<CallFnParams>,
    call_once: Option<CallFnParams>,
    access_mode: AccessMode,
    returnval: Option<proc_macro2::Ident>,
    enum_attr: Vec<proc_macro2::Group>,
}

#[proc_macro_attribute]
pub fn enumizer(
    attrs: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input: TokenStream = item.into();
    let attrs: TokenStream = attrs.into();

    let params =  parse_args::parse_args(attrs);

    let mut ret = TokenStream::new();
    let mut tra: syn::ItemTrait = syn::parse2(input).unwrap();
    let thetrait = TheTrait::parse(&mut tra, params.returnval.is_some());
    ret.extend(quote::quote!{#tra});
    //dbg!(thetrait);
    thetrait.generate_enum(&mut ret, params.access_mode, params.returnval.as_ref(), &params.enum_attr);


    let caller_inconv = thetrait.receiver_style_that_is_the_most_inconvenient_for_caller();

    if let Some(_g) = params.call_once {
        thetrait.generate_call_fn(&mut ret, ReceiverStyle::Move, params.access_mode, params.returnval.as_ref());
    }
    if let Some(g) = params.call_mut {
        if caller_inconv == ReceiverStyle::Move && ! g.allow_panic {
            panic!("Cannot generate `call_mut` function because of trait have `self` methods. Use `call_mut(allow_panic)` to override.");
        }
        thetrait.generate_call_fn(&mut ret, ReceiverStyle::Mut, params.access_mode, params.returnval.as_ref());
    }
    if let Some(g) = params.call_ref {
        if caller_inconv != ReceiverStyle::Ref && ! g.allow_panic {
            panic!("Cannot generate `call` function because of trait have non-`&self` methods. Use `call(allow_panic)` to override.");
        }
        thetrait.generate_call_fn(&mut ret, ReceiverStyle::Ref, params.access_mode, params.returnval.as_ref());
    }

    let callee_inconv = thetrait.receiver_style_that_is_the_most_inconvenient_for_callee();

    if let Some(g) = params.ref_proxy {
        thetrait.generate_resultified_trait(&mut ret, ReceiverStyle::Ref, params.access_mode, params.returnval.as_ref());
        thetrait.generate_proxy(&mut ret, ReceiverStyle::Ref, params.access_mode, params.returnval.as_ref());
        if g.gen_infallible {
            if params.returnval.is_some() {
                panic!("infallible_impl and returnval are incompatible");
            }
            thetrait.generate_infallible_impl(&mut ret, ReceiverStyle::Ref);
        }
        if g.gen_unwrapping || g.gen_unwrapping_and_panicking {
            thetrait.generate_unwrapping_impl(&mut ret, ReceiverStyle::Ref, params.returnval.as_ref());
        }
    }
    if let Some(g) = params.mut_proxy {
        thetrait.generate_resultified_trait(&mut ret, ReceiverStyle::Mut, params.access_mode, params.returnval.as_ref());
        thetrait.generate_proxy(&mut ret, ReceiverStyle::Mut, params.access_mode, params.returnval.as_ref());
        if g.gen_infallible || g.gen_unwrapping {
            if callee_inconv == ReceiverStyle::Ref {
                panic!("The trait contains &self methods. The mutable proxy cannot implement it. Use `unwrapping_and_panicking_impl` to force generation and retain only some methods");
            }
        }
        if g.gen_infallible {
            if params.returnval.is_some() {
                panic!("infallible_impl and returnval are incompatible");
            }
            thetrait.generate_infallible_impl(&mut ret, ReceiverStyle::Mut);
        }
        if g.gen_unwrapping || g.gen_unwrapping_and_panicking {
            thetrait.generate_unwrapping_impl(&mut ret, ReceiverStyle::Mut, params.returnval.as_ref());
        }
    }
    if let Some(g) = params.once_proxy {
        thetrait.generate_resultified_trait(&mut ret, ReceiverStyle::Move, params.access_mode, params.returnval.as_ref());
        thetrait.generate_proxy(&mut ret, ReceiverStyle::Move, params.access_mode, params.returnval.as_ref());
        if g.gen_infallible || g.gen_unwrapping {
            if callee_inconv != ReceiverStyle::Move {
                panic!("The trait contains `&self` or `&mut self` methods. The once proxy cannot implement it - only for traits with solely `self` methods. Use `unwrapping_and_panicking_impl` to force generation and retain only some methods");
            }
        }
        if g.gen_infallible {
            if params.returnval.is_some() {
                panic!("infallible_impl and returnval are incompatible");
            }
            thetrait.generate_infallible_impl(&mut ret, ReceiverStyle::Move);
        }
        if g.gen_unwrapping || g.gen_unwrapping_and_panicking {
            thetrait.generate_unwrapping_impl(&mut ret, ReceiverStyle::Move, params.returnval.as_ref());
        }
    }

    ret.into()
}

