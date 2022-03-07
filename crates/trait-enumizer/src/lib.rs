#![cfg_attr(feature="returnval", feature(generic_associated_types))]

pub use trait_enumizer_derive::enumizer;

#[cfg(feature="returnval")]
mod returnval;

#[cfg(feature="returnval")]
pub use returnval::*;
