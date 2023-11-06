// SPDX-License-Identifier: CC0-1.0

use core::fmt;

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
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum HexToBytesError {
    /// Non-hexadecimal character.
    InvalidChar(u8),
    /// Purported hex string had odd length.
    OddLengthString(usize),
}

impl fmt::Display for HexToBytesError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use HexToBytesError::*;

        match *self {
            InvalidChar(ch) => write!(f, "invalid hex character {}", ch),
            OddLengthString(ell) => write!(f, "odd hex string length {}", ell),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for HexToBytesError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        use HexToBytesError::*;

        match self {
            InvalidChar(_) | OddLengthString(_) => None,
        }
    }
}

/// Hex decoding error.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum HexToArrayError {
    /// Conversion error while parsing hex string.
    Conversion(HexToBytesError),
    /// Tried to parse fixed-length hash from a string with the wrong length (got, want).
    InvalidLength(usize, usize),
}

impl fmt::Display for HexToArrayError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use HexToArrayError::*;

        match *self {
            Conversion(ref e) => crate::write_err!(f, "conversion error"; e),
            InvalidLength(got, want) =>
                write!(f, "bad hex string length {} (expected {})", got, want),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for HexToArrayError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        use HexToArrayError::*;

        match *self {
            Conversion(ref e) => Some(e),
            InvalidLength(_, _) => None,
        }
    }
}

impl From<HexToBytesError> for HexToArrayError {
    #[inline]
    fn from(e: HexToBytesError) -> Self { Self::Conversion(e) }
}
