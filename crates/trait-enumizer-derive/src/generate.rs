use convert_case::Casing;
use proc_macro2::TokenStream;
use quote::quote as q;

use crate::{CallFnParams, GenProxyParams};

use super::{ReceiverStyle, InputData};
impl InputData {
    pub(crate) fn generate_enum(
        &self,
        out: &mut TokenStream,
    ) {
        let returnval_handler = self.params.returnval.as_ref();
        let custom_attrs = &self.params.enum_attr[..];
        let pub_or_priv = self.params.access_mode.code();
        let enum_name = self.enum_name();
        let mut variants = TokenStream::new();
        for method in &self.methods {
            let variant_name = method.variant_name();
            let mut variant_params = TokenStream::new();
            for arg in &method.args {
                let argument_name = &arg.name;
                let argument_type = if !arg.to_owned {
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
                let mut custom_attributes = TokenStream::new();
                for aa in &arg.enum_attr {
                    custom_attributes.extend(q!{# #aa});
                }
                variant_params.extend(q! {
                    #custom_attributes #argument_name : #argument_type,
                });
            }
            if let Some(return_type) = &method.ret {
                let mut custom_attributes = TokenStream::new();
                for aa in &method.return_attr {
                    custom_attributes.extend(q!{# #aa});
                }
                let returnval_macro = returnval_handler.unwrap(); 
                variant_params.extend(q!{
                    #custom_attributes ret : #returnval_macro ! (Sender<#return_type>),
                });
            } else {
                if ! method.return_attr.is_empty() {
                    panic!("`enumizer_return_attr[]` used in method without a return type. Add `-> ()` to force using the return channel.");
                }
            }
            let mut custom_attributes = TokenStream::new();
            for aa in &method.enum_attr {
                custom_attributes.extend(q!{# #aa});
            }

            variants.extend(q! {
                #custom_attributes #variant_name { #variant_params },
            });
        }
        let mut customattrs = TokenStream::new();
        for ca in custom_attrs {
            customattrs.extend(q!{# #ca});
        }
        out.extend(q! {
            #customattrs
            #pub_or_priv enum #enum_name {
                #variants
            }
        });
    }

    pub(crate) fn generate_call_fn(
        &self,
        out: &mut TokenStream,
        cfparams : &CallFnParams,
    ) {
        let pub_or_priv = self.params.access_mode.code();
        let returnval_handler = self.params.returnval.as_ref();
        let extra_arg = cfparams.extra_arg.as_ref();
        let level = cfparams.level;
        let enum_name = self.enum_name();
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
                let argname = &arg.name;
                //let t = &arg.ty;
                variant_params_with_ret.extend(q! {
                    #argname,
                });
                if !arg.to_owned {
                    variant_params.extend(q! {
                        #argname,
                    });
                } else {
                    variant_params.extend(q! {
                        ::std::borrow::Borrow::borrow(& #argname),
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
                if let Some(return_type) = &method.ret {
                    if let Some(returnval_handler_macro) = returnval_handler {
                        let maybe_extraarg = if extra_arg.is_some() {
                            q!{, extra_arg}
                        } else {
                            q!{}
                        };
                        q! { Ok(#returnval_handler_macro ! (send::<#return_type>(ret, o.#method_name(#variant_params) #maybe_extraarg))?)  }
                    } else {
                        unreachable!("parsing function should have already rejected this case");
                    }
                } else {
                    if returnval_handler.is_none() {
                        q! {o.#method_name(#variant_params)}
                    } else {
                        q! {Ok(o.#method_name(#variant_params))}
                    }
                }
            } else {
                let literal1 = proc_macro2::Literal::string(&format!(
                    "{}Enum::{}",
                    self.name,
                    level.call_fn_name(returnval_handler.is_some())
                ));
                let literal2 = proc_macro2::Literal::string(&method_name.to_string());
                q! {panic!("Cannot call `{}` from `{}` due to incompative `self` access mode", #literal2, #literal1)}
            };
            variants.extend(q! {
                #enum_name::#variant_name { #variant_params_with_ret } => #action,
            });
        }

        let fnname = level.call_fn_ts(returnval_handler.is_some());
        let arg_o_with_type = match level {
            ReceiverStyle::Move => q! {mut o: I},
            ReceiverStyle::Mut => q! {o: &mut I},
            ReceiverStyle::Ref => q! {o: &I},
        };

        let maybe_returntype = if let Some(returnval_handler_macro) = returnval_handler {
            q!{ -> Result<(), #returnval_handler_macro ! (SendError)>}
        } else {
            q!{}
        };
        let maybe_extraarg = if let Some(extr) = extra_arg {
            q!{, extra_arg : #extr}
        } else {
            q!{}
        };
        out.extend(q! {
            impl #enum_name {
                #pub_or_priv fn #fnname<I: #name>(self, #arg_o_with_type #maybe_extraarg) #maybe_returntype {
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
        gpparams: &GenProxyParams,
    ) {
        let pub_or_priv = self.params.access_mode.code();
        let level = gpparams.level;
        let returnval_handler = self.params.returnval.as_ref();
        let resultified_trait_name = self.resultified_trait_name(gpparams);
        //let name = &self.name;
        let mut methods = TokenStream::new();
        for method in &self.methods {
            let rt_method_name = quote::format_ident!("try_{}", method.name,);
            // let method_name = &method.name;
            let mut args = TokenStream::new();
            for arg in &method.args {
                let argname = &arg.name;
                let argtype = &arg.ty;
                args.extend(q! {
                    #argname : #argtype,
                });
            }
            let slf = level.ts();
            let ret = if let Some(return_type) = &method.ret {
                if let Some(returnval_handler_macro) = returnval_handler {
                    q!{::std::result::Result<#return_type, #returnval_handler_macro ! ( RecvError )>}
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
            #pub_or_priv trait #resultified_trait_name<E> {
                #methods
            }
        });
    }

    pub(crate) fn generate_proxy(
        &self,
        out: &mut TokenStream,
        gpparams: &GenProxyParams,
    ) {
        let pub_or_priv = self.params.access_mode.code();
        let returnval_handler = self.params.returnval.as_ref();
        let extra_arg = gpparams.extra_arg.as_ref();
        let level = gpparams.level;
        let enum_name = self.enum_name();
        let resultified_trait_name = self.resultified_trait_name(gpparams);
        let proxy_name = self.proxy_name(gpparams);
        //let name = &self.name;
        let mut methods = TokenStream::new();
        for method in &self.methods {
            let rt_method_name = quote::format_ident!("try_{}", method.name,);
            //let method_name = &method.name;
            let variant_name = method.variant_name();
            let mut args_with_types_for_signature = TokenStream::new();
            let mut enum_variant_fields = TokenStream::new();
            for arg in &method.args {
                let argname = &arg.name;
                let argtype = &arg.ty;
                args_with_types_for_signature.extend(q! {
                    #argname : #argtype,
                });
                if ! arg.to_owned {
                    enum_variant_fields.extend(q! {
                        #argname,
                    });
                } else {
                    enum_variant_fields.extend(q! {
                        #argname: ::std::borrow::ToOwned::to_owned(#argname),
                    });
                }
            }
            let slf = level.ts();
            if let Some(rt) = &method.ret {
                let returnval_handler_macro = returnval_handler.unwrap();
                let (maybe_extraarg_with_comma, maybe_extraarg) = if let Some(_eat) = extra_arg {
                    (q!{, self.1}, q!{self.1})
                } else {
                    (q!{}, q!{})
                };
                methods.extend(q! {
                    fn #rt_method_name(#slf, #args_with_types_for_signature ) -> ::std::result::Result<::std::result::Result<#rt, #returnval_handler_macro ! (RecvError)>, E> {
                        let (tx, rx) = #returnval_handler_macro !(create::<#rt>(#maybe_extraarg));
                        self.0(#enum_name::#variant_name { #enum_variant_fields ret: tx })?;
                        Ok(#returnval_handler_macro ! (recv::<#rt>(rx #maybe_extraarg_with_comma) ) )
                    }
                });
            } else {
                methods.extend(q! {
                    fn #rt_method_name(#slf, #args_with_types_for_signature ) -> ::std::result::Result<(), E> {
                        self.0(#enum_name::#variant_name{ #enum_variant_fields })
                    }
                });
            };
        }
        let fn_trait = level.fn_trait();

        let maybe_extraarg = if let Some(eat) = extra_arg {
            q!{, pub #eat}
        } else {
            q!{}
        };

        out.extend(q! {
            #pub_or_priv struct #proxy_name<E, F: #fn_trait(#enum_name)-> ::std::result::Result<(), E> > (pub F #maybe_extraarg);

            impl<E, F: #fn_trait(#enum_name) -> ::std::result::Result<(), E>> #resultified_trait_name<E> for #proxy_name<E, F> {
                #methods
            }
        });
    }
    pub(crate) fn generate_infallible_impl(&self, out: &mut TokenStream, gpparams: &GenProxyParams) {
        let resultified_trait_name = self.resultified_trait_name(gpparams);
        let name = &self.name;
        let mut methods = TokenStream::new();
        for method in &self.methods {
            let rt_method_name = quote::format_ident!("try_{}", method.name,);
            let method_name = &method.name;
            let mut args_for_signature = TokenStream::new();
            let mut args_for_calling = TokenStream::new();
            for arg in &method.args {
                let n = &arg.name;
                let t = &arg.ty;
                args_for_signature.extend(q! {
                    #n : #t,
                });
                args_for_calling.extend(q! {
                    #n,
                });
            }
            let slf = method.receiver_style.ts();
            methods.extend(q! {
                fn #method_name(#slf, #args_for_signature ) {
                    R::#rt_method_name(self, #args_for_calling).unwrap()
                }
            });
        }
        out.extend(q! {
            impl<R:#resultified_trait_name<::std::convert::Infallible>> #name for R {
                #methods
            }
        });
    }

    pub(crate) fn generate_unwrapping_impl(
        &self,
        out: &mut TokenStream,
        gpparams: &GenProxyParams,
    ) {
        let returnval_handler = self.params.returnval.as_ref();
        let level = gpparams.level;
        let resultified_trait_name = self.resultified_trait_name(gpparams);
        let proxy_name = self.proxy_name(gpparams);
        let fn_trait = level.fn_trait();
        let enum_name = self.enum_name();
        let name = &self.name;
        let mut methods = TokenStream::new();
        for method in &self.methods {
            let rt_method_name = quote::format_ident!("try_{}", method.name,);
            let method_name = &method.name;
            let mut args_with_types = TokenStream::new();
            let mut args_without_types = TokenStream::new();
            for arg in &method.args {
                let argname = &arg.name;
                let argtype = &arg.ty;
                args_with_types.extend(q! {
                    #argname : #argtype,
                });
                args_without_types.extend(q! {
                    #argname,
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
            let mut maybe_second_unwrap = q!{};
            let returntype = if let Some(rt) = &method.ret {
                maybe_second_unwrap = q!{.unwrap()};
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
                    fn #method_name(#slf, #args_with_types ) #returntype {
                        #resultified_trait_name::#rt_method_name(#slf2, #args_without_types).unwrap() #maybe_second_unwrap
                    }
                });
            } else {
                let literal1 = proc_macro2::Literal::string(&self.name.to_string());
                let literal2 = proc_macro2::Literal::string(&method.name.to_string());
                let literal3 = proc_macro2::Literal::string(&proxy_name.to_string());
                methods.extend(q! {
                    fn #method_name(#slf, #args_with_types ) #returntype {
                        panic!("Cannot call {}::{} accepting too weak `self` on {}", #literal1, #literal2, #literal3)
                    }
                });
            }
        }
        let maybe_additional_where_clause = if let Some(returval_macro) = returnval_handler {
            q!{,#returval_macro ! (RecvError) : ::std::fmt::Debug}
        } else {
            q!{}
        };
        out.extend(q! {
            impl<E, F: #fn_trait(#enum_name) -> ::std::result::Result<(), E>>  #name for #proxy_name<E,F> where E : ::std::fmt::Debug #maybe_additional_where_clause {
                #methods
            }
        });
    }
}
