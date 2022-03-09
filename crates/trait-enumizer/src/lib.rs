#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../../../README.md")]

pub use trait_enumizer_derive::enumizer;

mod returnval;

#[doc(inline)]
pub use returnval::*;
