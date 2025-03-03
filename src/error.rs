// SPDX-License-Identifier: CC0-1.0

//! Error code for the `hex-conservative` crate.

use core::convert::Infallible;
use core::fmt;
#[cfg(feature = "std")]
use std::error::Error as StdError;
#[cfg(all(not(feature = "std"), feature = "newer-rust-version"))]
if_rust_version::if_rust_version! {
    >= 1.81 {
        use core::error::Error as StdError;
    }
}

#[cfg(feature = "std")]
macro_rules! if_std_error {
    ({ $($if_yes:tt)* } $(else { $($if_not:tt)* })?) => {
        #[cfg_attr(docsrs, doc(cfg(any(feature = "std", all(feature = "newer-rust-version", rust_version = ">= 1.81.0")))))]
        $($if_yes)*
    }
}

#[cfg(all(not(feature = "std"), feature = "newer-rust-version"))]
macro_rules! if_std_error {
    ({ $($if_yes:tt)* } $(else { $($if_not:tt)* })?) => {
        if_rust_version::if_rust_version! {
            >= 1.81 {
                #[cfg_attr(docsrs, doc(cfg(any(feature = "std", all(feature = "newer-rust-version", rust_version = ">= 1.81.0")))))]
                $($if_yes)*
            } $(else { $($if_not)* })?
        }
    }
}

#[cfg(all(not(feature = "std"), not(feature = "newer-rust-version")))]
macro_rules! if_std_error {
    ({ $($if_yes:tt)* } $(else { $($if_not:tt)* })?) => {
        $($($if_not)*)?
    }
}

/// Formats error.
///
/// If `std` feature is OFF appends error source (delimited by `: `). We do this because
/// `e.source()` is only available in std builds, without this macro the error source is lost for
/// no-std builds.
macro_rules! write_err {
    ($writer:expr, $string:literal $(, $args:expr)*; $source:expr) => {
        {
            if_std_error! {
                {
                    {
                        let _ = &$source;   // Prevents clippy warnings.
                        write!($writer, $string $(, $args)*)
                    }
                } else {
                    {
                        write!($writer, concat!($string, ": {}") $(, $args)*, $source)
                    }
                }
            }
        }
    }
}
pub(crate) use write_err;

/// Error returned when hex decoding a hex string with variable length.
///
/// This represents the first error encountered during decoding, however we may add ther remaining
/// ones in the future.
///
/// This error differs from `DecodeFixedSizedBytesError` in that the number of bytes is only known
/// at run time - e.g. when decoding `Vec<u8>`.
#[cfg(feature = "alloc")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeDynSizedBytesError {
    /// Non-hexadecimal character.
    InvalidChar(InvalidCharError),
    /// Purported hex string had odd (not even) length.
    OddLengthString(OddLengthStringError),
}

#[cfg(feature = "alloc")]
impl DecodeDynSizedBytesError {
    /// Adds `by_bytes` to all character positions stored inside.
    ///
    /// If you're parsing a larger string that consists of multiple hex sub-strings and want to
    /// return `InvalidCharError` you may need to use this function so that the callers of your
    /// parsing function can tell the exact position where decoding failed relative to the start of
    /// the string passed into your parsing function.
    ///
    /// Note that this function has the standard Rust overflow behavior because you should only
    /// ever pass in the position of the parsed hex string relative to the start of the entire
    /// input. In that case overflow is impossible.
    ///
    /// This method is specifically designed to be used with [`map_err`](Result::map_err) method of 
    /// [`Result`].
    #[inline]
    pub fn offset(self, by_bytes: usize) -> Self {
        match self {
            DecodeDynSizedBytesError::InvalidChar(error) => {
                DecodeDynSizedBytesError::InvalidChar(error.offset(by_bytes))
            },
            DecodeDynSizedBytesError::OddLengthString(error) => {
                DecodeDynSizedBytesError::OddLengthString(error)
            },
        }
    }
}

#[cfg(feature = "alloc")]
impl From<Infallible> for DecodeDynSizedBytesError {
    #[inline]
    fn from(never: Infallible) -> Self { match never {} }
}

#[cfg(feature = "alloc")]
impl fmt::Display for DecodeDynSizedBytesError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use DecodeDynSizedBytesError as E;

        match *self {
            E::InvalidChar(ref e) =>
                write_err!(f, "failed to decode hex"; e),
            E::OddLengthString(ref e) =>
                write_err!(f, "failed to decode hex"; e),
        }
    }
}

#[cfg(feature = "alloc")]
if_std_error! {{
    impl StdError for DecodeDynSizedBytesError {
        fn source(&self) -> Option<&(dyn StdError + 'static)> {
            use DecodeDynSizedBytesError as E;

            match *self {
                E::InvalidChar(ref e) => Some(e),
                E::OddLengthString(ref e) => Some(e),
            }
        }
    }
}}

#[cfg(feature = "alloc")]
impl From<InvalidCharError> for DecodeDynSizedBytesError {
    #[inline]
    fn from(e: InvalidCharError) -> Self { Self::InvalidChar(e) }
}

#[cfg(feature = "alloc")]
impl From<OddLengthStringError> for DecodeDynSizedBytesError {
    #[inline]
    fn from(e: OddLengthStringError) -> Self { Self::OddLengthString(e) }
}

/// Error returned when hex decoding bytes whose length is known at compile time.
///
/// This error differs from `DecodeDynSizedBytesError` in that the number of bytes is known at
/// compile time - e.g. when decoding to an array of bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeFixedSizedBytesError {
    /// Non-hexadecimal character.
    InvalidChar(InvalidCharError),
    /// Tried to parse fixed-length hash from a string with the wrong length.
    InvalidLength(InvalidLengthError),
}

impl DecodeFixedSizedBytesError {
    /// Adds `by_bytes` to all character positions stored inside.
    ///
    /// If you're parsing a larger string that consists of multiple hex sub-strings and want to
    /// return `InvalidCharError` you may need to use this function so that the callers of your
    /// parsing function can tell the exact position where decoding failed relative to the start of
    /// the string passed into your parsing function.
    ///
    /// Note that this function has the standard Rust overflow behavior because you should only
    /// ever pass in the position of the parsed hex string relative to the start of the entire
    /// input. In that case overflow is impossible.
    ///
    /// This method is specifically designed to be used with [`map_err`](Result::map_err) method of 
    /// [`Result`].
    #[inline]
    pub fn offset(self, by_bytes: usize) -> Self {
        match self {
            DecodeFixedSizedBytesError::InvalidChar(error) => {
                DecodeFixedSizedBytesError::InvalidChar(error.offset(by_bytes))
            },
            DecodeFixedSizedBytesError::InvalidLength(error) => {
                DecodeFixedSizedBytesError::InvalidLength(error)
            },
        }
    }
}

impl From<Infallible> for DecodeFixedSizedBytesError {
    #[inline]
    fn from(never: Infallible) -> Self { match never {} }
}

impl fmt::Display for DecodeFixedSizedBytesError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use DecodeFixedSizedBytesError as E;

        match *self {
            E::InvalidChar(ref e) => write_err!(f, "failed to parse hex digit"; e),
            E::InvalidLength(ref e) => write_err!(f, "failed to parse hex"; e),
        }
    }
}

if_std_error! {{
    impl StdError for DecodeFixedSizedBytesError {
        fn source(&self) -> Option<&(dyn StdError + 'static)> {
            use DecodeFixedSizedBytesError as E;

            match *self {
                E::InvalidChar(ref e) => Some(e),
                E::InvalidLength(ref e) => Some(e),
            }
        }
    }
}}

impl From<InvalidCharError> for DecodeFixedSizedBytesError {
    #[inline]
    fn from(e: InvalidCharError) -> Self { Self::InvalidChar(e) }
}

impl From<InvalidLengthError> for DecodeFixedSizedBytesError {
    #[inline]
    fn from(e: InvalidLengthError) -> Self { Self::InvalidLength(e) }
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
    // We do not expose this because we want to eventually return a `char`.
    // https://github.com/rust-bitcoin/hex-conservative/issues/100
    pub(crate) fn invalid_char(&self) -> u8 { self.invalid }
    /// Returns the position of the invalid character byte.
    #[inline]
    pub fn pos(&self) -> usize { self.pos }

    /// Adds `by_bytes` to all character positions stored inside.
    ///
    /// **Important**: if you have `DecodeDynSizedBytesError` or `DecodeFixedSizedBytesError` you
    /// should call the method *on them* - do not match them and manually call this method. Doing
    /// so may lead to broken behavior in the future.
    ///
    /// If you're parsing a larger string that consists of multiple hex sub-strings and want to
    /// return `InvalidCharError` you may need to use this function so that the callers of your
    /// parsing function can tell the exact position where decoding failed relative to the start of
    /// the string passed into your parsing function.
    ///
    /// Note that this function has the standard Rust overflow behavior because you should only
    /// ever pass in the position of the parsed hex string relative to the start of the entire
    /// input. In that case overflow is impossible.
    ///
    /// This method is specifically designed to be used with [`map_err`](Result::map_err) method of 
    /// [`Result`].
    #[inline]
    pub fn offset(mut self, by_bytes: usize) -> Self {
        self.pos += by_bytes;
        self
    }
}

impl fmt::Display for InvalidCharError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid hex char {} at pos {}", self.invalid_char(), self.pos())
    }
}

if_std_error! {{
    impl StdError for InvalidCharError {}
}}

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
        write!(f, "the hex string is {} bytes long which is not an even number", self.length())
    }
}

if_std_error! {{
    impl StdError for OddLengthStringError {}
}}

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
    ///
    /// Note that this represents both the number of bytes and the number of characters that needs
    /// to be passed into the decoder, since the hex digits are ASCII and thus always 1-byte long.
    #[inline]
    pub fn expected_length(&self) -> usize { self.expected }

    /// Returns the number of *hex bytes* passed to the hex decoder.
    ///
    /// Note that this does not imply the number of characters nor hex digits since they may be
    /// invalid (wide unicode chars).
    #[inline]
    pub fn invalid_length(&self) -> usize { self.invalid }
}

impl fmt::Display for InvalidLengthError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "the hex string is {} bytes long but exactly {} bytes was required",
            self.invalid_length(),
            self.expected_length()
        )
    }
}

if_std_error! {{
    impl StdError for InvalidLengthError {}
}}
