use crate::{Method, Argument};

use super::{TheTrait, ReceiverStyle};
impl TheTrait {
    pub(crate) fn parse(item: syn::ItemTrait) -> TheTrait {
        let mut methods = Vec::with_capacity(item.items.len());

        for x in item.items {
            match x {
                syn::TraitItem::Method(m) => {
                    let sig = m.sig;
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
                                            panic!("Trait-enumizer does not support `ref` in argument names");
                                        }
                                        args.push(Argument { name: pi.ident, ty: *arg.ty });
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
}
