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

/// Hex decoding error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HexToBytesError(pub(crate) ToBytesError);

impl From<Infallible> for HexToBytesError {
    fn from(never: Infallible) -> Self { match never {} }
}

impl HexToBytesError {
    /// Returns a [`ToBytesError`] from this [`HexToBytesError`].
    // Use clone instead of reference to give use maximum forward flexibility.
    pub fn parse_error(&self) -> ToBytesError { self.0.clone() }
}

impl fmt::Display for HexToBytesError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { fmt::Display::fmt(&self.0, f) }
}

#[cfg(feature = "std")]
impl std::error::Error for HexToBytesError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> { Some(&self.0) }
}

impl From<InvalidCharError> for HexToBytesError {
    #[inline]
    fn from(e: InvalidCharError) -> Self { Self(e.into()) }
}

impl From<OddLengthStringError> for HexToBytesError {
    #[inline]
    fn from(e: OddLengthStringError) -> Self { Self(e.into()) }
}

/// Hex decoding error while parsing to a vector of bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToBytesError {
    /// Non-hexadecimal character.
    InvalidChar(InvalidCharError),
    /// Purported hex string had odd length.
    OddLengthString(OddLengthStringError),
}

impl From<Infallible> for ToBytesError {
    fn from(never: Infallible) -> Self { match never {} }
}

impl fmt::Display for ToBytesError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use ToBytesError::*;

        match *self {
            InvalidChar(ref e) => write_err!(f, "invalid char, failed to create bytes from hex"; e),
            OddLengthString(ref e) =>
                write_err!(f, "odd length, failed to create bytes from hex"; e),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ToBytesError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        use ToBytesError::*;

        match *self {
            InvalidChar(ref e) => Some(e),
            OddLengthString(ref e) => Some(e),
        }
    }
}

impl From<InvalidCharError> for ToBytesError {
    #[inline]
    fn from(e: InvalidCharError) -> Self { Self::InvalidChar(e) }
}

impl From<OddLengthStringError> for ToBytesError {
    #[inline]
    fn from(e: OddLengthStringError) -> Self { Self::OddLengthString(e) }
}

/// Invalid hex character.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidCharError {
    pub(crate) invalid: u8,
    pub(crate) pos: usize,
}

impl From<Infallible> for InvalidCharError {
    fn from(never: Infallible) -> Self { match never {} }
}

impl InvalidCharError {
    /// Returns the invalid character byte.
    pub fn invalid_char(&self) -> u8 { self.invalid }
    /// Returns the position of the invalid character byte.
    pub fn pos(&self) -> usize { self.pos }
}

impl fmt::Display for InvalidCharError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid hex char {} at pos {}", self.invalid_char(), self.pos())
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
    fn from(never: Infallible) -> Self { match never {} }
}

impl OddLengthStringError {
    /// Returns the odd length of the input string.
    pub fn length(&self) -> usize { self.len }
}

impl fmt::Display for OddLengthStringError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "odd hex string length {}", self.length())
    }
}

#[cfg(feature = "std")]
impl std::error::Error for OddLengthStringError {}

/// Hex decoding error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HexToArrayError(pub(crate) ToArrayError);

impl From<Infallible> for HexToArrayError {
    fn from(never: Infallible) -> Self { match never {} }
}

impl HexToArrayError {
    /// Returns a [`ToArrayError`] from this [`HexToArrayError`].
    // Use clone instead of reference to give use maximum forward flexibility.
    pub fn parse_error(&self) -> ToArrayError { self.0.clone() }
}

impl fmt::Display for HexToArrayError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { fmt::Display::fmt(&self.0, f) }
}

#[cfg(feature = "std")]
impl std::error::Error for HexToArrayError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> { Some(&self.0) }
}

impl From<InvalidCharError> for HexToArrayError {
    #[inline]
    fn from(e: InvalidCharError) -> Self { Self(e.into()) }
}

impl From<InvalidLengthError> for HexToArrayError {
    #[inline]
    fn from(e: InvalidLengthError) -> Self { Self(e.into()) }
}

/// Hex decoding error while parsing a byte array.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToArrayError {
    /// Non-hexadecimal character.
    InvalidChar(InvalidCharError),
    /// Tried to parse fixed-length hash from a string with the wrong length.
    InvalidLength(InvalidLengthError),
}

impl From<Infallible> for ToArrayError {
    fn from(never: Infallible) -> Self { match never {} }
}

impl fmt::Display for ToArrayError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use ToArrayError::*;

        match *self {
            InvalidChar(ref e) => write_err!(f, "failed to parse hex digit"; e),
            InvalidLength(ref e) => write_err!(f, "failed to parse hex"; e),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ToArrayError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        use ToArrayError::*;

        match *self {
            InvalidChar(ref e) => Some(e),
            InvalidLength(ref e) => Some(e),
        }
    }
}

impl From<InvalidCharError> for ToArrayError {
    #[inline]
    fn from(e: InvalidCharError) -> Self { Self::InvalidChar(e) }
}

impl From<InvalidLengthError> for ToArrayError {
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
    fn from(never: Infallible) -> Self { match never {} }
}

impl InvalidLengthError {
    /// Returns the expected length.
    pub fn expected_length(&self) -> usize { self.expected }
    /// Returns the position of the invalid character byte.
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
