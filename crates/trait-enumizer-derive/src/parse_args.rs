use proc_macro2::TokenStream;

use crate::AccessMode;
use crate::CallFnParams;

use super::GenProxyParams;

use super::Params;


pub(crate) fn parse_args(attrs: TokenStream) -> Params {
    let mut params = Params::default();
    let mut current_genproxy : Option<&mut GenProxyParams> = None;
    let mut current_callfn : Option<&mut CallFnParams> = None;
    for x in attrs {
        match x {
            proc_macro2::TokenTree::Group(g) => {
                if g.delimiter() != proc_macro2::Delimiter::Parenthesis {
                    panic!("Invalid input to `enumizer` attribute macro")
                }
                let mut claimed = false;
                if let Some(cgp) = current_genproxy.take() {
                    claimed = true;
                    let mut ctr = 0;
                    for xx in g.stream() {
                        match xx {
                            proc_macro2::TokenTree::Group(_) => {
                                panic!("Invalid input to `enumizer` attribute macro")
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
                                t => panic!("This suboption (`{}`) is not supported", t),
                            },
                            proc_macro2::TokenTree::Punct(p) => {
                                if p.as_char() == ',' {
                                    // OK, ignoring it
                                } else {
                                    panic!("Invalid input to `enumizer` attribute macro");
                                }
                            }
                            proc_macro2::TokenTree::Literal(_) => {
                                panic!("Invalid input to `enumizer` attribute macro")
                            }
                        }
                    }
                    
                    if ctr > 1 {
                        panic!("Choose only one of infallible, unwrapping or unwrapping-and-panicking impl");
                    }
                } 

                if let Some(cfp) = current_callfn.take() {
                    claimed = true;
                    for xx in g.stream() {
                        match xx {
                            proc_macro2::TokenTree::Group(_) => {
                                panic!("Invalid input to `enumizer` attribute macro")
                            }
                            proc_macro2::TokenTree::Ident(x) => match &*x.to_string() {
                                "allow_panic" => {
                                    if cfp.allow_panic {
                                        panic!("Duplicate `allow_panic`");
                                    }
                                    cfp.allow_panic = true;
                                }
                                t => panic!("This suboption (`{}`) is not supported", t),
                            },
                            proc_macro2::TokenTree::Punct(p) => {
                                if p.as_char() == ',' {
                                    // OK, ignoring it
                                } else {
                                    panic!("Invalid input to `enumizer` attribute macro");
                                }
                            }
                            proc_macro2::TokenTree::Literal(_) => {
                                panic!("Invalid input to `enumizer` attribute macro")
                            }
                        }
                    }
                } 
                if ! claimed {
                    panic!("Invalid input to `enumizer` attribute macro - unexpected parentheses")
                }
            }
            proc_macro2::TokenTree::Ident(x) => match &*x.to_string() {
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
                    if params.returnval {
                        panic!("Duplicate `returnval`");
                    }
                    params.returnval = true;
                }
                t => panic!("This option (`{}`) is not supported", t),
            },
            proc_macro2::TokenTree::Punct(x) => {
                if x.as_char() == ',' {
                    current_callfn = None;
                    current_genproxy = None;
                } else {
                    panic!("Invalid input to `enumizer` attribute macro");
                }
            }
            proc_macro2::TokenTree::Literal(_) => {
                panic!("Invalid input to `enumizer` attribute macro")
            }
        }
    }
    params
}
