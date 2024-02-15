// SPDX-License-Identifier: CC0-1.0

//! Supports parsing integer types as hex.

use crate::error::InvalidCharError;
use crate::iter::HexToBytesIter;
use crate::prelude::*;

macro_rules! impl_from_hex_for_int {
    ($ty:ty, $len:literal) => {
        impl FromHex for $ty {
            type FromByteIterError = InvalidCharError;
            type FromHexError = InvalidCharError;

            #[inline]
            fn from_byte_iter<I>(iter: I) -> Result<Self, Self::FromByteIterError>
            where
                I: Iterator<Item = Result<u8, InvalidCharError>>
                    + ExactSizeIterator
                    + DoubleEndedIterator,
            {
                let mut buf = [0_u8; $len];
                for (i, byte) in iter.rev().enumerate() {
                    let index = $len - 1 - i;
                    buf[index] = byte?;
                }
                Ok(<$ty>::from_be_bytes(buf))
            }

            #[inline]
            #[rustfmt::skip]
            fn from_hex(s: &str) -> Result<Self, Self::FromHexError> {
                let s = if s.starts_with("0x") || s.starts_with("0X") { &s[2..] } else { s };
                let iter = HexToBytesIter::new(s);
                Self::from_byte_iter(iter)
            }
        }
    };
}
impl_from_hex_for_int!(u8, 1);
impl_from_hex_for_int!(u16, 2);
impl_from_hex_for_int!(u32, 4);
impl_from_hex_for_int!(u64, 8);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_u8() {
        assert_eq!(u8::from_hex("1").expect("failed to parse u8"), 1);
    }

    #[test]
    fn basic_u16() {
        assert_eq!(u16::from_hex("1").expect("failed to parse u16"), 1);
    }

    #[test]
    fn basic_u32() {
        assert_eq!(u32::from_hex("1").expect("failed to parse u32"), 1);
    }

    #[test]
    fn basic_u64() {
        assert_eq!(u64::from_hex("1").expect("failed to parse u64"), 1);
    }

    macro_rules! check_u32_hex {
        ($($test_name:ident, $hex:literal, $expected:literal);* $(;)?) => {
            $(
                #[test]
                fn $test_name() {
                    assert_eq!(u32::from_hex($hex).expect("failed to parse hex"), $expected)
                }
            )*
        }
    }
    check_u32_hex! {
        check_u32_0, "1", 1;
        check_u32_1, "01", 1;
        check_u32_2, "0x1", 1;
        check_u32_3, "0x01", 1;
        check_u32_4, "0xdeadbeef", 3735928559;
        check_u32_5, "deadbeef", 3735928559;
        check_u32_6, "DEADBEEF", 3735928559;
        check_u32_7, "0XDEADBEEF", 3735928559;
    }
}
