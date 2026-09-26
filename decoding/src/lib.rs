// SPDX-License-Identifier: CC0-1.0

#![doc = include_str!("../README.md")]
#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod error;
mod iter;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

#[rustfmt::skip]
#[doc(inline)]
pub use self::iter::{HexSliceToBytesIter, HexToBytesIter};
#[doc(no_inline)]
pub use self::error::{
    DecodeFixedLengthBytesError, DecodeVariableLengthBytesError, InvalidCharError,
    InvalidLengthError, OddLengthStringError,
};

/// Decodes a hex string with variable length.
///
/// The length of the returned `Vec` is determined by the length of the input, meaning all even
/// lengths of the input string are allowed. If you know the required length at compile time using
/// [`decode_to_array`] is most likely a better choice.
///
/// # Errors
///
/// Returns an error if `hex` contains invalid characters or doesn't have even length.
#[cfg(feature = "alloc")]
pub fn decode_to_vec(hex: &str) -> Result<Vec<u8>, DecodeVariableLengthBytesError> {
    Ok(HexToBytesIter::new(hex)?.drain_to_vec()?)
}

/// Decodes a hex string with an expected length known at compile time.
///
/// If you don't know the required length at compile time you need to use [`decode_to_vec`]
/// instead.
///
/// # Errors
///
/// Returns an error if `hex` contains invalid characters or has incorrect length. (Should be
/// `N * 2`.)
pub fn decode_to_array<const N: usize>(hex: &str) -> Result<[u8; N], DecodeFixedLengthBytesError> {
    if hex.len() == N * 2 {
        let mut ret = [0u8; N];
        // checked above
        HexToBytesIter::new_unchecked(hex).drain_to_slice(&mut ret)?;
        Ok(ret)
    } else {
        Err(InvalidLengthError { invalid: hex.len(), expected: 2 * N }.into())
    }
}
