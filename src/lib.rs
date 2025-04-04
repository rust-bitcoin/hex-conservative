// SPDX-License-Identifier: CC0-1.0

//! Hex encoding and decoding.
//!
//! General purpose hex encoding/decoding library with a conservative MSRV and dependency policy.
//!
//! ## Basic Usage
//! ```
//! # #[cfg(feature = "alloc")] {
//! // In your manifest use the `package` key to improve import ergonomics.
//! // hex = { package = "hex-conservative", version = "*" }
//! # use hex_conservative as hex; // No need for this if using `package` as above.
//! use hex::prelude::*;
//!
//! // Decode an arbitrary length hex string into a vector.
//! let v = Vec::from_hex("deadbeef").expect("valid hex digits");
//! // Or a known length hex string into a fixed size array.
//! let a = <[u8; 4]>::from_hex("deadbeef").expect("valid length and valid hex digits");
//!
//! // We support `LowerHex` and `UpperHex` out of the box for `[u8]` slices.
//! println!("An array as lower hex: {:x}", a.as_hex());
//! // And for vecs since `Vec` derefs to byte slice.
//! println!("A vector as upper hex: {:X}", v.as_hex());
//!
//! // Allocate a new string (also `to_upper_hex_string`).
//! let s = v.to_lower_hex_string();
//!
//! // Please note, mixed case strings will still parse successfully but we only
//! // support displaying hex in a single case.
//! assert_eq!(
//!     Vec::from_hex("dEaDbEeF").expect("valid mixed case hex digits"),
//!     Vec::from_hex("deadbeef").expect("valid hex digits"),
//! );
//! # }
//! ```
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
//! ## Crate feature flags
//!
//! * `std` - enables the standard library, on by default.
//! * `alloc` - enables features that require allocation such as decoding into `Vec<u8>`, implied
//!   by `std`.
//! * `newer-rust-version` - enables Rust version detection and thus newer features, may add
//!   dependency on a feature detection crate to reduce compile times. This feature is expected to
//!   do nothing once the native detection is in Rust and our MSRV is at least that version. We may
//!   also remove the feature gate in 2.0 with semver trick once that happens.
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
#[allow(unused_imports)] // false positive regarding macro
#[macro_use]
extern crate alloc;

#[doc(hidden)]
pub mod _export {
    /// A re-export of `core::*`.
    pub mod _core {
        pub use core::*;
    }
}

pub mod buf_encoder;
pub mod display;
pub mod error;
mod iter;
pub mod parse;
#[cfg(feature = "serde")]
pub mod serde;

/// Re-exports of the common crate traits.
pub mod prelude {
    #[doc(inline)]
    pub use crate::{display::DisplayHex, parse::FromHex};
}

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

pub(crate) use table::Table;

#[rustfmt::skip]                // Keep public re-exports separate.
#[doc(inline)]
pub use self::{
    display::DisplayHex,
    error::{
        DecodeFixedSizedBytesError, DecodeDynSizedBytesError, InvalidCharError, InvalidLengthError,
        OddLengthStringError,
    },
    iter::{BytesToHexIter, HexToBytesIter, HexSliceToBytesIter},
    parse::FromHex,
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
pub fn decode_to_vec(hex: &str) -> Result<Vec<u8>, DecodeDynSizedBytesError> {
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
pub fn decode_to_array<const N: usize>(hex: &str) -> Result<[u8; N], DecodeFixedSizedBytesError> {
    if hex.len() == N * 2 {
        let mut ret = [0u8; N];
        // checked above
        HexToBytesIter::new_unchecked(hex).drain_to_slice(&mut ret)?;
        Ok(ret)
    } else {
        Err(InvalidLengthError { invalid: hex.len(), expected: 2 * N }.into())
    }
}

/// Possible case of hex.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Case {
    /// Produce lower-case chars (`[0-9a-f]`).
    ///
    /// This is the default.
    Lower,

    /// Produce upper-case chars (`[0-9A-F]`).
    Upper,
}

impl Default for Case {
    #[inline]
    fn default() -> Self { Case::Lower }
}

impl Case {
    /// Returns the encoding table.
    ///
    /// The returned table may only contain displayable ASCII chars.
    #[inline]
    #[rustfmt::skip]
    pub(crate) fn table(self) -> &'static Table {
        match self {
            Case::Lower => &Table::LOWER,
            Case::Upper => &Table::UPPER,
        }
    }
}

/// Correctness boundary for `Table`.
mod table {
    /// Table of hex chars.
    //
    // Correctness invariant: each byte in the table must be ASCII.
    #[derive(Debug)]
    pub(crate) struct Table([u8; 16]);

    impl Table {
        pub(crate) const LOWER: Self = Table([
            b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'a', b'b', b'c', b'd',
            b'e', b'f',
        ]);
        pub(crate) const UPPER: Self = Table([
            b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'A', b'B', b'C', b'D',
            b'E', b'F',
        ]);

        /// Encodes single byte as two ASCII chars using the given table.
        ///
        /// The function guarantees only returning values from the provided table.
        #[inline]
        pub(crate) fn byte_to_chars(&self, byte: u8) -> [char; 2] {
            let left = self.0[usize::from(byte >> 4)];
            let right = self.0[usize::from(byte & 0x0F)];
            [char::from(left), char::from(right)]
        }

        /// Writes the single byte as two ASCII chars in the provided buffer, and returns a `&str`
        /// to that buffer.
        ///
        /// The function guarantees only returning values from the provided table.
        #[inline]
        pub(crate) fn byte_to_str<'a>(&self, dest: &'a mut [u8; 2], byte: u8) -> &'a str {
            dest[0] = self.0[usize::from(byte >> 4)];
            dest[1] = self.0[usize::from(byte & 0x0F)];
            // SAFETY: Table inner array contains only valid ascii
            let hex_str = unsafe { core::str::from_utf8_unchecked(dest) };
            hex_str
        }
    }
}

/// Quick and dirty macro for parsing hex in tests.
///
/// For improved ergonomics import with: `use hex_conservative::test_hex_unwrap as hex;`
#[macro_export]
#[deprecated(since = "TBD", note = "use the one-liner `Vec::from_hex(hex).unwrap()` instead")]
#[cfg(feature = "alloc")]
macro_rules! test_hex_unwrap (($hex:expr) => (<Vec<u8> as $crate::FromHex>::from_hex($hex).unwrap()));

#[cfg(test)]
#[cfg(feature = "alloc")]
mod tests {
    use alloc::vec::Vec;

    use crate::test_hex_unwrap as hex;

    #[test]
    fn parse_hex_into_vector() {
        let got = hex!("deadbeef");
        let want = vec![0xde, 0xad, 0xbe, 0xef];
        assert_eq!(got, want);
    }
}
