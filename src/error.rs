// SPDX-License-Identifier: CC0-1.0

//! Error types for the hex crate.

#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::string::{String, ToString};
use core::fmt;

use crate::write_err;

/// Formats error.
///
/// If `std` feature is OFF appends error source (delimited by `: `). We do this because
/// `e.source()` is only available in std builds, without this macro the error source is lost for
/// no-std builds.
#[macro_export]
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

/// Hex decoding error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HexToVecError {
    /// Purported hex string had odd length.
    OddLength(OddLengthError),
    /// Invalid character while parsing hex string.
    InvalidChar(InvalidCharError),
}

impl fmt::Display for HexToVecError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use HexToVecError::*;

        match *self {
            OddLength(ref e) => write_err!(f, "odd length, failed to create bytes from hex"; e),
            InvalidChar(ref e) => write_err!(f, "invalid char, failed to create bytes from hex"; e),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for HexToVecError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        use HexToVecError::*;

        match *self {
            OddLength(ref e) => Some(e),
            InvalidChar(ref e) => Some(e),
        }
    }
}

impl From<OddLengthError> for HexToVecError {
    #[inline]
    fn from(e: OddLengthError) -> Self { Self::OddLength(e) }
}

impl From<InvalidCharError> for HexToVecError {
    #[inline]
    fn from(e: InvalidCharError) -> Self { Self::InvalidChar(e) }
}

/// Hex decoding error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HexToArrayError {
    /// Tried to parse fixed-length hash from a string with the wrong length.
    InvalidLength(InvalidLengthError),
    /// Invalid character while parsing hex string.
    InvalidChar(InvalidCharError),
}

impl fmt::Display for HexToArrayError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use HexToArrayError::*;

        match *self {
            InvalidLength(ref e) =>
                write_err!(f, "invalid length, failed to create array from hex"; e),
            InvalidChar(ref e) =>
                crate::write_err!(f, "invalid char, failed to create array from hex"; e),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for HexToArrayError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        use HexToArrayError::*;

        match *self {
            InvalidChar(ref e) => Some(e),
            InvalidLength(ref e) => Some(e),
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

/// Invalid hex character in input string.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct InvalidCharError {
    pub(crate) invalid: u8,
}

impl InvalidCharError {
    /// Creates a new invalid char error.
    pub fn new(invalid: u8) -> Self { Self { invalid } }

    /// Returns the invalid character.
    pub fn invalid_char(&self) -> u8 { self.invalid }
}

impl fmt::Display for InvalidCharError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid hex char {}", self.invalid)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for InvalidCharError {}

/// Purported hex string had odd length.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct OddLengthError {
    pub(crate) len: usize,
}

impl OddLengthError {
    /// Creates a new error from `len` (the input hex string length).
    pub fn new(len: usize) -> Self { Self { len } }

    /// Returns the length of the input string that caused this error.
    pub fn input_string_length(&self) -> usize { self.len }
}

impl fmt::Display for OddLengthError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "odd hex string length {}", self.len)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for OddLengthError {}

/// Tried to parse fixed-length object from a string with the wrong length.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct InvalidLengthError {
    pub(crate) expected: usize, // Number of hex characters.
    pub(crate) got: usize,
}

impl InvalidLengthError {
    /// Creates a new `InvalidLengthError`.
    pub fn new(got: usize, expected: usize) -> Self { Self { expected, got } }

    /// Returns the expected length of the hex string.
    pub fn expected_length(&self) -> usize { self.expected }

    /// Returns the invalid length of the parsed hex string.
    pub fn invalid_length(&self) -> usize { self.got }
}

impl fmt::Display for InvalidLengthError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "bad hex string length {} (expected {})", self.got, self.expected)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for InvalidLengthError {}
