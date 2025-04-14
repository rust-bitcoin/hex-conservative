// SPDX-License-Identifier: CC0-1.0

//! General purpose hex decoding library with a conservative MSRV and dependency policy.
//!
//! You're currently looking at the stable crate which has advanced features removed to make
//! stabilization quicker and thus allowing downstream crates to stabilize quicker too. To get the
//! full feature set check the lower (0.x.y) versions.
//!
//! ## Stabilization strategy
//!
//! Because downstream crates may need to return hex errors in their APIs and they need to be
//! stabilized soon, this crate only exposes the errors and two basic decoding functions. This
//! should already help with the vast majority of the cases and we're sufficiently confident that
//! these errors won't have a breaking change any time soon (possibly never).
//!
//! If you're writing a binary you don't need to worry about any of this and just use the unstable
//! version for now. If you're writing a library you should use these stable errors in the API but
//! you may internally depend on the unstable crate version to get the advanced features that won't
//! affect your API. This way your API can stabilize before all features in this crate are fully
//! stable and you still can use all of them.
//!
//! ## Crate features
//!
//! * `std` - enables the standard library, on by default.
//! * `alloc` - enables features that require allocation such as decoding into `Vec<u8>`, implied
//! by `std`.
//! * `newer-rust-version` - enables Rust version detection and thus newer features, may add
//!                          dependency on a feature detection crate to reduce compile times. This
//!                          feature is expected to do nothing once the native detection is in Rust
//!                          and our MSRV is at least that version. We may also remove the feature
//!                          gate in 2.0 with semver trick once that happens.
//!
//! ## MSRV policy
//!
//! The MSRV of the crate is currently 1.63.0 and we don't intend to bump it until the newer Rust
//! version is at least two years old and also included in Debian stable (1.63 is in Debian 12 at
//! the moment).
//!
//! Note though that the dependencies may have looser policy. This is not considered breaking/wrong
//! - you would just need to pin them in `Cargo.lock` (not `.toml`).

#![no_std]
// Experimental features we need.
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
// Coding conventions
#![warn(missing_docs)]

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "alloc")]
extern crate alloc;

mod error;
mod iter;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

use crate::iter::HexToBytesIter;

#[rustfmt::skip]                // Keep public re-exports separate.
#[doc(inline)]
pub use self::error::{DecodeFixedSizedBytesError, InvalidCharError, InvalidLengthError};
#[cfg(feature = "alloc")]
pub use self::error::{DecodeDynSizedBytesError, OddLengthStringError};

/// Decodes a hex string with variable length.
///
/// The length of the returned `Vec` is determined by the length of the input, meaning all even
/// lengths of the input string are allowed. If you know the required length at compile time using
/// [`decode_fixed_sized`] is most likely a better choice.
///
/// # Errors
///
/// Errors if `hex` contains invalid characters or doesn't have even length.
#[cfg(feature = "alloc")]
pub fn decode_dyn_sized(hex: &str) -> Result<Vec<u8>, DecodeDynSizedBytesError> {
    Ok(HexToBytesIter::new(hex)?.drain_to_vec()?)
}

/// Decodes a hex string with expected length kown at compile time.
///
/// If you don't know the required length at compile time you need to use [`decode_dyn_sized`]
/// instead.
///
/// # Errors
///
/// Errors if `hex` contains invalid characters or has incorrect length. (Should be `N * 2`.)
pub fn decode_fixed_sized<const N: usize>(hex: &str) -> Result<[u8; N], DecodeFixedSizedBytesError> {
    if hex.len() == N * 2 {
        let mut ret = [0u8; N];
        // checked above
        HexToBytesIter::new_unchecked(hex).drain_to_slice(&mut ret)?;
        Ok(ret)
    } else {
        Err(InvalidLengthError { invalid: hex.len(), expected: 2 * N }.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "alloc")]
    fn hex_error() {
        use crate::error::{InvalidCharError, OddLengthStringError};

        let oddlen = "0123456789abcdef0";
        let badchar1 = "Z123456789abcdef";
        let badchar2 = "012Y456789abcdeb";
        let badchar3 = "«23456789abcdef";

        assert_eq!(
            decode_dyn_sized(oddlen).unwrap_err(),
            OddLengthStringError { len: 17 }.into()
        );
        assert_eq!(
            decode_fixed_sized::<4>(oddlen).unwrap_err(),
            InvalidLengthError { invalid: 17, expected: 8 }.into()
        );
        assert_eq!(
            decode_dyn_sized(badchar1).unwrap_err(),
            InvalidCharError { pos: 0, invalid: b'Z' }.into()
        );
        assert_eq!(
            decode_dyn_sized(badchar2).unwrap_err(),
            InvalidCharError { pos: 3, invalid: b'Y' }.into()
        );
        assert_eq!(
            decode_dyn_sized(badchar3).unwrap_err(),
            InvalidCharError { pos: 0, invalid: 194 }.into()
        );
    }

    #[test]
    fn hex_to_array() {
        let len_sixteen = "0123456789abcdef";
        assert!(decode_fixed_sized::<8>(len_sixteen).is_ok());
    }

    #[test]
    fn hex_to_array_error() {
        let len_sixteen = "0123456789abcdef";
        assert_eq!(
            decode_fixed_sized::<4>(len_sixteen).unwrap_err(),
            InvalidLengthError { invalid: 16, expected: 8 }.into()
        )
    }
}
