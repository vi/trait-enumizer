use convert_case::Casing;
use proc_macro2::TokenStream;
use syn::Ident;

use quote::quote as q;

struct Argument {
    name: Ident,
    ty: syn::Type,
}

impl std::fmt::Debug for Argument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let t = &self.ty;
        f.debug_struct("Argument")
            .field("name", &self.name.to_string())
            .field("ty", &format!("{}", q! {#t}))
            .finish()
    }
}

#[derive(Clone,Copy,Debug)]
enum ReceiverStyle {
    Move,
    Ref,
    Mut,
}
impl ReceiverStyle {
    fn ts(self) -> TokenStream {
        match self {
            ReceiverStyle::Move => q!{self},
            ReceiverStyle::Ref => q!{&self},
            ReceiverStyle::Mut => q!{&mut self},
        }
    }
}

struct Method {
    name: Ident,
    receiver_style: ReceiverStyle,
    args: Vec<Argument>,
}

impl Method {
    fn variant_name(&self) -> proc_macro2::Ident {
        quote::format_ident!(
            "{}",
            self.name.to_string().to_case(convert_case::Case::UpperCamel)
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

impl std::fmt::Debug for TheTrait {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TheTrait")
            .field("name", &self.name.to_string())
            .field("methods", &self.methods)
            .finish()
    }
}

impl TheTrait {
    fn parse(item: syn::ItemTrait) -> TheTrait {
        let mut methods = Vec::with_capacity(item.items.len());

        for x in item.items {
            match x {
                syn::TraitItem::Method(m) => {
                    let sig = m.sig;
                    if sig.constness.is_some() {
                        eprintln!("Warning: consness ignored");
                    }
                    if sig.asyncness.is_some() {
                        panic!("Async not supported by trait-enumizer");
                    }
                    if sig.unsafety.is_some() {
                        panic!("Unsafe not supported by trait-enumizer");
                    }
                    if sig.abi.is_some() {
                        panic!("Trait-enumizer does not support custom ABI in trait methods")
                    }
                    if !sig.generics.params.is_empty() {
                        panic!("Trait-enumizer does not support generics or lifetimes in trait methods")
                    }
                    if sig.variadic.is_some() {
                        panic!("Trait-enumizer does not support variadics")
                    }
                    if !matches!(sig.output, syn::ReturnType::Default) {
                        panic!("Trait-enumizer does not yet support return values in methods")
                    }

                    let mut args = Vec::with_capacity(sig.inputs.len());

                    let mut receiver_style = None;

                    for inp in sig.inputs {
                        match inp {
                            syn::FnArg::Receiver(r) => {
                                receiver_style = if let Some(rr) = r.reference {
                                    if rr.1.is_some() {
                                        panic!("Trait-enumizer does not support explicit lifetimes");
                                    }
                                    if r.mutability.is_some() {
                                        Some(ReceiverStyle::Mut)
                                    } else {
                                        Some(ReceiverStyle::Ref)
                                    }
                                } else {
                                    Some(ReceiverStyle::Move)
                                }
                            }
                            syn::FnArg::Typed(arg) => {
                                match *arg.pat {
                                    syn::Pat::Ident(pi) => {
                                        if pi.by_ref.is_some() {
                                            panic!("Trait-enumizer does not support `ref`");
                                        }
                                        args.push(Argument { name: pi.ident, ty: *arg.ty });
                                    }
                                    _ => panic!("Trait-enumizer does not support method arguments that are not just simple identifiers"),
                                }
                            }
                        }
                    }

                    if receiver_style.is_none() {
                        panic!("Trait-enumizer does not support methods that do not accept `self`")
                    }

                    let method = Method {
                        args,
                        name: sig.ident,
                        receiver_style: receiver_style.unwrap(),
                    };
                    methods.push(method);
                }
                syn::TraitItem::Const(_) => {
                    panic!("Trait-enumizer does not support associated consts")
                }
                syn::TraitItem::Type(_) => {
                    panic!("Trait-enumizer does not support associated types")
                }
                syn::TraitItem::Macro(_) => {
                    panic!("Trait-enumizer does not support macro calls inside trait definition")
                }
                _ => (),
            }
        }

        TheTrait {
            name: item.ident,
            methods,
        }
    }

    fn generate_enum(&self, out: &mut TokenStream) {
        let enum_name = quote::format_ident!("{}Enum", self.name);
        let mut variants = TokenStream::new();
        for method in &self.methods {
            let variant_name = method.variant_name();
            let mut variant_params = TokenStream::new();
            for arg in &method.args {
                let n = &arg.name;
                let t = &arg.ty;
                variant_params.extend(q!{
                    #n : #t,
                });
            }
            variants.extend(q!{
                #variant_name { #variant_params },
            });
        }
        out.extend(q! {
            enum #enum_name {
                #variants
            }
        });
    }

    fn generate_call_fn(&self, out: &mut TokenStream) { 
        let enum_name = quote::format_ident!("{}Enum", self.name);
        let name = &self.name;
        let mut variants = TokenStream::new();
        for method in &self.methods {
            let variant_name = quote::format_ident!(
                "{}",
                method.name.to_string().to_case(convert_case::Case::UpperCamel)
            );
            let method_name = &method.name;
            let mut variant_params = TokenStream::new();
            for arg in &method.args {
                let n = &arg.name;
                //let t = &arg.ty;
                variant_params.extend(q!{
                    #n,
                });
            }
            variants.extend(q!{
                #enum_name::#variant_name { #variant_params } => o.#method_name(#variant_params),
            });
        }
        out.extend(q! {
            impl #enum_name {
                fn call<I: #name>(self, o: &I) {
                    match self {
                        #variants
                    }
                }
            }
        });
    }
    fn generate_resultified_trait(&self, out: &mut TokenStream) { 
        let rt_name = quote::format_ident!("{}Resultified", self.name);
        //let name = &self.name;
        let mut methods = TokenStream::new();
        for method in &self.methods {
            let rt_method_name = quote::format_ident!(
                "try_{}",
                method.name,
            );
            // let method_name = &method.name;
            let mut args = TokenStream::new();
            for arg in &method.args {
                let n = &arg.name;
                let t = &arg.ty;
                args.extend(q!{
                    #n : #t,
                });
            }
            let slf = method.receiver_style.ts();
            methods.extend(q!{
                fn #rt_method_name(#slf, #args ) -> ::std::result::Result<(), E>;
            });
        }
        out.extend(q! {
            trait #rt_name<E> {
                #methods
            }
        });
    }

    fn generate_resultified_trait_blanked_impl(&self, out: &mut TokenStream) {
        let rt_name = quote::format_ident!("{}Resultified", self.name);
        let name = &self.name;
        let mut methods = TokenStream::new();
        for method in &self.methods {
            let rt_method_name = quote::format_ident!(
                "try_{}",
                method.name,
            );
            let method_name = &method.name;
            let mut args_with_types = TokenStream::new();
            let mut args_without_types = TokenStream::new();
            for arg in &method.args {
                let n = &arg.name;
                let t = &arg.ty;
                args_with_types.extend(q!{
                    #n : #t,
                });
                args_without_types.extend(q!{
                    #n,
                });
            }
            let slf = method.receiver_style.ts();
            methods.extend(q!{
                fn #method_name(#slf, #args_with_types ) {
                    R::#rt_method_name(#slf, #args_without_types).unwrap()
                }
            });
        }
        out.extend(q! {
            impl<R:#rt_name<::std::convert::Infallible>> #name for R {
                #methods
            }
        });
    }

    fn generate_resultified_proxy(&self, out: &mut TokenStream) {
        let enum_name = quote::format_ident!("{}Enum", self.name);
        let rt_name = quote::format_ident!("{}Resultified", self.name);
        let proxy_name = quote::format_ident!("{}Proxy", self.name);
        //let name = &self.name;
        let mut methods = TokenStream::new();
        for method in &self.methods {
            let rt_method_name = quote::format_ident!(
                "try_{}",
                method.name,
            );
            //let method_name = &method.name;
            let variant_name = method.variant_name();
            let mut args_with_types = TokenStream::new();
            let mut args_without_types = TokenStream::new();
            for arg in &method.args {
                let n = &arg.name;
                let t = &arg.ty;
                args_with_types.extend(q!{
                    #n : #t,
                });
                args_without_types.extend(q!{
                    #n,
                });
            }
            let slf = method.receiver_style.ts();
            methods.extend(q!{
                fn #rt_method_name(#slf, #args_with_types ) -> ::std::result::Result<(), E> {
                    self.0(#enum_name::#variant_name{ #args_without_types })
                }
            });
        }
        out.extend(q! {
            struct #proxy_name<E, F: Fn(#enum_name)-> ::std::result::Result<(), E> > (F);

            impl<E, F: Fn(#enum_name) -> ::std::result::Result<(), E>> #rt_name<E> for #proxy_name<E, F> {
                #methods
            }
        });
    }
}

#[proc_macro_attribute]
pub fn enumizer(
    _attrs: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input: TokenStream = item.into();
    let mut ret = input.clone();
    let tra: syn::ItemTrait = syn::parse2(input).unwrap();
    let thetrait = TheTrait::parse(tra);
    //dbg!(thetrait);
    thetrait.generate_enum(&mut ret);
    thetrait.generate_call_fn(&mut ret);
    thetrait.generate_resultified_trait(&mut ret);
    thetrait.generate_resultified_trait_blanked_impl(&mut ret);
    thetrait.generate_resultified_proxy(&mut ret);
    ret.into()
}
