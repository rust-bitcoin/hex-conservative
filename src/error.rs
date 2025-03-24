// SPDX-License-Identifier: CC0-1.0

//! Error code for the `hex-conservative` crate.

use core::convert::Infallible;
use core::fmt;

/// Formats error.
///
/// If `std` feature is OFF appends error source (delimited by `: `). We do this because
/// `e.source()` is only available in std builds, without this macro the error source is lost for
/// no-std builds.
macro_rules! write_err {
    ($writer:expr, $string:literal $(, $args:expr)*; $source:expr) => {
        {
            #[cfg(feature = "std")]
            {
                let _ = &$source;   // Prevents clippy warnings.
                write!($writer, $string $(, $args)*)
            }
            #[cfg(not(feature = "std"))]
            {
                write!($writer, concat!($string, ": {}") $(, $args)*, $source)
            }
        }
    }
}

/// Hex decoding error while parsing to a vector of bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HexToBytesError {
    /// Non-hexadecimal character.
    InvalidChar(InvalidCharError),
    /// Purported hex string had odd length.
    OddLengthString(OddLengthStringError),
}

impl From<Infallible> for HexToBytesError {
    #[inline]
    fn from(never: Infallible) -> Self { match never {} }
}

impl From<InvalidCharError> for HexToBytesError {
    #[inline]
    fn from(e: InvalidCharError) -> Self { Self::InvalidChar(e) }
}

impl From<OddLengthStringError> for HexToBytesError {
    #[inline]
    fn from(e: OddLengthStringError) -> Self { Self::OddLengthString(e) }
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

/// Invalid hex character.
///
/// This error type only supports ASCII characters. If you try to parse a hex string that
/// includes multi-byte UTF-8 encoded characters this error will be confusing as it will only
/// include the first byte of the encoding not the whole character.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidCharError {
    pub(crate) invalid: u8,
    pub(crate) pos: usize,
}

impl From<Infallible> for InvalidCharError {
    #[inline]
    fn from(never: Infallible) -> Self { match never {} }
}

impl InvalidCharError {
    /// Returns the invalid character byte.
    ///
    /// If the parsing error is caused by a multi-byte UTF-8 encoded character then this will only
    /// return the first byte of that character.
    #[inline]
    pub fn invalid_byte(&self) -> u8 { self.invalid }
    /// Returns the position of the invalid character byte.
    #[inline]
    pub fn pos(&self) -> usize { self.pos }
}

impl fmt::Display for InvalidCharError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid hex char {} at pos {}", self.invalid_byte(), self.pos())
    }
}

#[cfg(feature = "std")]
impl std::error::Error for InvalidCharError {}

/// Purported hex string had odd length.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OddLengthStringError {
    pub(crate) len: usize,
}

impl From<Infallible> for OddLengthStringError {
    #[inline]
    fn from(never: Infallible) -> Self { match never {} }
}

impl OddLengthStringError {
    /// Returns the odd length of the input string.
    #[inline]
    pub fn length(&self) -> usize { self.len }
}

impl fmt::Display for OddLengthStringError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "odd hex string length {}", self.length())
    }
}

#[cfg(feature = "std")]
impl std::error::Error for OddLengthStringError {}

/// Hex decoding error while parsing a byte array.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HexToArrayError {
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

/// Tried to parse fixed-length hash from a string with the wrong length.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidLengthError {
    /// The expected length.
    pub(crate) expected: usize,
    /// The invalid length.
    pub(crate) invalid: usize,
}

impl From<Infallible> for InvalidLengthError {
    #[inline]
    fn from(never: Infallible) -> Self { match never {} }
}

impl InvalidLengthError {
    /// Returns the expected length.
    #[inline]
    pub fn expected_length(&self) -> usize { self.expected }
    /// Returns the position of the invalid character byte.
    #[inline]
    pub fn invalid_length(&self) -> usize { self.invalid }
}

impl fmt::Display for InvalidLengthError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "invalid hex string length {} (expected {})",
            self.invalid_length(),
            self.expected_length()
        )
    }
}

#[cfg(feature = "std")]
impl std::error::Error for InvalidLengthError {}

#[cfg(test)]
#[cfg(feature = "std")]
mod tests {
    use super::*;
    use crate::FromHex;

    fn check_source<T: std::error::Error>(error: &T) {
        assert!(error.source().is_some());
    }

    #[test]
    fn invalid_char_error() {
        let result = <Vec<u8> as FromHex>::from_hex("12G4");
        let error = result.unwrap_err();
        if let HexToBytesError::InvalidChar(e) = error {
            assert!(!format!("{}", e).is_empty());
            assert_eq!(e.invalid_byte(), b'G');
            assert_eq!(e.pos(), 2);
        } else {
            panic!("Expected InvalidCharError");
        }
    }

    #[test]
    fn odd_length_string_error() {
        let result = <Vec<u8> as FromHex>::from_hex("123");
        let error = result.unwrap_err();
        assert!(!format!("{}", error).is_empty());
        check_source(&error);
        if let HexToBytesError::OddLengthString(e) = error {
            assert!(!format!("{}", e).is_empty());
            assert_eq!(e.length(), 3);
        } else {
            panic!("Expected OddLengthStringError");
        }
    }

    #[test]
    fn invalid_length_error() {
        let result = <[u8; 4] as FromHex>::from_hex("123");
        let error = result.unwrap_err();
        assert!(!format!("{}", error).is_empty());
        check_source(&error);
        if let HexToArrayError::InvalidLength(e) = error {
            assert!(!format!("{}", e).is_empty());
            assert_eq!(e.expected_length(), 8);
            assert_eq!(e.invalid_length(), 3);
        } else {
            panic!("Expected InvalidLengthError");
        }
    }

    #[test]
    fn to_bytes_error() {
        let error = HexToBytesError::OddLengthString(OddLengthStringError { len: 7 });
        assert!(!format!("{}", error).is_empty());
        check_source(&error);
    }

    #[test]
    fn to_array_error() {
        let error = HexToArrayError::InvalidLength(InvalidLengthError { expected: 8, invalid: 7 });
        assert!(!format!("{}", error).is_empty());
        check_source(&error);
    }
}
