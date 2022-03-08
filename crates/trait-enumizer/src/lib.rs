#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(feature="returnval", feature(generic_associated_types))]
#![doc = include_str!("../../../README.md")]

pub use trait_enumizer_derive::enumizer;

#[cfg(feature="returnval")]
mod returnval;

#[cfg(feature="returnval")]
#[doc(inline)]
pub use returnval::*;
