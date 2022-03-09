use proc_macro2::TokenStream;

use crate::AccessMode;
use crate::CallFnParams;

use super::GenProxyParams;

use super::Params;

pub(crate) fn parse_args(attrs: TokenStream) -> Params {
    let mut params = Params::default();
    let mut current_genproxy: Option<&mut GenProxyParams> = None;
    let mut current_callfn: Option<&mut CallFnParams> = None;
    let mut custom_attr_pending = false;
    let mut returnval_eqsign_pending = false;
    let mut returnval_ident_pending = false;
    for x in attrs {
        match x {
            proc_macro2::TokenTree::Group(g) => {
                if returnval_eqsign_pending {
                    panic!("returnval should be followed by `=` character");
                }
                if returnval_ident_pending {
                    panic!("returnval= should be followed by ident");
                }
                if custom_attr_pending {
                    params.enum_attr.push(g);
                    custom_attr_pending = false;
                    continue;
                }
                if g.delimiter() != proc_macro2::Delimiter::Parenthesis {
                    panic!("Invalid input to `enumizer` attribute macro - non-round parentheses")
                }
                let mut claimed = false;
                if let Some(cgp) = current_genproxy.take() {
                    claimed = true;
                    let mut ctr = 0;
                    let mut extra_arg_expecting_value = false;
                    for xx in g.stream() {
                        if extra_arg_expecting_value {
                            match xx {
                                proc_macro2::TokenTree::Group(g) => {
                                    if g.delimiter() != proc_macro2::Delimiter::Parenthesis {
                                        panic!("extra_field_type must be followed by ()");
                                    }
                                    cgp.extra_arg = Some(g.stream());
                                    extra_arg_expecting_value = false;
                                    continue;
                                }
                                _ => {
                                    panic!("extra_field_type must be followed by ()");
                                }
                            }
                        }
                        match xx {
                            proc_macro2::TokenTree::Group(_) => {
                                panic!("Invalid input to `enumizer` attribute macro - no groups expected here")
                            }
                            proc_macro2::TokenTree::Ident(x) => match &*x.to_string() {
                                "infallible_impl" => {
                                    if cgp.gen_infallible {
                                        panic!("Duplicate `infallible_impl`");
                                    }
                                    cgp.gen_infallible = true;
                                    ctr += 1;
                                }
                                "unwrapping_impl" => {
                                    if cgp.gen_unwrapping {
                                        panic!("Duplicate `unwrapping_impl`");
                                    }
                                    cgp.gen_unwrapping = true;
                                    ctr += 1;
                                }
                                "unwrapping_and_panicking_impl" => {
                                    if cgp.gen_unwrapping_and_panicking {
                                        panic!("Duplicate `unwrapping_impl`");
                                    }
                                    cgp.gen_unwrapping_and_panicking = true;
                                    ctr += 1;
                                }
                                "extra_field_type" => {
                                    if cgp.extra_arg.is_some() {
                                        panic!("Duplicate `extra_field_type`. Use a tuple if you want to pass multiple values.");
                                    }
                                    extra_arg_expecting_value = true;
                                }
                                t => panic!("This suboption (`{}`) is not supported", t),
                            },
                            proc_macro2::TokenTree::Punct(p) => {
                                if p.as_char() == ',' || p.as_char() == '\n'  {
                                    // OK, ignoring it
                                } else {
                                    panic!("Invalid input to `enumizer` attribute macro - the only punctuation accepted is `,`");
                                }
                            }
                            proc_macro2::TokenTree::Literal(_) => {
                                panic!("Invalid input to `enumizer` attribute macro - no literals accepted here")
                            }
                        }
                    }

                    if extra_arg_expecting_value {
                        panic!("Unfinished `extra_field_type`")
                    }

                    if ctr > 1 {
                        panic!("Choose only one of infallible, unwrapping or unwrapping-and-panicking impl");
                    }
                }

                if let Some(cfp) = current_callfn.take() {
                    claimed = true;
                    let mut extra_arg_expecting_value = false;
                    for xx in g.stream() {
                        if extra_arg_expecting_value {
                            match xx {
                                proc_macro2::TokenTree::Group(g) => {
                                    if g.delimiter() != proc_macro2::Delimiter::Parenthesis {
                                        panic!("extra_arg_type must be followed by ()");
                                    }
                                    cfp.extra_arg = Some(g.stream());
                                    extra_arg_expecting_value = false;
                                    continue;
                                }
                                _ => {
                                    panic!("extra_arg_type must be followed by ()");
                                }
                            }
                        }
                        match xx {
                            proc_macro2::TokenTree::Group(_) => {
                                panic!("Invalid input to `enumizer` attribute macro - no groups in callfn params")
                            }
                            proc_macro2::TokenTree::Ident(x) => {
                                match &*x.to_string() {
                                    "allow_panic" => {
                                        if cfp.allow_panic {
                                            panic!("Duplicate `allow_panic`");
                                        }
                                        cfp.allow_panic = true;
                                    }
                                    "extra_arg_type" => {
                                        if cfp.extra_arg.is_some() {
                                            panic!("Duplicate `extra_arg_type`. Use a tuple if you want to pass multiple values.");
                                        }
                                        extra_arg_expecting_value = true;
                                    }
                                    t => panic!("This suboption (`{}`) is not supported", t),
                                }
                            }
                            proc_macro2::TokenTree::Punct(p) => {
                                if p.as_char() == ',' || p.as_char() == '\n' {
                                    // OK, ignoring it
                                } else {
                                    panic!("Invalid input to `enumizer` attribute macro - non-`,` punct in callfn params");
                                }
                            }
                            proc_macro2::TokenTree::Literal(_) => {
                                panic!("Invalid input to `enumizer` attribute macro - literal unexpected in callnf params")
                            }
                        }
                    }
                    if  extra_arg_expecting_value {
                        panic!("Unfinished `extra_arg_type(...)` subargument");
                    }
                }
                if !claimed {
                    panic!("Invalid input to `enumizer` attribute macro - unexpected group")
                }
            }
            proc_macro2::TokenTree::Ident(x) => {
                if returnval_eqsign_pending {
                    panic!("returnval should be followed by `=` character");
                }
                if returnval_ident_pending {
                    params.returnval = Some(x.clone());
                    returnval_ident_pending = false;
                    continue;
                }
                if custom_attr_pending {
                    panic!("custom_attr should be followed by a group");
                }
                match &*x.to_string() {
                    "ref_proxy" => {
                        if params.ref_proxy.is_some() {
                            panic!("Duplicate `ref_proxy`");
                        }
                        params.ref_proxy = Some(GenProxyParams::default());
                        current_genproxy = params.ref_proxy.as_mut();
                    }
                    "mut_proxy" => {
                        if params.mut_proxy.is_some() {
                            panic!("Duplicate `ref_proxy`");
                        }
                        params.mut_proxy = Some(GenProxyParams::default());
                        current_genproxy = params.mut_proxy.as_mut();
                    }
                    "once_proxy" => {
                        if params.once_proxy.is_some() {
                            panic!("Duplicate `ref_proxy`");
                        }
                        params.once_proxy = Some(GenProxyParams::default());
                        current_genproxy = params.once_proxy.as_mut();
                    }
                    "call" => {
                        if params.call_ref.is_some() {
                            panic!("Duplicate `call`");
                        }
                        params.call_ref = Some(CallFnParams::default());
                        current_callfn = params.call_ref.as_mut();
                    }
                    "call_mut" => {
                        if params.call_mut.is_some() {
                            panic!("Duplicate `call`");
                        }
                        params.call_mut = Some(CallFnParams::default());
                        current_callfn = params.call_mut.as_mut();
                    }
                    "call_once" => {
                        if params.call_once.is_some() {
                            panic!("Duplicate `call`");
                        }
                        params.call_once = Some(CallFnParams::default());
                        current_callfn = params.call_once.as_mut();
                    }
                    "pub" => {
                        if params.access_mode != AccessMode::Priv {
                            panic!("Duplicate `pub` or `pub_crate`");
                        }
                        params.access_mode = AccessMode::Pub;
                    }
                    "pub_crate" => {
                        if params.access_mode != AccessMode::Priv {
                            panic!("Duplicate `pub` or `pub_crate`");
                        }
                        params.access_mode = AccessMode::PubCrate;
                    }
                    "returnval" => {
                        if params.returnval.is_some() {
                            panic!("Duplicate `returnval`");
                        }
                        returnval_eqsign_pending = true;
                    }
                    "enum_attr" => {
                        custom_attr_pending = true;
                    }
                    t => panic!("This option (`{}`) is not supported", t),
                }
            }
            proc_macro2::TokenTree::Punct(x) => {
                if returnval_ident_pending {
                    panic!("returnval= should be followed by ident");
                }
                if returnval_eqsign_pending {
                    if x.as_char() != '=' {
                        panic!("returnval should be followed by `=` character");
                    }
                    returnval_eqsign_pending = false;
                    returnval_ident_pending = true;
                    continue;
                }
                if custom_attr_pending {
                    panic!("custom_attr should be followed by a group");
                }
                if x.as_char() == ',' || x.as_char() == '\n'  {
                    current_callfn = None;
                    current_genproxy = None;
                } else {
                    panic!("Invalid input to `enumizer` attribute macro - non-comma punct");
                }
            }
            proc_macro2::TokenTree::Literal(_) => {
                panic!("Invalid input to `enumizer` attribute macro - no literals expected")
            }
        }
    }
    if returnval_ident_pending || returnval_eqsign_pending {
        panic!("Unfinished `returnval=...` argument");
    }
    params
}
