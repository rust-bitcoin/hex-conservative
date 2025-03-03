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
        DecodeToArrayError, DecodeToBytesError, InvalidCharError,
        InvalidLengthError, OddLengthStringError, 
    },
};

/// Decodes a hex string into a vector of bytes.
///
/// # Errors
///
/// Errors if `s` is not a valid hex string.
#[cfg(feature = "alloc")]
pub fn decode_vec(s: &str) -> Result<Vec<u8>, DecodeToBytesError> {
    // This error conversion is because we have a private error associated with `FromHex`.
    use crate::parse::HexToBytesError as E;

    match Vec::from_hex(s) {
        Ok(v) => Ok(v),
        Err(e) => match e {
            E::InvalidChar(e) => Err(DecodeToBytesError::InvalidChar(e)),
            E::OddLengthString(e) => Err(DecodeToBytesError::OddLengthString(e)),
        }
    }
}

/// Decodes a hex string into an array of bytes.
///
/// # Errors
///
/// Errors if `s` is not a valid hex string or the correct length.
pub fn decode_array<const N: usize>(s: &str) -> Result<[u8; N], DecodeToArrayError> {
    // This error conversion is because we have a private error associated with `FromHex`.
    use crate::parse::HexToArrayError as E;

    match <[u8; N]>::from_hex(s) {
        Ok(v) => Ok(v),
        Err(e) => match e {
            E::InvalidChar(e) => Err(DecodeToArrayError::InvalidChar(e)),
            E::InvalidLength(e) => Err(DecodeToArrayError::InvalidLength(e)),
        }
    }
}
