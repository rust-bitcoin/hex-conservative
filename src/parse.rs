// SPDX-License-Identifier: CC0-1.0

//! Hex encoding and decoding.

#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::vec::Vec;
use core::{fmt, str};

use arrayvec::ArrayVec;

use crate::error::{InvalidLengthError, OddLengthError};
use crate::iter::HexToBytesIter;

#[rustfmt::skip]                // Keep public re-exports separate.
#[doc(inline)]
pub use crate::error::{HexToVecError, HexToArrayError, InvalidCharError};

/// Trait for objects that can be deserialized from hex strings.
pub trait FromHex: Sized {
    /// Error type returned while constructing type from byte iterator.
    type FromByteIterError: fmt::Debug + fmt::Display;

    /// Error type returned by `from_hex`.
    type FromHexError: From<Self::FromByteIterError> + fmt::Debug + fmt::Display;

    /// Produces an object from a byte iterator.
    fn from_byte_iter<I>(iter: I) -> Result<Self, Self::FromByteIterError>
    where
        I: Iterator<Item = Result<u8, InvalidCharError>> + ExactSizeIterator + DoubleEndedIterator;

    /// Produces an object from a hex string.
    ///
    /// Override this method if you need to do length checks on the input string (including odd
    /// length) or if you would like to handle `0x` prefix. The default implementation does not
    /// accept a prefix and pads with a leading '0' if the input string has odd length.
    ///
    /// You will get an [`InvalidCharError`] for the `x` if `s` includes a prefix.
    fn from_hex(s: &str) -> Result<Self, Self::FromHexError> {
        let iter = HexToBytesIter::new(s);
        Ok(Self::from_byte_iter(iter)?)
    }
}

#[cfg(any(test, feature = "std", feature = "alloc"))]
impl FromHex for Vec<u8> {
    type FromByteIterError = InvalidCharError;
    type FromHexError = HexToVecError;

    #[inline]
    fn from_byte_iter<I>(iter: I) -> Result<Self, Self::FromByteIterError>
    where
        I: Iterator<Item = Result<u8, InvalidCharError>> + ExactSizeIterator + DoubleEndedIterator,
    {
        iter.collect()
    }

    #[inline]
    fn from_hex(s: &str) -> Result<Self, Self::FromHexError> {
        if s.len() % 2 == 1 {
            return Err(OddLengthError::new(s.len()).into());
        }
        let iter = HexToBytesIter::new(s);
        Ok(Self::from_byte_iter(iter)?)
    }
}

impl<const LEN: usize> FromHex for [u8; LEN] {
    type FromByteIterError = HexToArrayError;
    type FromHexError = HexToArrayError;

    fn from_byte_iter<I>(iter: I) -> Result<Self, Self::FromByteIterError>
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

    #[inline]
    fn from_hex(s: &str) -> Result<Self, Self::FromHexError> {
        let expected = LEN * 2; // 2 hex characters per byte.

        // We don't want to any padding so we check the length.
        if s.len() != expected {
            return Err(InvalidLengthError::new(s.len(), expected).into());
        }

        let iter = HexToBytesIter::new(s);
        Self::from_byte_iter(iter)
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
        use crate::error::{InvalidCharError, InvalidLengthError, OddLengthError};

        let oddlen = "0123456789abcdef0";
        let badchar1 = "Z123456789abcdef";
        let badchar2 = "012Y456789abcdeb";
        let badchar3 = "«23456789abcdef";

        assert_eq!(Vec::<u8>::from_hex(oddlen), Err(OddLengthError::new(17).into()));
        assert_eq!(
            <[u8; 4]>::from_hex(oddlen),
            Err(HexToArrayError::InvalidLength(InvalidLengthError::new(17, 8)))
        );
        assert_eq!(Vec::<u8>::from_hex(badchar1), Err(InvalidCharError { invalid: b'Z' }.into()));
        assert_eq!(Vec::<u8>::from_hex(badchar2), Err(InvalidCharError { invalid: b'Y' }.into()));
        assert_eq!(Vec::<u8>::from_hex(badchar3), Err(InvalidCharError { invalid: 194 }.into()));
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
            Err(InvalidLengthError { expected: 8, got: 16 }.into())
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
