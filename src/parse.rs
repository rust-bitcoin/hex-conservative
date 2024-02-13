// SPDX-License-Identifier: CC0-1.0

//! Hex encoding and decoding.

#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::vec::Vec;
use core::{fmt, str};

use arrayvec::ArrayVec;

use crate::error::{ContainsPrefixError, InvalidLengthError, MissingPrefixError};
use crate::iter::HexToBytesIter;

#[rustfmt::skip]                // Keep public re-exports separate.
#[doc(inline)]
pub use crate::error::{FromHexError, FromNoPrefixHexError, FromPrefixedHexError, HexToArrayError, InvalidCharError};

/// Trait for objects that can be deserialized from hex strings.
pub trait FromHex: Sized {
    /// Error type returned while constructing type from byte iterator.
    type Error: fmt::Debug + fmt::Display;

    /// Produces an object from a byte iterator.
    fn from_byte_iter<I>(iter: I) -> Result<Self, Self::Error>
    where
        I: Iterator<Item = Result<u8, InvalidCharError>> + ExactSizeIterator + DoubleEndedIterator;

    /// Produces an object from a hex string that may or may not include a `0x` prefix.
    ///
    /// Equivalent to [`Self::from_maybe_prefixed_hex`].
    fn from_hex(s: &str) -> Result<Self, FromHexError<Self::Error>> {
        Self::from_maybe_prefixed_hex(s)
    }

    /// Produces an object from a hex string that may or may not include a `0x` prefix.
    fn from_maybe_prefixed_hex(s: &str) -> Result<Self, FromHexError<Self::Error>> {
        let s = s.strip_prefix("0x").unwrap_or(s);
        let iter = HexToBytesIter::new(s).map_err(FromHexError::OddLengthString)?;
        Self::from_byte_iter(iter).map_err(FromHexError::Invalid)
    }

    /// Produces an object from a hex string that does not contain a `0x` prefix.
    fn from_no_prefix_hex(s: &str) -> Result<Self, FromNoPrefixHexError<Self::Error>> {
        if s.strip_prefix("0x").is_some() {
            return Err(ContainsPrefixError::new(s).into());
        }

        let iter = HexToBytesIter::new(s).map_err(FromNoPrefixHexError::OddLengthString)?;
        Self::from_byte_iter(iter).map_err(FromNoPrefixHexError::Invalid)
    }

    /// Produces an object from a `0x` prefixed hex string.
    fn from_prefixed_hex(s: &str) -> Result<Self, FromPrefixedHexError<Self::Error>> {
        if let Some(stripped) = s.strip_prefix("0x") {
            let iter =
                HexToBytesIter::new(stripped).map_err(FromPrefixedHexError::OddLengthString)?;
            Self::from_byte_iter(iter).map_err(FromPrefixedHexError::Invalid)
        } else {
            Err(MissingPrefixError::new(s).into())
        }
    }
}

#[cfg(any(test, feature = "std", feature = "alloc"))]
impl FromHex for Vec<u8> {
    type Error = InvalidCharError;

    #[inline]
    fn from_byte_iter<I>(iter: I) -> Result<Self, Self::Error>
    where
        I: Iterator<Item = Result<u8, InvalidCharError>> + ExactSizeIterator + DoubleEndedIterator,
    {
        iter.collect()
    }
}

impl<const LEN: usize> FromHex for [u8; LEN] {
    type Error = HexToArrayError;

    fn from_byte_iter<I>(iter: I) -> Result<Self, Self::Error>
    where
        I: Iterator<Item = Result<u8, InvalidCharError>> + ExactSizeIterator + DoubleEndedIterator,
    {
        if iter.len() == LEN {
            let mut ret = ArrayVec::<u8, LEN>::new();
            for byte in iter {
                ret.push(byte?);
            }
            Ok(ret.into_inner().expect("inner is full"))
        } else {
            Err(InvalidLengthError { expected: 2 * LEN, got: 2 * iter.len() }.into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::display::DisplayHex;
    use crate::error::InvalidLengthError;

    #[test]
    #[cfg(feature = "alloc")]
    fn hex_error() {
        use crate::error::{InvalidCharError, OddLengthStringError};

        let oddlen = "0123456789abcdef0";
        let badchar1 = "Z123456789abcdef";
        let badchar2 = "012Y456789abcdeb";
        let badchar3 = "«23456789abcdef";

        assert_eq!(Vec::<u8>::from_hex(oddlen), Err(OddLengthStringError { len: 17 }.into()));
        assert_eq!(<[u8; 4]>::from_hex(oddlen), Err(OddLengthStringError { len: 17 }.into()));
        assert_eq!(
            Vec::<u8>::from_hex(badchar1),
            Err(FromHexError::Invalid(InvalidCharError { invalid: b'Z' }))
        );
        assert_eq!(
            Vec::<u8>::from_hex(badchar2),
            Err(FromHexError::Invalid(InvalidCharError { invalid: b'Y' }))
        );
        assert_eq!(
            Vec::<u8>::from_hex(badchar3),
            Err(FromHexError::Invalid(InvalidCharError { invalid: 194 }))
        );
    }

    #[test]
    fn hex_to_array() {
        let len_sixteen = "0123456789abcdef";
        assert!(<[u8; 8]>::from_hex(len_sixteen).is_ok());
    }

    #[test]
    fn hex_to_array_error() {
        let len_sixteen = "0123456789abcdef";
        assert_eq!(
            <[u8; 4]>::from_hex(len_sixteen),
            Err(FromHexError::Invalid(InvalidLengthError { expected: 8, got: 16 }.into()))
        )
    }

    #[test]
    fn mixed_case() {
        let s = "DEADbeef0123";
        let want_lower = "deadbeef0123";
        let want_upper = "DEADBEEF0123";

        let v = Vec::<u8>::from_hex(s).expect("valid hex");
        assert_eq!(format!("{:x}", v.as_hex()), want_lower);
        assert_eq!(format!("{:X}", v.as_hex()), want_upper);
    }
}
