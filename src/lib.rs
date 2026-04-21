// SPDX-License-Identifier: CC0-1.0

//! # Hex encoding and decoding
//!
//! General purpose hex encoding/decoding library with a conservative MSRV and dependency policy.
//!
//! ## Const hex literals
//!
//! ```
//! use hex_conservative::hex;
//!
//! const GENESIS: [u8; 32] = hex!("000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f");
//! ```
//!
//! ## Runtime hex parsing
//!
//! ```
//! # #[cfg(feature = "alloc")] {
//! // In your manifest use the `package` key to improve import ergonomics.
//! // hex = { package = "hex-conservative", version = "*" }
//! # use hex_conservative as hex; // No need for this if using `package` as above.
//! use hex::prelude::*;
//!
//! // Decode an arbitrary length hex string into a vector.
//! let v = hex::decode_to_vec("deadbeef").expect("valid hex digits");
//! // Or a known length hex string into a fixed size array.
//! let a = hex::decode_to_array::<4>("deadbeef").expect("valid length and valid hex digits");
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
//!     hex::decode_to_vec("dEaDbEeF").expect("valid mixed case hex digits"),
//!     hex::decode_to_vec("deadbeef").expect("valid hex digits"),
//! );
//! # }
//! ```
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
//! ## Minimum Supported Rust Version (MSRV)
//!
//! The current MSRV is Rust `1.74.0`. Policy is to never use an MSRV that is less than two years
//! old and also that ships in Debian stable. We may bump our MSRV in a minor version, but we have
//! no plans to.
//!
//! Note though that the dependencies may have looser policy. This is not considered breaking/wrong
//! - you would just need to pin them in `Cargo.lock` (not `.toml`).

#![no_std]
// Experimental features we need.
#![cfg_attr(docsrs, feature(doc_cfg))]
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

/// Re-exports of the common crate traits.
pub mod prelude {
    #[doc(inline)]
    pub use crate::display::DisplayHex;
}

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

pub(crate) use table::Table;

#[rustfmt::skip]                // Keep public re-exports separate.
#[doc(inline)]
pub use self::{
    display::DisplayHex,
    iter::{BytesToHexIter, HexToBytesIter, HexSliceToBytesIter},
};
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

/// Parses hex strings in const contexts.
///
/// This is primarily useful for testing, panics on all error paths.
///
/// # Returns
///
/// `[u8; N]` array containing the parsed data if valid.
///
/// # Panics
///
/// Panics on all error paths:
///
/// * If input string is not even length.
/// * If input string contains non-hex characters.
#[macro_export]
macro_rules! hex {
    ($hex:expr) => {{
        const _: () = assert!($hex.len() % 2 == 0, "hex string must have even length");

        const fn decode_digit(digit: u8) -> u8 {
            match digit {
                b'0'..=b'9' => digit - b'0',
                b'a'..=b'f' => digit - b'a' + 10,
                b'A'..=b'F' => digit - b'A' + 10,
                _ => panic!("invalid hex digit"),
            }
        }

        let mut output = [0u8; $hex.len() / 2];
        let bytes = $hex.as_bytes();

        let mut i = 0;
        while i < output.len() {
            let high = decode_digit(bytes[i * 2]);
            let low = decode_digit(bytes[i * 2 + 1]);
            output[i] = (high << 4) | low;
            i += 1;
        }

        output
    }};
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

/// A valid hex character: one of `[0-9a-fA-F]`.
//
// The `repr(u8)` guarantees that representation matches the ASCII byte value of the character,
// making transmute between `Char` and `u8` sound whenever the byte is a valid hex digit.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum Char {
    /// `'0'`
    Zero = b'0',
    /// `'1'`
    One = b'1',
    /// `'2'`
    Two = b'2',
    /// `'3'`
    Three = b'3',
    /// `'4'`
    Four = b'4',
    /// `'5'`
    Five = b'5',
    /// `'6'`
    Six = b'6',
    /// `'7'`
    Seven = b'7',
    /// `'8'`
    Eight = b'8',
    /// `'9'`
    Nine = b'9',
    /// `'a'`
    LowerA = b'a',
    /// `'b'`
    LowerB = b'b',
    /// `'c'`
    LowerC = b'c',
    /// `'d'`
    LowerD = b'd',
    /// `'e'`
    LowerE = b'e',
    /// `'f'`
    LowerF = b'f',
    /// `'A'`
    UpperA = b'A',
    /// `'B'`
    UpperB = b'B',
    /// `'C'`
    UpperC = b'C',
    /// `'D'`
    UpperD = b'D',
    /// `'E'`
    UpperE = b'E',
    /// `'F'`
    UpperF = b'F',
}

impl Char {
    /// Casts a slice of `Char`s to `&str`.
    ///
    /// This conversion is zero-cost.
    #[inline]
    pub fn slice_as_str(slice: &[Self]) -> &str {
        let bytes = Self::slice_as_bytes(slice);
        // Guaranteed becuase it's all ASCII.
        unsafe { core::str::from_utf8_unchecked(bytes) }
    }

    /// Casts a slice of `Char`s to `&[u8]`.
    ///
    /// This conversion is zero-cost.
    #[inline]
    pub fn slice_as_bytes(slice: &[Self]) -> &[u8] {
        let ptr = slice.as_ptr().cast();
        let len = slice.len();
        // SOUNDNESS: `Self` is repr(u8)
        // Because all chars are ASCII a slice of chars is also guaranteed to be valid slice of
        // bytes.
        unsafe { core::slice::from_raw_parts(ptr, len) }
    }
}

impl From<Char> for char {
    #[inline]
    fn from(c: Char) -> char { char::from(c as u8) }
}

impl From<Char> for u8 {
    #[inline]
    fn from(c: Char) -> u8 { c as u8 }
}

/// Correctness boundary for `Table`.
mod table {
    use super::Char;

    /// Table of hex chars.
    //
    // Correctness invariant: each byte in the table must be ASCII.
    #[allow(clippy::derived_hash_with_manual_eq)] // The Eq impl distinguishes the two possible values of Table
    #[derive(Debug, Hash)]
    pub(crate) struct Table([Char; 16]);

    impl Table {
        #[rustfmt::skip] // rustfmt wants to make these one per line.
        pub(crate) const LOWER: Self = Table([
            Char::Zero, Char::One, Char::Two, Char::Three,
            Char::Four, Char::Five, Char::Six, Char::Seven,
            Char::Eight, Char::Nine, Char::LowerA, Char::LowerB,
            Char::LowerC, Char::LowerD, Char::LowerE, Char::LowerF,
        ]);
        #[rustfmt::skip] // rustfmt wants to make these one per line.
        pub(crate) const UPPER: Self = Table([
            Char::Zero, Char::One, Char::Two, Char::Three,
            Char::Four, Char::Five, Char::Six, Char::Seven,
            Char::Eight, Char::Nine, Char::UpperA, Char::UpperB,
            Char::UpperC, Char::UpperD, Char::UpperE, Char::UpperF,
        ]);

        /// Encodes single byte as two ASCII chars using the given table.
        ///
        /// The function guarantees only returning values from the provided table.
        #[inline]
        pub(crate) fn byte_to_chars(&self, byte: u8) -> [char; 2] {
            self.byte_to_hex_chars(byte).map(char::from)
        }

        /// Writes the single byte as two ASCII chars in the provided buffer, and returns a `&str`
        /// to that buffer.
        ///
        /// The function guarantees only returning values from the provided table.
        #[inline]
        pub(crate) fn byte_to_str<'a>(&self, dest: &'a mut [u8; 2], byte: u8) -> &'a str {
            dest[0] = self.0[usize::from(byte >> 4)].into();
            dest[1] = self.0[usize::from(byte & 0x0F)].into();
            // SAFETY: Table inner array contains only valid ascii
            let hex_str = unsafe { core::str::from_utf8_unchecked(dest) };
            hex_str
        }

        /// Encodes a single byte as two [`Char`] values using the given table.
        ///
        /// The function guarantees only returning values from the provided table.
        #[inline]
        pub(crate) fn byte_to_hex_chars(&self, byte: u8) -> [Char; 2] {
            let left = self.0[usize::from(byte >> 4)];
            let right = self.0[usize::from(byte & 0x0F)];
            [left, right]
        }
    }

    impl PartialEq for Table {
        // Table can only be Table::LOWER or Table::UPPER. These differ in any of the Chars from
        // indices 10-15.
        fn eq(&self, other: &Self) -> bool { self.0[10] == other.0[10] }
    }
    impl Eq for Table {}
}

#[cfg(test)]
#[cfg(feature = "alloc")]
mod tests {
    #[test]
    fn hex_macro() {
        let data = hex!("deadbeef");
        assert_eq!(data, [0xde, 0xad, 0xbe, 0xef]);
    }

    #[test]
    fn hex_macro_case_insensitive() {
        assert_eq!(hex!("DEADBEEF"), hex!("deadbeef"));
    }

    #[test]
    fn hex_macro_const_context() {
        const HASH: [u8; 32] =
            hex!("000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f");
        assert_eq!(HASH[0], 0x00);
        assert_eq!(HASH[31], 0x6f);
    }

    #[test]
    fn char_slice_casts() {
        use super::Char;

        const BEEF: &[Char] = &[Char::LowerB, Char::LowerE, Char::LowerE, Char::LowerF];

        assert_eq!(Char::slice_as_bytes(&[]), &[]);
        assert_eq!(Char::slice_as_bytes(&BEEF[..1]), b"b");
        assert_eq!(Char::slice_as_bytes(BEEF), b"beef");
        assert_eq!(Char::slice_as_str(&[]), "");
        assert_eq!(Char::slice_as_str(&BEEF[..1]), "b");
        assert_eq!(Char::slice_as_str(BEEF), "beef");
    }
}
