#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

#![deny(missing_docs)]
#![no_std]

#[cfg(feature="std")]
extern crate std;


/// Main item of this crate. See crate-level doc (or, equivalently, README) for details
pub use trait_enumizer_derive::enumizer;

mod returnval;

#[doc(inline)]
pub use returnval::*;
