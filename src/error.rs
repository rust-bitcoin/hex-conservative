// SPDX-License-Identifier: CC0-1.0

//! Error code for the `hex-conservative` crate.

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

/// UTF-8 character is not an ASCII hex character.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidCharError {
    pub(crate) invalid: char,
    pub(crate) pos: usize,
}

impl InvalidCharError {
    /// Returns the invalid character.
    pub fn invalid_char(&self) -> char { self.invalid }

    /// Returns the position in the input string of the invalid character.
    pub fn pos(&self) -> usize { self.pos }
}

impl fmt::Display for InvalidCharError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "char {} at pos {} is not a valid ASCII hex character",
            self.invalid_char(),
            self.pos()
        )
    }
}

#[cfg(feature = "std")]
impl std::error::Error for InvalidCharError {}

/// Invalid ASCII encoding of a hex digit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidDigitError {
    /// The high digit in the pair e.g., for 'ab' this is 'a'.
    pub(crate) hi: u8,

    /// The high digit in the pair e.g., for 'ab' this is 'b'.
    pub(crate) lo: u8,

    /// `true` if the invalid byte is the `hi` one, `false` if its the `lo` one.
    pub(crate) is_hi: bool,

    /// Position of the stat of the byte pair in the input string.
    pub(crate) pos: usize,
}

impl InvalidDigitError {
    /// Returns the invalid byte.
    pub fn invalid_byte(&self) -> u8 {
        if self.is_hi {
            self.hi
        } else {
            self.lo
        }
    }

    /// Returns the character position in input string of the invalid byte.
    pub fn pos(&self) -> usize {
        if self.is_hi {
            self.pos
        } else {
            self.pos + 1
        }
    }

    pub(crate) fn into_invalid_char_error(
        self,
        // The next item from the iterator.
        next: Option<Result<u8, InvalidDigitError>>,
    ) -> InvalidCharError {
        // Try to map the UTF-8 values in `v` to an invalid character.
        fn error(v: &[u8], pos: usize) -> Option<InvalidCharError> {
            core::str::from_utf8(v)
                .map(|s| InvalidCharError { invalid: s.chars().next().unwrap(), pos })
                .ok()
        }

        let rich_error = match next {
            Some(res) => match res {
                // The next two bytes happen to be valid ASCII.
                Ok(byte) => {
                    let (hi, lo) = crate::iter::byte_to_hex_digits(byte);
                    if !self.is_hi {
                        let vals = [self.lo];
                        error(&vals, self.pos + 1).or_else(|| {
                            let vals = [self.lo, hi];
                            error(&vals, self.pos + 1).or_else(|| {
                                let vals = [self.lo, hi, lo];
                                error(&vals, self.pos + 1)
                            })
                        })
                    } else {
                        let vals = [self.hi];
                        error(&vals, self.pos).or_else(|| {
                            let vals = [self.hi, self.lo];
                            error(&vals, self.pos).or_else(|| {
                                let vals = [self.hi, self.lo, hi];
                                error(&vals, self.pos).or_else(|| {
                                    let vals = [self.hi, self.lo, hi, lo];
                                    error(&vals, self.pos)
                                })
                            })
                        })
                    }
                }
                // The next two bytes happen to be invalid ASCII.
                Err(e) =>
                    if !self.is_hi {
                        let vals = [self.lo, e.hi];
                        error(&vals, self.pos + 1).or_else(|| {
                            let vals = [self.lo, e.hi, e.lo];
                            error(&vals, self.pos + 1)
                        })
                    } else {
                        let vals = [self.hi, self.lo, e.hi];
                        error(&vals, self.pos).or_else(|| {
                            let vals = [self.hi, self.lo];
                            error(&vals, self.pos).or_else(|| {
                                let vals = [self.hi, self.lo, e.hi];
                                error(&vals, self.pos).or_else(|| {
                                    let vals = [self.hi, self.lo, e.hi, e.lo];
                                    error(&vals, self.pos)
                                })
                            })
                        })
                    },
            },
            // Invalid character was for the last character in the input string.
            None =>
                if !self.is_hi {
                    let vals = [self.lo];
                    error(&vals, self.pos + 1)
                } else {
                    let vals = [self.hi];
                    error(&vals, self.pos).or_else(|| {
                        let vals = [self.hi, self.lo];
                        error(&vals, self.pos)
                    })
                },
        };
        rich_error.unwrap_or(InvalidCharError {
            invalid: if self.is_hi { self.hi.into() } else { self.lo.into() },
            pos: self.pos,
        })
    }
}

impl fmt::Display for InvalidDigitError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "byte value {:x} is not a valid ASCII encoding of a hex digit (pos {})",
            self.invalid_byte(),
            self.pos()
        )
    }
}

#[cfg(feature = "std")]
impl std::error::Error for InvalidDigitError {}

/// Purported hex string had odd length.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OddLengthStringError {
    pub(crate) len: usize,
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
            "invilad hex string length {} (expected {})",
            self.invalid_length(),
            self.expected_length()
        )
    }
}

#[cfg(feature = "std")]
impl std::error::Error for InvalidLengthError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_digit_to_invalid_char_za() {
        // Input string: 'za'
        let e = InvalidDigitError { hi: b'z', lo: b'a', is_hi: true, pos: 0 };
        let want = InvalidCharError { invalid: 'z', pos: 0 };
        let got = e.into_invalid_char_error(None);
        assert_eq!(got, want);
    }

    #[test]
    fn invalid_digit_to_invalid_char_az() {
        // Input string: 'az'
        let e = InvalidDigitError {
            hi: b'a',
            lo: b'z',
            is_hi: false,
            pos: 0, // This is the position of 'hi'.
        };
        let want = InvalidCharError { invalid: 'z', pos: 1 }; // This is the position of 'z'.
        let got = e.into_invalid_char_error(None);
        assert_eq!(got, want);
    }
}
