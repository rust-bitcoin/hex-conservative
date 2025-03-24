// SPDX-License-Identifier: CC0-1.0

//! Hex encoding and decoding.

use core::convert::Infallible;
use core::{fmt, str};

#[cfg(all(feature = "alloc", not(feature = "std")))]
use crate::alloc::vec::Vec;
use crate::error::{write_err, InvalidLengthError, InvalidCharError, OddLengthStringError};
use crate::iter::HexToBytesIter;

/// Trait for objects that can be deserialized from hex strings.
pub(crate) trait FromHex: Sized {
    /// Error type returned while parsing hex string.
    type Error: Sized + fmt::Debug + fmt::Display;

    /// Produces an object from a hex string.
    fn from_hex(s: &str) -> Result<Self, Self::Error>;
}

#[cfg(feature = "alloc")]
impl FromHex for Vec<u8> {
    type Error = HexToBytesError;

    fn from_hex(s: &str) -> Result<Self, Self::Error> {
        Ok(HexToBytesIter::new(s)?.drain_to_vec()?)
    }
}

impl<const LEN: usize> FromHex for [u8; LEN] {
    type Error = HexToArrayError;

    fn from_hex(s: &str) -> Result<Self, Self::Error> {
        if s.len() == LEN * 2 {
            let mut ret = [0u8; LEN];
            // checked above
            HexToBytesIter::new_unchecked(s).drain_to_slice(&mut ret)?;
            Ok(ret)
        } else {
            Err(InvalidLengthError { invalid: s.len(), expected: 2 * LEN }.into())
        }
    }
}

/// Hex decoding error while parsing to a vector of bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum HexToBytesError {
    /// Non-hexadecimal character.
    InvalidChar(InvalidCharError),
    /// Purported hex string had odd length.
    OddLengthString(OddLengthStringError),
}

impl From<Infallible> for HexToBytesError {
    #[inline]
    fn from(never: Infallible) -> Self { match never {} }
}

impl fmt::Display for HexToBytesError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use HexToBytesError as E;

        match *self {
            E::InvalidChar(ref e) =>
                write_err!(f, "invalid char, failed to create bytes from hex"; e),
            E::OddLengthString(ref e) =>
                write_err!(f, "odd length, failed to create bytes from hex"; e),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for HexToBytesError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        use HexToBytesError as E;

        match *self {
            E::InvalidChar(ref e) => Some(e),
            E::OddLengthString(ref e) => Some(e),
        }
    }
}

impl From<InvalidCharError> for HexToBytesError {
    #[inline]
    fn from(e: InvalidCharError) -> Self { Self::InvalidChar(e) }
}

impl From<OddLengthStringError> for HexToBytesError {
    #[inline]
    fn from(e: OddLengthStringError) -> Self { Self::OddLengthString(e) }
}

/// Hex decoding error while parsing a byte array.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum HexToArrayError {
    /// Non-hexadecimal character.
    InvalidChar(InvalidCharError),
    /// Tried to parse fixed-length hash from a string with the wrong length.
    InvalidLength(InvalidLengthError),
}

impl From<Infallible> for HexToArrayError {
    #[inline]
    fn from(never: Infallible) -> Self { match never {} }
}

impl fmt::Display for HexToArrayError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use HexToArrayError as E;

        match *self {
            E::InvalidChar(ref e) => write_err!(f, "failed to parse hex digit"; e),
            E::InvalidLength(ref e) => write_err!(f, "failed to parse hex"; e),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for HexToArrayError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        use HexToArrayError as E;

        match *self {
            E::InvalidChar(ref e) => Some(e),
            E::InvalidLength(ref e) => Some(e),
        }
    }
}

impl From<InvalidCharError> for HexToArrayError {
    #[inline]
    fn from(e: InvalidCharError) -> Self { Self::InvalidChar(e) }
}

impl From<InvalidLengthError> for HexToArrayError {
    #[inline]
    fn from(e: InvalidLengthError) -> Self { Self::InvalidLength(e) }
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
            Vec::<u8>::from_hex(oddlen).unwrap_err(),
            OddLengthStringError { len: 17 }.into()
        );
        assert_eq!(
            <[u8; 4]>::from_hex(oddlen).unwrap_err(),
            InvalidLengthError { invalid: 17, expected: 8 }.into()
        );
        assert_eq!(
            Vec::<u8>::from_hex(badchar1).unwrap_err(),
            InvalidCharError { pos: 0, invalid: b'Z' }.into()
        );
        assert_eq!(
            Vec::<u8>::from_hex(badchar2).unwrap_err(),
            InvalidCharError { pos: 3, invalid: b'Y' }.into()
        );
        assert_eq!(
            Vec::<u8>::from_hex(badchar3).unwrap_err(),
            InvalidCharError { pos: 0, invalid: 194 }.into()
        );
    }

    #[test]
    fn hex_error_position() {
        use crate::error::InvalidCharError;
        let badpos1 = "Z123456789abcdef";
        let badpos2 = "012Y456789abcdeb";
        let badpos3 = "0123456789abcdeZ";
        let badpos4 = "0123456789abYdef";

        assert_eq!(
            HexToBytesIter::new(badpos1).unwrap().next().unwrap().unwrap_err(),
            InvalidCharError { pos: 0, invalid: b'Z' }
        );
        assert_eq!(
            HexToBytesIter::new(badpos2).unwrap().nth(1).unwrap().unwrap_err(),
            InvalidCharError { pos: 3, invalid: b'Y' }
        );
        assert_eq!(
            HexToBytesIter::new(badpos3).unwrap().next_back().unwrap().unwrap_err(),
            InvalidCharError { pos: 15, invalid: b'Z' }
        );
        assert_eq!(
            HexToBytesIter::new(badpos4).unwrap().nth_back(1).unwrap().unwrap_err(),
            InvalidCharError { pos: 12, invalid: b'Y' }
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
            <[u8; 4]>::from_hex(len_sixteen).unwrap_err(),
            InvalidLengthError { invalid: 16, expected: 8 }.into()
        )
    }
}
