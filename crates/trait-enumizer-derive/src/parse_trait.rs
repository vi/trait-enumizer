use proc_macro2::TokenTree;

use crate::{Method, Argument};

use super::{TheTrait, ReceiverStyle};
impl TheTrait {
    pub(crate) fn parse(item: &mut syn::ItemTrait, returnval: bool) -> TheTrait {
        let mut methods = Vec::with_capacity(item.items.len());

        for x in &mut item.items {
            match x {
                syn::TraitItem::Method(m) => {
                    let mut enum_attr = vec![];
                    let mut return_attr = vec![];
                    let sig = &mut m.sig;
                    if sig.constness.is_some() {
                        panic!("Trait-enumizer does not support const");
                    }
                    if sig.asyncness.is_some() {
                        panic!("Trait-enumizer does not support async");
                    }
                    if sig.unsafety.is_some() {
                        panic!("Trait-enumizer does not support unsafe");
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
                    if !returnval && !matches!(sig.output, syn::ReturnType::Default) {
                        panic!("Specify `returnval` parameter to handle methods with return types.")
                    }
                    m.attrs.retain(|a| match a.path.get_ident() {
                        Some(x) if x == "enumizer_enum_attr" || x == "enumizer_return_attr" => {
                            let g = match a.tokens.clone().into_iter().next() {
                                Some(TokenTree::Group(g)) => {
                                    g
                                }
                                _ => panic!("Input of `enumizer_{{enum|return}}_attr` should be single [...] group"),
                            };
                            match x {
                                x if x == "enumizer_enum_attr" => enum_attr.push(g),
                                x if x == "enumizer_return_attr" => return_attr.push(g),
                                _ => unreachable!(),
                            }
                            false
                        }
                        _ => true,
                    });

                    let mut args = Vec::with_capacity(sig.inputs.len());

                    let mut receiver_style = None;

                    let ret = match &sig.output {
                        syn::ReturnType::Default => None,
                        syn::ReturnType::Type(_, t) => Some(*t.clone()),
                    };

                    for inp in &mut sig.inputs {
                        match inp {
                            syn::FnArg::Receiver(r) => {
                                receiver_style = if let Some(rr) = &r.reference {
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
                                let mut enum_attr = vec![];
                                arg.attrs.retain(|a| match a.path.get_ident() {
                                    Some(x) if x == "enumizer_enum_attr" => {
                                        match a.tokens.clone().into_iter().next() {
                                            Some(TokenTree::Group(g)) => {
                                                enum_attr.push(g);
                                            }
                                            _ => panic!("Input of `enumizer_enum_attr` should be a single [...] group"),
                                        }
                                        false
                                    }
                                    _ => true,
                                });
                                match &*arg.pat {
                                    syn::Pat::Ident(pi) => {
                                        if pi.by_ref.is_some() {
                                            panic!("Trait-enumizer does not support `ref` in argument names");
                                        }
                                        if returnval {
                                            if pi.ident.to_string() == "ret" {
                                                panic!("In `returnval` mode, method's arguments cannot be named literally `ret`. Rename it away in `{}`.", sig.ident);
                                            }
                                        }
                                        args.push(Argument { name: pi.ident.clone(), ty: *arg.ty.clone(), enum_attr });
                                    }
                                    _ => panic!("Trait-enumizer does not support method arguments that are patterns, not just simple identifiers"),
                                }
                            }
                        }
                    }

                    if receiver_style.is_none() {
                        panic!("Trait-enumizer does not support methods that do not accept `self`")
                    }

                    let method = Method {
                        args,
                        name: sig.ident.clone(),
                        receiver_style: receiver_style.unwrap(),
                        ret,
                        enum_attr,
                        return_attr,
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
            name: item.ident.clone(),
            methods,
        }
    }
}
