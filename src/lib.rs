// SPDX-License-Identifier: CC0-1.0

//! Hex encoding and decoding.
//!
//! General purpose hex encoding/decoding library with a conservative MSRV and dependency policy.
//!
//! ## Stabalization strategy
//!
//! In an effort to release stable 1.0 crates that are forward compatible we are striving
//! relentlessly to release the bare minimum amount of code. This 1.0 version currently holds
//! only two hex parsing functions and the associated error types.

#![cfg_attr(all(not(test), not(feature = "std")), no_std)]
// Experimental features we need.
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
// Coding conventions
#![warn(missing_docs)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[doc(hidden)]
pub mod _export {
    /// A re-export of core::*
    pub mod _core {
        pub use core::*;
    }
}

mod error;
mod iter;
mod parse;

#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::vec::Vec;

use crate::parse::FromHex as _;

#[rustfmt::skip]                // Keep public re-exports separate.
#[doc(inline)]
pub use self::{
    error::{
        HexToArrayError, HexToBytesError, InvalidCharError, InvalidLengthError,
        OddLengthStringError, ToArrayError, ToBytesError,
    },
};

/// Decodes a hex string into a vector of bytes.
#[cfg(feature = "alloc")]
pub fn decode_vec(s: &str) -> Result<Vec<u8>, HexToBytesError> { Vec::from_hex(s) }

/// Decodes a hex string into an array of bytes.
pub fn decode_array<const N: usize>(s: &str) -> Result<[u8; N], HexToArrayError> {
    <[u8; N]>::from_hex(s)
}
