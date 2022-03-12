use convert_case::Casing;
use proc_macro2::TokenStream;
use quote::quote as q;

use crate::AccessMode;

use super::{ReceiverStyle, TheTrait};
impl TheTrait {
    pub(crate) fn generate_enum(
        &self,
        out: &mut TokenStream,
        am: AccessMode,
        returnval_mode: Option<&proc_macro2::Ident>,
        custom_attrs: &[proc_macro2::Group],
    ) {
        let am = am.code();
        let enum_name = quote::format_ident!("{}Enum", self.name);
        let mut variants = TokenStream::new();
        for method in &self.methods {
            let variant_name = method.variant_name();
            let mut variant_params = TokenStream::new();
            for arg in &method.args {
                let n = &arg.name;
                let t = if !arg.to_owned {
                    let ty = &arg.ty;
                    q!{#ty}
                } else {
                    match &arg.ty {
                        syn::Type::Reference(r) => {
                            let ty = &*r.elem;
                            q!{<#ty as ::std::borrow::ToOwned>::Owned}
                        }
                        _ => panic!("Argument marked with `#[enumizer_to_owned]` must be a &reference"),
                    }
                };
                let mut a = TokenStream::new();
                for aa in &arg.enum_attr {
                    a.extend(q!{# #aa});
                }
                variant_params.extend(q! {
                    #a #n : #t,
                });
            }
            if let Some(rv) = &method.ret {
                let mut ra = TokenStream::new();
                for aa in &method.return_attr {
                    ra.extend(q!{# #aa});
                }
                let chmacro = returnval_mode.unwrap(); 
                variant_params.extend(q!{
                    #ra ret : #chmacro ! (Sender<#rv>),
                });
            } else {
                if ! method.return_attr.is_empty() {
                    panic!("`enumizer_return_attr[]` used in method without a return type. Add `-> ()` to force using the return channel.");
                }
            }
            let mut a = TokenStream::new();
            for aa in &method.enum_attr {
                a.extend(q!{# #aa});
            }

            variants.extend(q! {
                #a #variant_name { #variant_params },
            });
        }
        let mut customattrs = TokenStream::new();
        for ca in custom_attrs {
            customattrs.extend(q!{# #ca});
        }
        out.extend(q! {
            #customattrs
            #am enum #enum_name {
                #variants
            }
        });
    }

    pub(crate) fn generate_call_fn(
        &self,
        out: &mut TokenStream,
        level: ReceiverStyle,
        am: AccessMode,
        returnval_mode: Option<&proc_macro2::Ident>,
        extra_arg: Option<&proc_macro2::TokenStream>,
    ) {
        let am = am.code();
        let enum_name = quote::format_ident!("{}Enum", self.name);
        let name = &self.name;
        let mut variants = TokenStream::new();
        for method in &self.methods {
            let variant_name = quote::format_ident!(
                "{}",
                method
                    .name
                    .to_string()
                    .to_case(convert_case::Case::UpperCamel)
            );
            let method_name = &method.name;
            let mut variant_params = TokenStream::new();
            let mut variant_params_with_ret = TokenStream::new();
            for arg in &method.args {
                let n = &arg.name;
                //let t = &arg.ty;
                variant_params_with_ret.extend(q! {
                    #n,
                });
                if !arg.to_owned {
                    variant_params.extend(q! {
                        #n,
                    });
                } else {
                    variant_params.extend(q! {
                        ::std::borrow::Borrow::borrow(& #n),
                    });
                }
            }
            if let Some(_rt) = &method.ret {
                variant_params_with_ret.extend(q! {
                    ret,
                });
            }
            let can_do_it = match (level, method.receiver_style) {
                (ReceiverStyle::Move, _) => true,
                (ReceiverStyle::Mut, ReceiverStyle::Move) => false,
                (ReceiverStyle::Mut, _) => true,
                (ReceiverStyle::Ref, ReceiverStyle::Ref) => true,
                (ReceiverStyle::Ref, _) => false,
            };
            let action = if can_do_it {
                if let Some(rt) = &method.ret {
                    if let Some(m) = returnval_mode {
                        let ea = if extra_arg.is_some() {
                            q!{, extra_arg}
                        } else {
                            q!{}
                        };
                        q! { Ok(#m ! (send::<#rt>(ret, o.#method_name(#variant_params) #ea))?)  }
                    } else {
                        unreachable!("parsing function should have already rejected this case");
                    }
                } else {
                    if returnval_mode.is_none() {
                        q! {o.#method_name(#variant_params)}
                    } else {
                        q! {Ok(o.#method_name(#variant_params))}
                    }
                }
            } else {
                let literal1 = proc_macro2::Literal::string(&format!(
                    "{}Enum::{}",
                    self.name,
                    level.call_fn_name(returnval_mode.is_some())
                ));
                let literal2 = proc_macro2::Literal::string(&method_name.to_string());
                q! {panic!("Cannot call `{}` from `{}` due to incompative `self` access mode", #literal2, #literal1)}
            };
            variants.extend(q! {
                #enum_name::#variant_name { #variant_params_with_ret } => #action,
            });
        }

        let fnname = level.call_fn_ts(returnval_mode.is_some());
        let o = match level {
            ReceiverStyle::Move => q! {mut o: I},
            ReceiverStyle::Mut => q! {o: &mut I},
            ReceiverStyle::Ref => q! {o: &I},
        };

        let rett = if let Some(m) = returnval_mode {
            q!{ -> Result<(), #m ! (SendError)>}
        } else {
            q!{}
        };
        let ea = if let Some(extr) = extra_arg {
            q!{, extra_arg : #extr}
        } else {
            q!{}
        };
        out.extend(q! {
            impl #enum_name {
                #am fn #fnname<I: #name>(self, #o #ea) #rett {
                    match self {
                        #variants
                    }
                }
            }
        });
    }
    pub(crate) fn generate_resultified_trait(
        &self,
        out: &mut TokenStream,
        level: ReceiverStyle,
        am: AccessMode,
        returnval_mode: Option<&proc_macro2::Ident>,
    ) {
        let am = am.code();
        let rt_name = quote::format_ident!("{}Resultified{}", self.name, level.identpart());
        //let name = &self.name;
        let mut methods = TokenStream::new();
        for method in &self.methods {
            let rt_method_name = quote::format_ident!("try_{}", method.name,);
            // let method_name = &method.name;
            let mut args = TokenStream::new();
            for arg in &method.args {
                let n = &arg.name;
                let t = &arg.ty;
                args.extend(q! {
                    #n : #t,
                });
            }
            let slf = level.ts();
            let ret = if let Some(rt) = &method.ret {
                if let Some(m) = returnval_mode {
                    q!{::std::result::Result<#rt, #m ! ( RecvError )>}
                } else {
                    unreachable!("Should had been rejected earlier")
                }
            } else {
                q!{()}
            };
            methods.extend(q! {
                fn #rt_method_name(#slf, #args ) -> ::std::result::Result<#ret, E>;
            });
        }

        out.extend(q! {
            #am trait #rt_name<E> {
                #methods
            }
        });
    }

    pub(crate) fn generate_proxy(
        &self,
        out: &mut TokenStream,
        level: ReceiverStyle,
        am: AccessMode,
        returnval_mode: Option<&proc_macro2::Ident>,
        extra_arg: Option<&proc_macro2::TokenStream>,
    ) {
        let am = am.code();
        let enum_name = quote::format_ident!("{}Enum", self.name);
        let rt_name = quote::format_ident!("{}Resultified{}", self.name, level.identpart());
        let proxy_name = quote::format_ident!("{}Proxy{}", self.name, level.identpart());
        //let name = &self.name;
        let mut methods = TokenStream::new();
        for method in &self.methods {
            let rt_method_name = quote::format_ident!("try_{}", method.name,);
            //let method_name = &method.name;
            let variant_name = method.variant_name();
            let mut args_with_types = TokenStream::new();
            let mut args_without_types = TokenStream::new();
            for arg in &method.args {
                let n = &arg.name;
                let t = &arg.ty;
                args_with_types.extend(q! {
                    #n : #t,
                });
                if ! arg.to_owned {
                    args_without_types.extend(q! {
                        #n,
                    });
                } else {
                    args_without_types.extend(q! {
                        #n: ::std::borrow::ToOwned::to_owned(#n),
                    });
                }
            }
            let slf = level.ts();
            if let Some(rt) = &method.ret {
                let chmacro = returnval_mode.unwrap();
                let (ea_with_comma, ea) = if let Some(_eat) = extra_arg {
                    (q!{, self.1}, q!{self.1})
                } else {
                    (q!{}, q!{})
                };
                methods.extend(q! {
                    fn #rt_method_name(#slf, #args_with_types ) -> ::std::result::Result<::std::result::Result<#rt, #chmacro ! (RecvError)>, E> {
                        let (tx, rx) = #chmacro !(create::<#rt>(#ea));
                        self.0(#enum_name::#variant_name { #args_without_types ret: tx })?;
                        Ok(#chmacro ! (recv::<#rt>(rx #ea_with_comma) ) )
                    }
                });
            } else {
                methods.extend(q! {
                    fn #rt_method_name(#slf, #args_with_types ) -> ::std::result::Result<(), E> {
                        self.0(#enum_name::#variant_name{ #args_without_types })
                    }
                });
            };
        }
        let fn_trait = level.fn_trait();

        let ea = if let Some(eat) = extra_arg {
            q!{, pub #eat}
        } else {
            q!{}
        };

        out.extend(q! {
            #am struct #proxy_name<E, F: #fn_trait(#enum_name)-> ::std::result::Result<(), E> > (pub F #ea);

            impl<E, F: #fn_trait(#enum_name) -> ::std::result::Result<(), E>> #rt_name<E> for #proxy_name<E, F> {
                #methods
            }
        });
    }
    pub(crate) fn generate_infallible_impl(&self, out: &mut TokenStream, level: ReceiverStyle) {
        let rt_name = quote::format_ident!("{}Resultified{}", self.name, level.identpart());
        let name = &self.name;
        let mut methods = TokenStream::new();
        for method in &self.methods {
            let rt_method_name = quote::format_ident!("try_{}", method.name,);
            let method_name = &method.name;
            let mut args_with_types = TokenStream::new();
            let mut args_without_types = TokenStream::new();
            for arg in &method.args {
                let n = &arg.name;
                let t = &arg.ty;
                args_with_types.extend(q! {
                    #n : #t,
                });
                args_without_types.extend(q! {
                    #n,
                });
            }
            let slf = method.receiver_style.ts();
            methods.extend(q! {
                fn #method_name(#slf, #args_with_types ) {
                    R::#rt_method_name(self, #args_without_types).unwrap()
                }
            });
        }
        out.extend(q! {
            impl<R:#rt_name<::std::convert::Infallible>> #name for R {
                #methods
            }
        });
    }

    pub(crate) fn generate_unwrapping_impl(
        &self,
        out: &mut TokenStream,
        level: ReceiverStyle,
        returnval_mode: Option<&proc_macro2::Ident>,
    ) {
        let rt_name = quote::format_ident!("{}Resultified{}", self.name, level.identpart());
        let proxy_name = quote::format_ident!("{}Proxy{}", self.name, level.identpart());
        let fn_trait = level.fn_trait();
        let enum_name = quote::format_ident!("{}Enum", self.name);
        let name = &self.name;
        let mut methods = TokenStream::new();
        for method in &self.methods {
            let rt_method_name = quote::format_ident!("try_{}", method.name,);
            let method_name = &method.name;
            let mut args_with_types = TokenStream::new();
            let mut args_without_types = TokenStream::new();
            for arg in &method.args {
                let n = &arg.name;
                let t = &arg.ty;
                args_with_types.extend(q! {
                    #n : #t,
                });
                args_without_types.extend(q! {
                    #n,
                });
            }
            let slf = method.receiver_style.ts();
            let can_call = match (method.receiver_style, level) {
                (ReceiverStyle::Move, _) => true,
                (ReceiverStyle::Mut, ReceiverStyle::Move) => false,
                (ReceiverStyle::Mut, ReceiverStyle::Mut) => true,
                (ReceiverStyle::Mut, ReceiverStyle::Ref) => true,
                (ReceiverStyle::Ref, ReceiverStyle::Move) => false,
                (ReceiverStyle::Ref, ReceiverStyle::Mut) => false,
                (ReceiverStyle::Ref, ReceiverStyle::Ref) => true,
            };
            let mut second_unwrap = q!{};
            let rett = if let Some(rt) = &method.ret {
                second_unwrap = q!{.unwrap()};
                q!{-> #rt}
            } else {
                q!{}
            };
            if can_call {
                let slf2 = match (method.receiver_style, level) {
                    (ReceiverStyle::Move, ReceiverStyle::Ref) => q! {&self},
                    (ReceiverStyle::Move, ReceiverStyle::Mut) => q! {&mut self},
                    _ => q! {self},
                };
                methods.extend(q! {
                    fn #method_name(#slf, #args_with_types ) #rett {
                        #rt_name::#rt_method_name(#slf2, #args_without_types).unwrap() #second_unwrap
                    }
                });
            } else {
                let literal1 = proc_macro2::Literal::string(&self.name.to_string());
                let literal2 = proc_macro2::Literal::string(&method.name.to_string());
                let literal3 = proc_macro2::Literal::string(&proxy_name.to_string());
                methods.extend(q! {
                    fn #method_name(#slf, #args_with_types ) #rett {
                        panic!("Cannot call {}::{} accepting too weak `self` on {}", #literal1, #literal2, #literal3)
                    }
                });
            }
        }
        let wh1 = if let Some(m) = returnval_mode {
            q!{,#m ! (RecvError) : ::std::fmt::Debug}
        } else {
            q!{}
        };
        out.extend(q! {
            impl<E, F: #fn_trait(#enum_name) -> ::std::result::Result<(), E>>  #name for #proxy_name<E,F> where E : ::std::fmt::Debug #wh1 {
                #methods
            }
        });
    }
}
