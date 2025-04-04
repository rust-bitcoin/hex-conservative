// SPDX-License-Identifier: CC0-1.0

//! Hex encoding and decoding.

use core::{fmt, str};

#[cfg(all(feature = "alloc", not(feature = "std")))]
use crate::alloc::vec::Vec;

#[rustfmt::skip]                // Keep public re-exports separate.
pub use crate::error::{HexToBytesError, HexToArrayError};

/// Trait for objects that can be deserialized from hex strings.
pub trait FromHex: Sized + sealed::Sealed {
    /// Error type returned while parsing hex string.
    type Error: Sized + fmt::Debug + fmt::Display;

    /// Produces an object from a hex string.
    ///
    /// # Errors
    ///
    /// Errors if parsing of hex string fails for any reason.
    fn from_hex(s: &str) -> Result<Self, Self::Error>;
}

#[cfg(feature = "alloc")]
impl FromHex for Vec<u8> {
    type Error = HexToBytesError;

    #[inline]
    fn from_hex(s: &str) -> Result<Self, Self::Error> { crate::decode_to_vec(s) }
}

impl<const LEN: usize> FromHex for [u8; LEN] {
    type Error = HexToArrayError;

    fn from_hex(s: &str) -> Result<Self, Self::Error> { crate::decode_to_array(s) }
}

mod sealed {
    /// Used to seal the `FromHex` trait.
    pub trait Sealed {}

    #[cfg(feature = "alloc")]
    impl Sealed for alloc::vec::Vec<u8> {}

    impl<const LEN: usize> Sealed for [u8; LEN] {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{HexToBytesIter, InvalidLengthError};

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
        );
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn mixed_case() {
        use crate::display::DisplayHex as _;

        let s = "DEADbeef0123";
        let want_lower = "deadbeef0123";
        let want_upper = "DEADBEEF0123";

        let v = Vec::<u8>::from_hex(s).expect("valid hex");
        assert_eq!(format!("{:x}", v.as_hex()), want_lower);
        assert_eq!(format!("{:X}", v.as_hex()), want_upper);
    }
}
