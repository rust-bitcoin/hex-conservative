// SPDX-License-Identifier: CC0-1.0

//! Error types for the [`hex-conservative`] crate.
//!
//! [`hex-conservative`]: <https://crates.io/crates/hex-conservative>

#![no_std]
// Experimental features we need.
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
// Coding conventions.
#![warn(missing_docs)]
#![warn(deprecated_in_future)]
#![doc(test(attr(warn(unused))))]

#[cfg(feature = "alloc")]
extern crate alloc;

extern crate core;

#[cfg(feature = "std")]
extern crate std;

mod error;

#[doc(inline)]
pub use self::error::{
    HexToArrayError, HexToBytesError, InvalidCharError, InvalidLengthError, OddLengthStringError,
    ToArrayError, ToBytesError,
};
