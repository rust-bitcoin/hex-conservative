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
    fn hex_error_ascii() {
        use crate::error::{InvalidCharError, OddLengthStringError};

        let oddlen = "0123456789abcdef0";
        let badchar1 = "Z123456789abcdef";
        let badchar2 = "012Y456789abcdeb";

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
            InvalidCharError { pos: 0, invalid: 'Z' }.into()
        );
        assert_eq!(
            Vec::<u8>::from_hex(badchar2).unwrap_err(),
            InvalidCharError { pos: 3, invalid: 'Y' }.into()
        );
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn hex_error_non_ascii() {
        use crate::error::{InvalidCharError, OddLengthStringError};

        // These are for sanity and documentation purposes.
        assert_eq!("«".len(), 2); // 0xC2 0xAB
        assert_eq!("✓".len(), 3); // 0xE2 0x9C 0x93
        assert_eq!("𓃾".len(), 4); // 0xF0 0x93 0x83 0xbe
        assert_eq!("0123456789abcdef".len(), 16);

        let badchar = "0123456789abcde«";
        assert_eq!(badchar.len(), 17);
        assert_eq!(
            Vec::<u8>::from_hex(badchar).unwrap_err(),
            OddLengthStringError { len: 17 }.into(),
        );

        // I would have thought this was length 1.
        let badchar = "«";
        assert_eq!(badchar.len(), 2); // Sanity check.
        assert_eq!(
            Vec::<u8>::from_hex(badchar).unwrap_err(),
            InvalidCharError { pos: 0, invalid: '«' }.into()
        );

        let badchar = "«✓a";
        assert_eq!(badchar.len(), 6); // Sanity check.
        assert_eq!(
            Vec::<u8>::from_hex(badchar).unwrap_err(),
            InvalidCharError { pos: 0, invalid: '«' }.into()
        );

        let badchar = "0123456789abcd«";
        assert_eq!(badchar.len(), 16); // Sanity check.
        assert_eq!(
            Vec::<u8>::from_hex(badchar).unwrap_err(),
            InvalidCharError { pos: 14, invalid: '«' }.into()
        );

        let badchar = "0123456789a«✓";
        assert_eq!(badchar.len(), 16); // Sanity check.
        assert_eq!(
            Vec::<u8>::from_hex(badchar).unwrap_err(),
            InvalidCharError { pos: 11, invalid: '«' }.into()
        );

        let badchar = "«0123456789abcd";
        assert_eq!(badchar.len(), 16); // Sanity check.
        assert_eq!(
            Vec::<u8>::from_hex(badchar).unwrap_err(),
            InvalidCharError { pos: 0, invalid: '«' }.into()
        );

        let badchar = "«✓56789abcdef";
        assert_eq!(badchar.len(), 16); // Sanity check.
        assert_eq!(
            Vec::<u8>::from_hex(badchar).unwrap_err(),
            InvalidCharError { pos: 0, invalid: '«' }.into()
        );

        let badchar = "0123456789abc✓";
        assert_eq!(badchar.len(), 16); // Sanity check.
        assert_eq!(
            Vec::<u8>::from_hex(badchar).unwrap_err(),
            InvalidCharError { pos: 13, invalid: '✓' }.into()
        );

        let badchar = "✓3456789abcdef";
        assert_eq!(badchar.len(), 16); // Sanity check.
        assert_eq!(
            Vec::<u8>::from_hex(badchar).unwrap_err(),
            InvalidCharError { pos: 0, invalid: '✓' }.into()
        );

        let badchar = "✓«56789abcdef";
        assert_eq!(badchar.len(), 16); // Sanity check.
        assert_eq!(
            Vec::<u8>::from_hex(badchar).unwrap_err(),
            InvalidCharError { pos: 0, invalid: '✓' }.into()
        );

        let badchar = "0123456789ab𓃾";
        assert_eq!(badchar.len(), 16); // Sanity check.
        assert_eq!(
            Vec::<u8>::from_hex(badchar).unwrap_err(),
            InvalidCharError { pos: 12, invalid: '𓃾' }.into()
        );
        let badchar = "𓃾456789abcdef";
        assert_eq!(badchar.len(), 16); // Sanity check.
        assert_eq!(
            Vec::<u8>::from_hex(badchar).unwrap_err(),
            InvalidCharError { pos: 0, invalid: '𓃾' }.into()
        );
        let badchar = "01𓃾6789abcdef";
        assert_eq!(badchar.len(), 16); // Sanity check.
        assert_eq!(
            Vec::<u8>::from_hex(badchar).unwrap_err(),
            InvalidCharError { pos: 2, invalid: '𓃾' }.into()
        );

        // Can't handle 4 byte encoded character that is in an odd position.
        // let badchar = "0123456789a𓃾f";
        // assert_eq!(badchar.len(), 16); // Sanity check.
        // assert_eq!(
        //     Vec::<u8>::from_hex(badchar).unwrap_err(),
        //     InvalidCharError { pos: 11, invalid: '𓃾' }.into()
        // );
        // let badchar = "0𓃾456789abcde";
        // assert_eq!(badchar.len(), 16); // Sanity check.
        // assert_eq!(
        //     Vec::<u8>::from_hex(badchar).unwrap_err(),
        //     InvalidCharError { pos: 1, invalid: '𓃾' }.into()
        // );
    }

    #[test]
    fn hex_error_position() {
        use crate::error::InvalidDigitError;
        let badpos1 = "Z123456789abcdef";
        let badpos2 = "012Y456789abcdeb";
        let badpos3 = "0123456789abcdeZ";
        let badpos4 = "0123456789abYdef";

        assert_eq!(
            HexToBytesIter::new(badpos1).unwrap().next().unwrap().unwrap_err(),
            InvalidDigitError { hi: b'Z', lo: b'1', is_hi: true, pos: 0 }
        );
        assert_eq!(
            HexToBytesIter::new(badpos2).unwrap().nth(1).unwrap().unwrap_err(),
            InvalidDigitError { hi: b'2', lo: b'Y', is_hi: false, pos: 2 }
        );
        assert_eq!(
            HexToBytesIter::new(badpos3).unwrap().next_back().unwrap().unwrap_err(),
            InvalidDigitError { hi: b'e', lo: b'Z', is_hi: false, pos: 14 }
        );
        assert_eq!(
            HexToBytesIter::new(badpos4).unwrap().nth_back(1).unwrap().unwrap_err(),
            InvalidDigitError { hi: b'Y', lo: b'd', is_hi: true, pos: 12 }
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
