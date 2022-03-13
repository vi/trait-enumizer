use proc_macro2::TokenStream;
use proc_macro2::TokenTree;

use crate::AccessMode;
use crate::CallFnParams;
use crate::ReceiverStyle;

use super::GenProxyParams;

use super::Params;
enum ParserState<I, G> {
    ExpectingNewParam,
    ExpectingIdent(I),
    ExpectingEqsign(I),
    ExpectingGroup(G),
}

#[derive(Debug, Clone, Copy)]
enum RootLevelIdentAssignmentTargets {
    Returnval,
    Name,
}
#[derive(Debug, Clone, Copy)]
enum RootLevelGroupAssignmentTargets {
    CallFn,
    CustomAttr,
    Proxy,
}

pub(crate) fn parse_args(input: TokenStream) -> Params {
    let mut ref_proxy = None;
    let mut mut_proxy = None;
    let mut once_proxy = None;
    let mut call_ref = None;
    let mut call_mut = None;
    let mut call_once = None;
    let mut access_mode = AccessMode::Priv;
    let mut returnval = None;
    let mut enum_attr = vec![];
    let mut enum_name = None;
    let mut inherent_impl_mode = false;

    let mut state = ParserState::<RootLevelIdentAssignmentTargets,RootLevelGroupAssignmentTargets>::ExpectingNewParam;

    use ParserState::*;
    use RootLevelGroupAssignmentTargets::*;
    use RootLevelIdentAssignmentTargets::*;

    let mut cached_level = ReceiverStyle::Ref;

    for x in input {
        match state {
            ExpectingNewParam => match x {
                TokenTree::Ident(y) => match y.to_string().as_str() {
                    "pub" => access_mode = AccessMode::Pub,
                    "pub_crate" => access_mode = AccessMode::PubCrate,
                    "priv" => access_mode = AccessMode::Priv,
                    "returnval" => state = ExpectingEqsign(Returnval),
                    "call" => {
                        cached_level = ReceiverStyle::Ref;
                        state = ExpectingGroup(CallFn)
                    }
                    "call_mut" => {
                        cached_level = ReceiverStyle::Mut;
                        state = ExpectingGroup(CallFn)
                    }
                    "call_once" => {
                        cached_level = ReceiverStyle::Move;
                        state = ExpectingGroup(CallFn)
                    }
                    "ref_proxy" => {
                        cached_level = ReceiverStyle::Ref;
                        state = ExpectingGroup(Proxy)
                    }
                    "mut_proxy" => {
                        cached_level = ReceiverStyle::Mut;
                        state = ExpectingGroup(Proxy)
                    }
                    "once_proxy" => {
                        cached_level = ReceiverStyle::Move;
                        state = ExpectingGroup(Proxy)
                    }
                    "enum_attr" => state = ExpectingGroup(CustomAttr),
                    "name" => state = ExpectingEqsign(Name),
                    "inherent_impl" => inherent_impl_mode = true,
                    z => panic!("Unknown parameter {}", z),
                },
                TokenTree::Group(_) => panic!("No group is expected here"),
                TokenTree::Punct(y) if y.as_char() == ',' => (),
                TokenTree::Punct(_) => panic!("No punctuation is expected here"),
                TokenTree::Literal(_) => panic!("No literal is expected here"),
            },
            ExpectingIdent(t) => {
                match x {
                    TokenTree::Ident(y) => match t {
                        Returnval => returnval = Some(y),
                        Name => enum_name = Some(y),
                    },
                    _ => panic!(
                        "Single identifier is expected in {:?} state after `=` sign",
                        t
                    ),
                }
                state = ExpectingNewParam;
            }
            ExpectingEqsign(t) => match x {
                TokenTree::Punct(y) if y.as_char() == '=' => state = ExpectingIdent(t),
                _ => panic!("Expected `=` character after parameter for {:?}", t),
            },
            ExpectingGroup(t) => {
                match x {
                    TokenTree::Group(y) => match t {
                        CallFn => match cached_level {
                            ReceiverStyle::Move => {
                                call_once = Some(parse_call_fn(y.stream(), cached_level))
                            }
                            ReceiverStyle::Mut => {
                                call_mut = Some(parse_call_fn(y.stream(), cached_level))
                            }
                            ReceiverStyle::Ref => {
                                call_ref = Some(parse_call_fn(y.stream(), cached_level))
                            }
                        },
                        CustomAttr => enum_attr.push(y),
                        Proxy => match cached_level {
                            ReceiverStyle::Move => {
                                once_proxy = Some(parse_proxy(y.stream(), cached_level))
                            }
                            ReceiverStyle::Mut => {
                                mut_proxy = Some(parse_proxy(y.stream(), cached_level))
                            }
                            ReceiverStyle::Ref => {
                                ref_proxy = Some(parse_proxy(y.stream(), cached_level))
                            }
                        },
                    },
                    _ => panic!("Expected a group after parameter for {:?}", t),
                }
                state = ExpectingNewParam
            }
        }
    }

    Params {
        ref_proxy,
        mut_proxy,
        once_proxy,
        call_ref,
        call_mut,
        call_once,
        access_mode,
        returnval,
        enum_attr,
        enum_name,
        inherent_impl_mode,
    }
}

#[derive(Debug, Clone, Copy)]
enum CallFnIdentAssignmentTargets {}
#[derive(Debug, Clone, Copy)]
enum CallFnGroupAssignmentTargets {
    ExtraArgType,
}

#[allow(unused)]
fn parse_call_fn(input: TokenStream, level: ReceiverStyle) -> CallFnParams {
    let mut allow_panic = false;
    let mut extra_arg = None;

    let mut state = ParserState::<CallFnIdentAssignmentTargets,CallFnGroupAssignmentTargets>::ExpectingNewParam;

    use CallFnGroupAssignmentTargets::*;
    use CallFnIdentAssignmentTargets::*;
    use ParserState::*;

    for x in input {
        match state {
            ExpectingNewParam => match x {
                TokenTree::Ident(y) => match y.to_string().as_str() {
                    "allow_panic" => allow_panic = true,
                    "deny_panic" => allow_panic = false,
                    "extra_arg_type" => state = ExpectingGroup(ExtraArgType),
                    z => panic!("Unknown subparameter {}", z),
                },
                TokenTree::Punct(y) if y.as_char() == ',' => (),
                _ => panic!("Expecting some call_fn subparameter, got {:?}", x),
            },
            ExpectingIdent(t) => {
                match x {
                    TokenTree::Ident(y) => match t {},
                    _ => panic!(
                        "Single identifier is expected in {:?} state after `=` sign",
                        t
                    ),
                }
                state = ExpectingNewParam;
            }
            ExpectingEqsign(t) => match x {
                TokenTree::Punct(y) if y.as_char() == '=' => state = ExpectingIdent(t),
                _ => panic!("Expected `=` character after parameter for {:?}", t),
            },
            ExpectingGroup(t) => {
                match x {
                    TokenTree::Group(y) => match t {
                        ExtraArgType => extra_arg = Some(y.stream()),
                    },
                    _ => panic!("Expected a group after parameter for {:?}", t),
                }
                state = ExpectingNewParam;
            }
        }
    }

    CallFnParams {
        level,
        allow_panic,
        extra_arg,
    }
}

#[derive(Debug, Clone, Copy)]
enum ProxyIdentAssignmentTargets {
    Name,
    TraitName,
}
#[derive(Debug, Clone, Copy)]
enum ProxyGroupAssignmentTargets {
    ExtraFieldType,
}

fn parse_proxy(input: TokenStream, level: ReceiverStyle) -> GenProxyParams {
    let mut gen_infallible = false;
    let mut gen_unwrapping = false;
    let mut gen_unwrapping_and_panicking = false;
    let mut extra_arg = None;
    let mut name = None;
    let mut traitname = None;

    let mut state =
        ParserState::<ProxyIdentAssignmentTargets, ProxyGroupAssignmentTargets>::ExpectingNewParam;

    use ParserState::*;
    use ProxyGroupAssignmentTargets::*;
    use ProxyIdentAssignmentTargets::*;

    for x in input {
        match state {
            ExpectingNewParam => match x {
                TokenTree::Ident(y) => match y.to_string().as_str() {
                    "infallible_impl" => gen_infallible = true,
                    "unwrapping_impl" => gen_unwrapping = true,
                    "unwrapping_and_panicking_impl" => gen_unwrapping_and_panicking = true,
                    "extra_field_type" => state = ExpectingGroup(ExtraFieldType),
                    "name" => state = ExpectingEqsign(Name),
                    "traitname" => state = ExpectingEqsign(TraitName),
                    z => panic!("Unknown subparameter {}", z),
                },
                TokenTree::Punct(y) if y.as_char() == ',' => (),
                _ => panic!("Expecting some proxy subparameter, got {:?}", x),
            },
            ExpectingIdent(t) => {
                match x {
                    TokenTree::Ident(y) => match t {
                        Name => name = Some(y),
                        TraitName => traitname = Some(y),
                    },
                    _ => panic!(
                        "Single identifier is expected in {:?} state after `=` sign",
                        t
                    ),
                }
                state = ExpectingNewParam;
            }
            ExpectingEqsign(t) => match x {
                TokenTree::Punct(y) if y.as_char() == '=' => state = ExpectingIdent(t),
                _ => panic!("Expected `=` character after parameter for {:?}", t),
            },
            ExpectingGroup(t) => {
                match x {
                    TokenTree::Group(y) => match t {
                        ExtraFieldType => extra_arg = Some(y.stream()),
                    },
                    _ => panic!("Expected a group after parameter for {:?}", t),
                }
                state = ExpectingNewParam;
            }
        }
    }

    let mut ctr = 0;
    if gen_infallible { ctr += 1; }
    if gen_unwrapping { ctr += 1; }
    if gen_unwrapping_and_panicking { ctr += 1; }
    if ctr > 1 {
        panic!("Choose only one of infallible or unwrapping impl")
    }

    GenProxyParams {
        level,
        gen_infallible,
        gen_unwrapping,
        gen_unwrapping_and_panicking,
        extra_arg,
        name,
        traitname,
    }
}

#[test]
fn test_parser1() {
    let attrs = parse_args(quote::quote! {});
    assert_eq!(attrs.access_mode, AccessMode::Priv);
    assert!(attrs.call_ref.is_none());
    assert!(attrs.call_mut.is_none());
    assert!(attrs.call_once.is_none());
    assert!(attrs.ref_proxy.is_none());
    assert!(attrs.mut_proxy.is_none());
    assert!(attrs.once_proxy.is_none());
    assert!(attrs.enum_attr.is_empty());
    assert!(attrs.returnval.is_none());
}

#[test]
fn test_parser2() {
    let attrs = parse_args(quote::quote! {
        returnval=my_rpc_class,
        call(extra_arg_type(i32)),
        call_mut(extra_arg_type(&flume::Sender<String>)),
        call_once(allow_panic),
        ref_proxy(unwrapping_impl,extra_field_type(MyRpcClient)),
        mut_proxy(infallible_impl),
        once_proxy(unwrapping_and_panicking_impl),
        enum_attr[derive(serde_derive::Serialize,serde_derive::Deserialize)],
        enum_attr[222]
    });
    assert_eq!(attrs.access_mode, AccessMode::Priv);
    assert_eq!(attrs.call_ref.as_ref().unwrap().allow_panic, false);
    assert_eq!(attrs.call_mut.as_ref().unwrap().allow_panic, false);
    assert_eq!(attrs.call_once.as_ref().unwrap().allow_panic, true);

    assert!(attrs.call_ref.as_ref().unwrap().extra_arg.is_some());
    assert!(attrs.call_mut.as_ref().unwrap().extra_arg.is_some());
    assert!(attrs.call_once.as_ref().unwrap().extra_arg.is_none());

    assert_eq!(attrs.ref_proxy.as_ref().unwrap().gen_unwrapping, true);
    assert_eq!(attrs.ref_proxy.as_ref().unwrap().gen_infallible, false);
    assert_eq!(
        attrs
            .ref_proxy
            .as_ref()
            .unwrap()
            .gen_unwrapping_and_panicking,
        false
    );

    assert_eq!(attrs.mut_proxy.as_ref().unwrap().gen_unwrapping, false);
    assert_eq!(attrs.mut_proxy.as_ref().unwrap().gen_infallible, true);
    assert_eq!(
        attrs
            .mut_proxy
            .as_ref()
            .unwrap()
            .gen_unwrapping_and_panicking,
        false
    );

    assert_eq!(attrs.once_proxy.as_ref().unwrap().gen_unwrapping, false);
    assert_eq!(attrs.once_proxy.as_ref().unwrap().gen_infallible, false);
    assert_eq!(
        attrs
            .once_proxy
            .as_ref()
            .unwrap()
            .gen_unwrapping_and_panicking,
        true
    );

    assert_eq!(attrs.enum_attr.len(), 2);
    assert_eq!(attrs.returnval.unwrap().to_string(), "my_rpc_class");
}
