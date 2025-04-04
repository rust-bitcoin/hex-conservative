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
    #[inline]
    fn from(never: Infallible) -> Self { match never {} }
}

impl HexToBytesError {
    /// Returns a [`ToBytesError`] from this [`HexToBytesError`].
    // Use clone instead of reference to give use maximum forward flexibility.
    #[inline]
    pub fn parse_error(&self) -> ToBytesError { self.0.clone() }
}

impl fmt::Display for HexToBytesError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { fmt::Display::fmt(&self.0, f) }
}

#[cfg(feature = "std")]
impl std::error::Error for HexToBytesError {
    #[inline]
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
    #[inline]
    fn from(never: Infallible) -> Self { match never {} }
}

impl fmt::Display for ToBytesError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use ToBytesError as E;

        match *self {
            E::InvalidChar(ref e) => write_err!(f, "failed to decode hex"; e),
            E::OddLengthString(ref e) => write_err!(f, "failed to decode hex"; e),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ToBytesError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        use ToBytesError as E;

        match *self {
            E::InvalidChar(ref e) => Some(e),
            E::OddLengthString(ref e) => Some(e),
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
    #[inline]
    fn from(never: Infallible) -> Self { match never {} }
}

impl InvalidCharError {
    /// Returns the invalid character byte.
    #[inline]
    pub fn invalid_char(&self) -> u8 { self.invalid }
    /// Returns the position of the invalid character byte.
    #[inline]
    pub fn pos(&self) -> usize { self.pos }
}

/// Note that the implementation displays position as 1-based instead of 0-based to be more
/// suitable to end users who might be non-programmers.
impl fmt::Display for InvalidCharError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // We're displaying this for general audience, not programmers, so we want to do 1-based
        // position but that might confuse programmers who might think it's 0-based. Hopefully
        // using more wordy approach will avoid the confusion.

        // format_args! would be simpler but we can't use it because of  Rust issue #92698.
        struct Format<F: Fn(&mut fmt::Formatter<'_>) -> fmt::Result>(F);
        impl<F: Fn(&mut fmt::Formatter<'_>) -> fmt::Result> fmt::Display for Format<F> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { self.0(f) }
        }

        // The lifetime is not extended in MSRV, so we need this.
        let which;
        let which: &dyn fmt::Display = match self.pos() {
            0 => &"1st",
            1 => &"2nd",
            2 => &"3rd",
            pos => {
                which = Format(move |f| write!(f, "{}th", pos + 1));
                &which
            }
        };

        // The lifetime is not extended in MSRV, so we need these.
        let chr_ascii;
        let chr_non_ascii;

        let invalid_char = self.invalid_char();
        // We're currently not storing the entire character, so we need to make sure values >=
        // 128 don't get misinterpreted as ISO-8859-1.
        let chr: &dyn fmt::Display = if self.invalid_char().is_ascii() {
            // Yes, the Debug output is correct here. Display would print the characters
            // directly which would be confusing in case of control characters and it would
            // also mess up the formatting. The `Debug` implementation of `char` properly
            // escapes such characters.
            chr_ascii = Format(move |f| write!(f, "{:?}", invalid_char as char));
            &chr_ascii
        } else {
            chr_non_ascii = Format(move |f| write!(f, "{:#02x}", invalid_char));
            &chr_non_ascii
        };

        write!(f, "the {} character, {}, is not a valid hex digit", which, chr)
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
        if self.length() == 1 {
            write!(f, "the hex string is 1 byte long which is not an even number")
        } else {
            write!(f, "the hex string is {} bytes long which is not an even number", self.length())
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for OddLengthStringError {}

/// Hex decoding error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HexToArrayError(pub(crate) ToArrayError);

impl From<Infallible> for HexToArrayError {
    #[inline]
    fn from(never: Infallible) -> Self { match never {} }
}

impl HexToArrayError {
    /// Returns a [`ToArrayError`] from this [`HexToArrayError`].
    // Use clone instead of reference to give use maximum forward flexibility.
    #[inline]
    pub fn parse_error(&self) -> ToArrayError { self.0.clone() }
}

impl fmt::Display for HexToArrayError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { fmt::Display::fmt(&self.0, f) }
}

#[cfg(feature = "std")]
impl std::error::Error for HexToArrayError {
    #[inline]
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
    #[inline]
    fn from(never: Infallible) -> Self { match never {} }
}

impl fmt::Display for ToArrayError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use ToArrayError as E;

        match *self {
            E::InvalidChar(ref e) => write_err!(f, "failed to parse hex"; e),
            E::InvalidLength(ref e) => write_err!(f, "failed to parse hex"; e),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ToArrayError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        use ToArrayError as E;

        match *self {
            E::InvalidChar(ref e) => Some(e),
            E::InvalidLength(ref e) => Some(e),
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
            // Note on singular vs plural: expected length is never odd, so it cannot be 1
            "the hex string is {} bytes long but exactly {} bytes were required",
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
        if let HexToBytesError(ToBytesError::InvalidChar(e)) = error {
            assert!(!format!("{}", e).is_empty());
            assert_eq!(e.invalid_char(), b'G');
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
        if let HexToBytesError(ToBytesError::OddLengthString(e)) = error {
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
        if let HexToArrayError(ToArrayError::InvalidLength(e)) = error {
            assert!(!format!("{}", e).is_empty());
            assert_eq!(e.expected_length(), 8);
            assert_eq!(e.invalid_length(), 3);
        } else {
            panic!("Expected InvalidLengthError");
        }
    }

    #[test]
    fn to_bytes_error() {
        let error = ToBytesError::OddLengthString(OddLengthStringError { len: 7 });
        assert!(!format!("{}", error).is_empty());
        check_source(&error);
    }

    #[test]
    fn to_array_error() {
        let error = ToArrayError::InvalidLength(InvalidLengthError { expected: 8, invalid: 7 });
        assert!(!format!("{}", error).is_empty());
        check_source(&error);
    }
}
