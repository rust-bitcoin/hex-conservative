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
pub enum FromHexError<E> {
    /// Failed to create object from bytes iterator.
    Invalid(E),
    /// Purported hex string had odd length.
    OddLengthString(OddLengthStringError),
}

impl<E: fmt::Display> fmt::Display for FromHexError<E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use FromHexError::*;

        match *self {
            Invalid(ref e) => write_err!(f, "invalid"; e),
            OddLengthString(ref e) =>
                write_err!(f, "odd length, failed to create bytes from hex"; e),
        }
    }
}

#[cfg(feature = "std")]
impl<E: std::error::Error + 'static> std::error::Error for FromHexError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        use FromHexError::*;

        match *self {
            Invalid(ref e) => Some(e),
            OddLengthString(ref e) => Some(e),
        }
    }
}

impl<E> From<OddLengthStringError> for FromHexError<E> {
    #[inline]
    fn from(e: OddLengthStringError) -> Self { Self::OddLengthString(e) }
}

/// Error decoding a hex string that explicitly excludes a prefix.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FromNoPrefixHexError<E> {
    /// Input string contains a prefix.
    Prefix(ContainsPrefixError),
    /// Failed to create object from bytes iterator.
    Invalid(E),
    /// Purported hex string had odd length.
    OddLengthString(OddLengthStringError),
}

impl<E: fmt::Display> fmt::Display for FromNoPrefixHexError<E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use FromNoPrefixHexError::*;

        match *self {
            Prefix(ref e) => write_err!(f, "prefix"; e),
            Invalid(ref e) => write_err!(f, "invalid"; e),
            OddLengthString(ref e) =>
                write_err!(f, "odd length, failed to create bytes from hex"; e),
        }
    }
}

#[cfg(feature = "std")]
impl<E: std::error::Error + 'static> std::error::Error for FromNoPrefixHexError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        use FromNoPrefixHexError::*;

        match *self {
            Prefix(ref e) => Some(e),
            Invalid(ref e) => Some(e),
            OddLengthString(ref e) => Some(e),
        }
    }
}

impl<E> From<ContainsPrefixError> for FromNoPrefixHexError<E> {
    #[inline]
    fn from(e: ContainsPrefixError) -> Self { Self::Prefix(e) }
}

impl<E> From<OddLengthStringError> for FromNoPrefixHexError<E> {
    #[inline]
    fn from(e: OddLengthStringError) -> Self { Self::OddLengthString(e) }
}

/// Error decoding a hex string that explicitly includes a prefix.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FromPrefixedHexError<E> {
    /// Input string is missing a prefix.
    Prefix(MissingPrefixError),
    /// Failed to create object from bytes iterator.
    Invalid(E),
    /// Purported hex string had odd length.
    OddLengthString(OddLengthStringError),
}

impl<E: fmt::Display> fmt::Display for FromPrefixedHexError<E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use FromPrefixedHexError::*;

        match *self {
            Prefix(ref e) => write_err!(f, "prefix"; e),
            Invalid(ref e) => write_err!(f, "invalid"; e),
            OddLengthString(ref e) =>
                write_err!(f, "odd length, failed to create bytes from hex"; e),
        }
    }
}

#[cfg(feature = "std")]
impl<E: std::error::Error + 'static> std::error::Error for FromPrefixedHexError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        use FromPrefixedHexError::*;

        match *self {
            Prefix(ref e) => Some(e),
            Invalid(ref e) => Some(e),
            OddLengthString(ref e) => Some(e),
        }
    }
}

impl<E> From<MissingPrefixError> for FromPrefixedHexError<E> {
    #[inline]
    fn from(e: MissingPrefixError) -> Self { Self::Prefix(e) }
}

impl<E> From<OddLengthStringError> for FromPrefixedHexError<E> {
    #[inline]
    fn from(e: OddLengthStringError) -> Self { Self::OddLengthString(e) }
}

/// Purported hex string had odd length.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct OddLengthStringError {
    pub(crate) len: usize,
}

impl OddLengthStringError {
    /// Returns the length of the input string that caused this error.
    pub fn input_string_length(&self) -> usize { self.len }
}

impl fmt::Display for OddLengthStringError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "odd hex string length {}", self.len)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for OddLengthStringError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> { None }
}

/// Input string contains a prefix.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ContainsPrefixError {
    #[cfg(feature = "alloc")]
    input: String,
}

impl ContainsPrefixError {
    /// Creates the error using `input` string if we have an allocater.
    #[allow(unused_variables)] // If alloc feature is not enabled.
    pub fn new(input: &str) -> Self {
        #[cfg(feature = "alloc")]
        let result = Self { input: input.to_string() };
        #[cfg(not(feature = "alloc"))]
        let result = Self {};

        result
    }
}

impl fmt::Display for ContainsPrefixError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        #[cfg(feature = "alloc")]
        let result = write!(f, "input string contained a prefix (e.g. 0x): {}", self.input);
        #[cfg(not(feature = "alloc"))]
        let result = write!(f, "input string contained a prefix (e.g. 0x)");

        result
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ContainsPrefixError {}

/// Input string is missing a prefix.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct MissingPrefixError {
    #[cfg(feature = "alloc")]
    input: String,
}

impl MissingPrefixError {
    /// Creates the error using `input` string if we have an allocater.
    #[allow(unused_variables)] // If alloc feature is not enabled.
    pub fn new(input: &str) -> Self {
        #[cfg(feature = "alloc")]
        let result = Self { input: input.to_string() };
        #[cfg(not(feature = "alloc"))]
        let result = Self {};

        result
    }
}

impl fmt::Display for MissingPrefixError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        #[cfg(feature = "alloc")]
        let result = write!(f, "input string is missing a prefix (e.g. 0x): {}", self.input);
        #[cfg(not(feature = "alloc"))]
        let result = write!(f, "input string is missing a prefix (e.g. 0x)");

        result
    }
}

#[cfg(feature = "std")]
impl std::error::Error for MissingPrefixError {}

/// Invalid hex character.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct InvalidCharError {
    pub(crate) invalid: u8,
}

impl InvalidCharError {
    /// Returns the invalid character.
    pub fn invalid_char(&self) -> u8 { self.invalid }
}

impl fmt::Display for InvalidCharError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid hex char {}", self.invalid)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for InvalidCharError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> { None }
}

/// Hex decoding error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HexToArrayError {
    /// Invalid character while parsing hex string.
    InvalidChar(InvalidCharError),
    /// Tried to parse fixed-length hash from a string with the wrong length.
    InvalidLength(InvalidLengthError),
}

impl fmt::Display for HexToArrayError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use HexToArrayError::*;

        match *self {
            InvalidChar(ref e) =>
                crate::write_err!(f, "invalid char, failed to create array from hex"; e),
            InvalidLength(ref e) =>
                write_err!(f, "invalid length, failed to create array from hex"; e),
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

/// Tried to parse fixed-length hash from a string with the wrong length.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct InvalidLengthError {
    pub(crate) expected: usize,
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
impl std::error::Error for InvalidLengthError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> { None }
}
