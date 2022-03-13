use convert_case::Casing;
use proc_macro2::TokenStream;
use syn::Ident;

struct Argument {
    name: Ident,
    ty: syn::Type,
    enum_attr: Vec<proc_macro2::Group>,
    to_owned: bool,
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
    r#async: bool,
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

struct InputData {
    /// Source trait or inherent impl name.
    name: Ident,
    methods: Vec<Method>,
    params: Params,
}

mod generate;
mod parse_args;
mod parse_input;
mod util;


struct GenProxyParams {
    level: ReceiverStyle,
    gen_infallible: bool,
    gen_unwrapping: bool,
    gen_unwrapping_and_panicking: bool,
    extra_arg: Option<proc_macro2::TokenStream>,
    name: Ident,
    traitname: Option<Ident>,
    r#async: bool,
}
impl GenProxyParams {
    fn some_impl_requested(&self) -> bool {
        self.gen_infallible || self.gen_unwrapping || self.gen_unwrapping_and_panicking
    }
}

struct CallFnParams {
    level: ReceiverStyle,
    allow_panic: bool,
    extra_arg: Option<proc_macro2::TokenStream>,
    name: Ident,
    r#async: bool,
}


#[derive(PartialEq, Eq, Copy, Clone, Debug)]
enum AccessMode {
    Priv,
    Pub,
    PubCrate,
}


struct Params {
    proxies: Vec<GenProxyParams>,
    call_fns: Vec<CallFnParams>,
    access_mode: AccessMode,
    returnval: Option<proc_macro2::Ident>,
    enum_attr: Vec<proc_macro2::Group>,
    enum_name: Ident,
    inherent_impl_mode : bool,
}

#[proc_macro_attribute]
pub fn enumizer(
    attrs: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input: TokenStream = item.into();
    let attrs: TokenStream = attrs.into();

    let params = parse_args::parse_args(attrs);

    let mut ret = TokenStream::new();
    let input_data = if ! params.inherent_impl_mode {
        let mut tra: syn::ItemTrait = syn::parse2(input).unwrap();
        let input_data = InputData::parse_trait(&mut tra, params);
        ret.extend(quote::quote! {#tra});
        input_data
    } else {
        let mut imp: syn::ItemImpl = syn::parse2(input).unwrap();
        let input_data = InputData::parse_inherent_impl(&mut imp, params);
        ret.extend(quote::quote! {#imp});
        input_data
    };
    let params = &input_data.params;
   
    //dbg!(thetrait);
    input_data.generate_enum(&mut ret);

    let caller_inconv = input_data.receiver_style_that_is_the_most_inconvenient_for_caller();

    for g in &params.call_fns {
        match g.level {
            ReceiverStyle::Move => (),
            ReceiverStyle::Mut => {
                if caller_inconv == ReceiverStyle::Move && !g.allow_panic {
                    panic!("Cannot generate `call_fn(ref_mut)` function because of trait have `self` methods. Use `call_fn(... ,allow_panic)` to override.");
                }
            }
            ReceiverStyle::Ref => {
                if caller_inconv != ReceiverStyle::Ref && !g.allow_panic {
                    panic!("Cannot generate `call_fn(ref)` function because of trait have non-`&self` methods. Use `call_fn(... ,allow_panic)` to override.");
                }
            }
        }
        input_data.generate_call_fn(&mut ret, g);
    }

    let callee_inconv = input_data.receiver_style_that_is_the_most_inconvenient_for_callee();

    for g in &params.proxies {
        if params.inherent_impl_mode && g.some_impl_requested() {
            panic!("Generating trait impls is incompatible with inherent_impl mode");
        }

        if g.gen_infallible {
            if params.returnval.is_some() {
                panic!("infallible_impl and returnval are incompatible");
            }
        }
        match g.level {
            ReceiverStyle::Move => {
                if g.gen_infallible || g.gen_unwrapping {
                    if callee_inconv != ReceiverStyle::Move {
                        panic!("The trait contains `&self` or `&mut self` methods. The FnOnce proxy cannot implement it - only for traits with solely `self` methods. Use `unwrapping_and_panicking_impl` to force generation and retain only some methods");
                    }
                }
            }
            ReceiverStyle::Mut => {
                if g.gen_infallible || g.gen_unwrapping {
                    if callee_inconv == ReceiverStyle::Ref {
                        panic!("The trait contains &self methods. The FnMut proxy cannot implement it. Use `unwrapping_and_panicking_impl` to force generation and retain only some methods");
                    }
                }
            }
            ReceiverStyle::Ref => {
               
            }
        }

        if g.traitname.is_some() {
            input_data.generate_resultified_trait(&mut ret, g);
        }
        input_data.generate_proxy(&mut ret, g);
        if g.gen_infallible {
            input_data.generate_infallible_impl(&mut ret, g);
        }
        if g.gen_unwrapping || g.gen_unwrapping_and_panicking {
            input_data.generate_unwrapping_impl(&mut ret, g);
        }
    }

    ret.into()
}
