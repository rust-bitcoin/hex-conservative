// SPDX-License-Identifier: CC0-1.0

//! Hex encoding and decoding.

use core::{fmt, str};

#[cfg(all(feature = "alloc", not(feature = "std")))]
use crate::alloc::vec::Vec;
use crate::error::InvalidLengthError;
use crate::iter::HexToBytesIter;

#[rustfmt::skip]                // Keep public re-exports separate.
pub use crate::error::{HexToBytesError, HexToArrayError};

/// Trait for objects that can be deserialized from hex strings.
pub trait FromHex: Sized {
    /// Error type returned while parsing hex string.
    type Error: Sized + fmt::Debug + fmt::Display;

    /// Produces an object from a hex string.
    fn from_hex(s: &str) -> Result<Self, Self::Error>;
}

#[cfg(any(test, feature = "std", feature = "alloc"))]
impl FromHex for Vec<u8> {
    type Error = HexToBytesError;

    fn from_hex(s: &str) -> Result<Self, Self::Error> {
        Ok(HexToBytesIter::new(s)?.drain_to_vec()?)
    }
}

impl<const LEN: usize> FromHex for [u8; LEN] {
    type Error = HexToArrayError;

    fn from_hex(s: &str) -> Result<Self, Self::Error> {
        if s.len() == LEN * 2 {
            let mut ret = [0u8; LEN];
            // checked above
            HexToBytesIter::new_unchecked(s).drain_to_slice(&mut ret)?;
            Ok(ret)
        } else {
            Err(InvalidLengthError { invalid: s.len(), expected: 2 * LEN }.into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::display::DisplayHex;

    #[test]
    #[cfg(feature = "alloc")]
    fn hex_error() {
        let oddlen = "0123456789abcdef0";
        let badchar1 = "Z123456789abcdef";
        let badchar2 = "012Y456789abcdeb";
        let badchar3 = "«23456789abcdef";

        match Vec::<u8>::from_hex(oddlen) {
            Err(HexToBytesError::OddLengthString(e)) => assert_eq!(e.length(), 17),
            v => panic!("Wrong return value: {:?}", v),
        }

        assert_eq!(
            <[u8; 4]>::from_hex(oddlen),
            Err(InvalidLengthError { invalid: 17, expected: 8 }.into())
        );

        match Vec::<u8>::from_hex(badchar1) {
            Err(HexToBytesError::InvalidChar(e)) => {
                assert_eq!(e.pos(), 0);
                assert_eq!(e.char(), Ok('Z'));
            }
            v => panic!("Wrong return value: {:?}", v),
        }

        match Vec::<u8>::from_hex(badchar2) {
            Err(HexToBytesError::InvalidChar(e)) => {
                assert_eq!(e.pos(), 3);
                assert_eq!(e.char(), Ok('Y'));
            }
            v => panic!("Wrong return value: {:?}", v),
        }

        match Vec::<u8>::from_hex(badchar3) {
            Err(HexToBytesError::InvalidChar(e)) => {
                assert_eq!(e.pos(), 0);
                assert_eq!(e.char(), Err(194));
            }
            v => panic!("Wrong return value: {:?}", v),
        }
    }

    #[test]
    fn hex_error_position() {
        let badpos1 = "Z123456789abcdef";
        let badpos2 = "012Y456789abcdeb";
        let badpos3 = "0123456789abcdeZ";
        let badpos4 = "0123456789abYdef";

        match Vec::<u8>::from_hex(badpos1) {
            Err(HexToBytesError::InvalidChar(e)) => {
                assert_eq!(e.pos(), 0);
                assert_eq!(e.char(), Ok('Z'));
            }
            v => panic!("Wrong return value: {:?}", v),
        };

        match Vec::<u8>::from_hex(badpos2) {
            Err(HexToBytesError::InvalidChar(e)) => {
                assert_eq!(e.pos(), 3);
                assert_eq!(e.char(), Ok('Y'));
            }
            v => panic!("Wrong return value: {:?}", v),
        };

        match Vec::<u8>::from_hex(badpos3) {
            Err(HexToBytesError::InvalidChar(e)) => {
                assert_eq!(e.pos(), 15);
                assert_eq!(e.char(), Ok('Z'));
            }
            v => panic!("Wrong return value: {:?}", v),
        };

        match Vec::<u8>::from_hex(badpos4) {
            Err(HexToBytesError::InvalidChar(e)) => {
                assert_eq!(e.pos(), 12);
                assert_eq!(e.char(), Ok('Y'));
            }
            v => panic!("Wrong return value: {:?}", v),
        };
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
            Err(InvalidLengthError { invalid: 16, expected: 8 }.into())
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
