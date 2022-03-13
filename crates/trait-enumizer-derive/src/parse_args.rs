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
    let mut proxies = vec![];
    let mut call_fns = vec![];
    let mut access_mode = AccessMode::Priv;
    let mut returnval = None;
    let mut enum_attr = vec![];
    let mut enum_name = None;
    let mut inherent_impl_mode = false;

    let mut state = ParserState::<RootLevelIdentAssignmentTargets,RootLevelGroupAssignmentTargets>::ExpectingNewParam;

    use ParserState::*;
    use RootLevelGroupAssignmentTargets::*;
    use RootLevelIdentAssignmentTargets::*;

    for x in input {
        match state {
            ExpectingNewParam => match x {
                TokenTree::Ident(y) => match y.to_string().as_str() {
                    "pub" => access_mode = AccessMode::Pub,
                    "pub_crate" => access_mode = AccessMode::PubCrate,
                    "priv" => access_mode = AccessMode::Priv,
                    "returnval" => state = ExpectingEqsign(Returnval),
                    "call_fn" => state = ExpectingGroup(CallFn),
                    "proxy" => state = ExpectingGroup(Proxy),
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
                        CustomAttr => enum_attr.push(y),
                        CallFn => call_fns.push(parse_call_fn(y.stream())),
                        Proxy => proxies.push(parse_proxy(y.stream())),
                    },
                    _ => panic!("Expected a group after parameter for {:?}", t),
                }
                state = ExpectingNewParam
            }
        }
    }

    let enum_name = enum_name.expect("`name` parameter is required.");

    Params {
        proxies,
        call_fns,
        access_mode,
        returnval,
        enum_attr,
        enum_name,
        inherent_impl_mode,
    }
}

#[derive(Debug, Clone, Copy)]
enum CallFnIdentAssignmentTargets {
    Name,
}
#[derive(Debug, Clone, Copy)]
enum CallFnGroupAssignmentTargets {
    ExtraArgType,
}

fn parse_call_fn(input: TokenStream) -> CallFnParams {
    let mut level = None;
    let mut allow_panic = false;
    let mut extra_arg = None;
    let mut name = None;
    let mut r#async = false;

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
                    "async" => r#async = true,
                    "no_async" => r#async = false,
                    "extra_arg_type" => state = ExpectingGroup(ExtraArgType),
                    "ref" => level = Some(ReceiverStyle::Ref),
                    "ref_mut" | "mut_ref" => level = Some(ReceiverStyle::Mut),
                    "once" | "move" => level = Some(ReceiverStyle::Move),
                    "Fn" | "FnMut" | "FnOnce" => {
                        panic!("Use ref/ref_mut/once for call_fn, not Fn*")
                    }
                    "name" => state=ExpectingEqsign(Name),
                    z => panic!("Unknown subparameter {}", z),
                },
                TokenTree::Punct(y) if y.as_char() == ',' => (),
                _ => panic!("Expecting some call_fn subparameter, got {:?}", x),
            },
            ExpectingIdent(t) => {
                match x {
                    TokenTree::Ident(y) => match t {
                        Name => name = Some(y),
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
                        ExtraArgType => extra_arg = Some(y.stream()),
                    },
                    _ => panic!("Expected a group after parameter for {:?}", t),
                }
                state = ExpectingNewParam;
            }
        }
    }

    let level = level.expect("Set one of `ref`, `ref_mut` or `once` subparameters");
    let name = name.expect("`name` subparameter is required.");

    CallFnParams {
        level,
        name,
        allow_panic,
        extra_arg,
        r#async,
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

fn parse_proxy(input: TokenStream) -> GenProxyParams {
    let mut gen_infallible = false;
    let mut gen_unwrapping = false;
    let mut gen_unwrapping_and_panicking = false;
    let mut extra_arg = None;
    let mut name = None;
    let mut traitname = None;
    let mut level = None;
    let mut r#async = false;

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
                    "Fn" => level = Some(ReceiverStyle::Ref),
                    "FnMut" => level = Some(ReceiverStyle::Mut),
                    "FnOnce" => level = Some(ReceiverStyle::Move),
                    "ref" | "ref_mut" | "mut_ref" | "once" | "move" => panic!(
                        "Maybe you meant Fn/FnMut/FnOnce for proxy subparam, not ref/once/move/mut"
                    ),
                    "extra_field_type" => state = ExpectingGroup(ExtraFieldType),
                    "name" => state = ExpectingEqsign(Name),
                    "resultified_trait" => state = ExpectingEqsign(TraitName),
                    "async" => r#async = true,
                    "no_async" => r#async = false,
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
    if gen_infallible {
        ctr += 1;
    }
    if gen_unwrapping {
        ctr += 1;
    }
    if gen_unwrapping_and_panicking {
        ctr += 1;
    }
    if ctr > 1 {
        panic!("Choose only one of infallible or unwrapping impl")
    }
    if r#async && ctr > 0 {
        panic!("async is incompatible with any impls");
    }

    let name = name.expect("`name` subparameter is required.");
    let level = level.expect("Set one of `Fn`, `FnMut` or `FnOnce` subparameters");

    GenProxyParams {
        level,
        gen_infallible,
        gen_unwrapping,
        gen_unwrapping_and_panicking,
        extra_arg,
        name,
        traitname,
        r#async,
    }
}

#[test]
fn test_parser1() {
    let attrs = parse_args(quote::quote! {name=Qqq});
    assert_eq!(attrs.access_mode, AccessMode::Priv);
    assert!(attrs.call_fns.is_empty());
    assert!(attrs.proxies.is_empty());
    assert!(attrs.enum_attr.is_empty());
    assert!(attrs.returnval.is_none());
}

#[test]
fn test_parser2() {
    let attrs = parse_args(quote::quote! {
        returnval=my_rpc_class,
        name=MyEnum,
        call_fn(ref,name=call,extra_arg_type(i32)),
        call_fn(ref_mut,name=call_mut,extra_arg_type(&flume::Sender<String>)),
        call_fn(once,name=call_once,allow_panic),
        proxy(Fn,name=MyProxy,unwrapping_impl,extra_field_type(MyRpcClient)),
        proxy(FnMut,name=MyMutProxy,infallible_impl),
        proxy(FnOnce,name=MyOnceProxy,resultified_trait=Qqq,unwrapping_and_panicking_impl),
        enum_attr[derive(serde_derive::Serialize,serde_derive::Deserialize)],
        enum_attr[222]
    });
    assert_eq!(attrs.access_mode, AccessMode::Priv);
    assert_eq!(attrs.call_fns[0].allow_panic, false);
    assert_eq!(attrs.call_fns[1].allow_panic, false);
    assert_eq!(attrs.call_fns[2].allow_panic, true);

    assert!(attrs.call_fns[0].extra_arg.is_some());
    assert!(attrs.call_fns[1].extra_arg.is_some());
    assert!(attrs.call_fns[2].extra_arg.is_none());

    assert_eq!(attrs.call_fns[0].level, ReceiverStyle::Ref);
    assert_eq!(attrs.call_fns[1].level, ReceiverStyle::Mut);
    assert_eq!(attrs.call_fns[2].level, ReceiverStyle::Move);

    assert_eq!(attrs.proxies[0].gen_unwrapping, true);
    assert_eq!(attrs.proxies[0].gen_infallible, false);
    assert_eq!(attrs.proxies[0].gen_unwrapping_and_panicking, false);

    assert_eq!(attrs.proxies[1].gen_unwrapping, false);
    assert_eq!(attrs.proxies[1].gen_infallible, true);
    assert_eq!(attrs.proxies[1].gen_unwrapping_and_panicking, false);

    assert_eq!(attrs.proxies[2].gen_unwrapping, false);
    assert_eq!(attrs.proxies[2].gen_infallible, false);
    assert_eq!(attrs.proxies[2].gen_unwrapping_and_panicking, true);

    assert_eq!(attrs.proxies[0].level, ReceiverStyle::Ref);
    assert_eq!(attrs.proxies[1].level, ReceiverStyle::Mut);
    assert_eq!(attrs.proxies[2].level, ReceiverStyle::Move);

    assert_eq!(attrs.enum_attr.len(), 2);
    assert_eq!(attrs.returnval.unwrap().to_string(), "my_rpc_class");
}
