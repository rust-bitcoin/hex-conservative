// SPDX-License-Identifier: CC0-1.0

//! Hex encoding and decoding.

use core::{fmt, str};

#[cfg(all(feature = "alloc", not(feature = "std")))]
use crate::alloc::{string::String, vec::Vec};
use crate::error::InvalidLengthError;
use crate::iter::HexToBytesIter;
use crate::write_err;

#[rustfmt::skip]                // Keep public re-exports separate.
pub use crate::error::{HexToBytesError, HexToArrayError};

/// Trait for objects that can be deserialized from hex strings.
pub trait FromHex: Sized {
    /// Error type returned while parsing hex string.
    type Error: From<HexToBytesError> + Sized + fmt::Debug + fmt::Display;

    /// Produces an object from a byte iterator.
    fn from_byte_iter<I>(iter: I) -> Result<Self, Self::Error>
    where
        I: Iterator<Item = Result<u8, HexToBytesError>> + ExactSizeIterator + DoubleEndedIterator;

    /// Parses provided string as hex explicitly requiring there to not be a `0x` prefix.
    ///
    /// This is not recommended for user-supplied inputs because of possible confusion with decimals.
    /// It should be only used for existing protocols which always encode values as hex without 0x prefix.
    #[rustfmt::skip]
    fn from_hex<
        #[cfg(feature = "alloc")] S: AsRef<str> + Into<String>,
        #[cfg(not(feature = "alloc"))] S: AsRef<str>,
    >(s: S) -> Result<Self, Self::Error> {
        Self::from_no_prefix_hex(s)
    }

    /// Produces an object from a hex string.
    ///
    /// Accepts an input string either with `0x` prefix or without, if you require specific handling
    /// of the prefix see [`Self::from_prefixed_hex`] and [`Self::from_no_prefix_hex`].
    #[rustfmt::skip]
    fn from_maybe_prefixed_hex<
        #[cfg(feature = "alloc")] S: AsRef<str> + Into<String>,
        #[cfg(not(feature = "alloc"))] S: AsRef<str>,
    >(s: S) -> Result<Self, Self::Error> {
        if s.as_ref().starts_with("0x") {
            Self::from_no_prefix_hex(s.as_ref().trim_start_matches("0x"))
        } else {
            Self::from_no_prefix_hex(s)
        }
    }

    /// Parses provided string as hex explicitly requiring there to not be a `0x` prefix.
    ///
    /// This is not recommended for user-supplied inputs because of possible confusion with decimals.
    /// It should be only used for existing protocols which always encode values as hex without 0x prefix.
    #[rustfmt::skip]
    fn from_no_prefix_hex<
        #[cfg(feature = "alloc")] S: AsRef<str> + Into<String>,
        #[cfg(not(feature = "alloc"))] S: AsRef<str>,
    >(s: S) -> Result<Self, Self::Error> {
        Self::from_byte_iter(HexToBytesIter::new(s.as_ref())?)
    }

    /// Parses provided string as hex requiring 0x prefix.
    ///
    /// This is intended for user-supplied inputs or already-existing protocols in which 0x prefix is used.
    #[rustfmt::skip]
    fn from_prefixed_hex<
        #[cfg(feature = "alloc")] S: AsRef<str> + Into<String>,
        #[cfg(not(feature = "alloc"))] S: AsRef<str>,
    >(s: S) -> Result<Self, PrefixedError<Self::Error>> {
        use PrefixedError::*;

        if !s.as_ref().starts_with("0x") {
            #[cfg(feature = "alloc")]
            return Err(MissingPrefix(MissingPrefixError(s.into())));
            #[cfg(not(feature = "alloc"))]
            return Err(MissingPrefix(MissingPrefixError()));
        } else {
            Ok(Self::from_no_prefix_hex(s.as_ref().trim_start_matches("0x"))?)
        }
    }
}

#[cfg(any(test, feature = "std", feature = "alloc"))]
impl FromHex for Vec<u8> {
    type Error = HexToBytesError;

    #[inline]
    fn from_byte_iter<I>(iter: I) -> Result<Self, Self::Error>
    where
        I: Iterator<Item = Result<u8, HexToBytesError>> + ExactSizeIterator + DoubleEndedIterator,
    {
        iter.collect()
    }
}

macro_rules! impl_fromhex_array {
    ($len:expr) => {
        impl FromHex for [u8; $len] {
            type Error = HexToArrayError;

            fn from_byte_iter<I>(iter: I) -> Result<Self, Self::Error>
            where
                I: Iterator<Item = Result<u8, HexToBytesError>>
                    + ExactSizeIterator
                    + DoubleEndedIterator,
            {
                if iter.len() == $len {
                    let mut ret = [0; $len];
                    for (n, byte) in iter.enumerate() {
                        ret[n] = byte?;
                    }
                    Ok(ret)
                } else {
                    Err(InvalidLengthError { expected: 2 * $len, got: 2 * iter.len() }.into())
                }
            }
        }
    };
}

impl_fromhex_array!(2);
impl_fromhex_array!(4);
impl_fromhex_array!(6);
impl_fromhex_array!(8);
impl_fromhex_array!(10);
impl_fromhex_array!(12);
impl_fromhex_array!(14);
impl_fromhex_array!(16);
impl_fromhex_array!(20);
impl_fromhex_array!(24);
impl_fromhex_array!(28);
impl_fromhex_array!(32);
impl_fromhex_array!(33);
impl_fromhex_array!(64);
impl_fromhex_array!(65);
impl_fromhex_array!(128);
impl_fromhex_array!(256);
impl_fromhex_array!(384);
impl_fromhex_array!(512);

/// Hex parsing error
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum PrefixedError<E> {
    /// The input was not a valid hex string, contains the error that occurred while parsing.
    ParseHex(E),
    /// The input is missing `0x` prefix, contains the invalid input.
    MissingPrefix(MissingPrefixError),
}

impl<E> From<E> for PrefixedError<E> {
    fn from(e: E) -> Self { PrefixedError::ParseHex(e) }
}

impl<E: fmt::Display> fmt::Display for PrefixedError<E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use PrefixedError::*;

        match *self {
            ParseHex(ref e) => write_err!(f, "failed to parse hex string"; e),
            MissingPrefix(ref e) => write_err!(f, "missing prefix"; e),
        }
    }
}

#[cfg(feature = "std")]
impl<E> std::error::Error for PrefixedError<E>
where
    E: std::error::Error + 'static,
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        use PrefixedError::*;

        match *self {
            ParseHex(ref e) => Some(e),
            MissingPrefix(ref e) => Some(e),
        }
    }
}

/// Hex string was missing the `0x` prefix.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissingPrefixError(#[cfg(feature = "alloc")] String);

impl fmt::Display for MissingPrefixError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        #[cfg(feature = "alloc")]
        let res = write!(f, "the input value `{}` is missing the `0x` prefix", self.0);
        #[cfg(not(feature = "alloc"))]
        let res = write!(f, "input string is missing the `0x` prefix");

        res
    }
}

#[cfg(feature = "std")]
impl std::error::Error for MissingPrefixError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::display::DisplayHex;
    use crate::error::InvalidLengthError;

    #[test]
    #[cfg(feature = "alloc")]
    fn hex_error() {
        use crate::error::{InvalidCharError, OddLengthStringError};

        let oddlen = "0123456789abcdef0";
        let badchar1 = "Z123456789abcdef";
        let badchar2 = "012Y456789abcdeb";
        let badchar3 = "«23456789abcdef";

        assert_eq!(Vec::<u8>::from_hex(oddlen), Err(OddLengthStringError { len: 17 }.into()));
        assert_eq!(
            <[u8; 4]>::from_hex(oddlen),
            Err(HexToBytesError::OddLengthString(OddLengthStringError { len: 17 }).into())
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
