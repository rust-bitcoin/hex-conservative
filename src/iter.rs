// SPDX-License-Identifier: CC0-1.0

//! Iterator that converts hex to bytes.

use core::str;
#[cfg(feature = "std")]
use std::io;

#[cfg(all(feature = "core2", not(feature = "std")))]
use core2::io;

use crate::parse::HexToBytesError;

/// Iterator over a hex-encoded string slice which decodes hex and yields bytes.
pub struct HexToBytesIter<'a> {
    /// The `Bytes` iterator whose next two bytes will be decoded to yield
    /// the next byte.
    iter: str::Bytes<'a>,
}

impl<'a> HexToBytesIter<'a> {
    /// Constructs a new `HexToBytesIter` from a string slice.
    ///
    /// # Errors
    ///
    /// If the input string is of odd length.
    pub fn new(s: &'a str) -> Result<HexToBytesIter<'a>, HexToBytesError> {
        if s.len() % 2 != 0 {
            Err(HexToBytesError::OddLengthString(s.len()))
        } else {
            Ok(HexToBytesIter { iter: s.bytes() })
        }
    }
}

fn chars_to_hex(hi: u8, lo: u8) -> Result<u8, HexToBytesError> {
    let hih = (hi as char).to_digit(16).ok_or(HexToBytesError::InvalidChar(hi))?;
    let loh = (lo as char).to_digit(16).ok_or(HexToBytesError::InvalidChar(lo))?;

    let ret = (hih << 4) + loh;
    Ok(ret as u8)
}

impl<'a> Iterator for HexToBytesIter<'a> {
    type Item = Result<u8, HexToBytesError>;

    fn next(&mut self) -> Option<Result<u8, HexToBytesError>> {
        let hi = self.iter.next()?;
        let lo = self.iter.next().unwrap();
        Some(chars_to_hex(hi, lo))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (min, max) = self.iter.size_hint();
        (min / 2, max.map(|x| x / 2))
    }
}

#[cfg(any(feature = "std", feature = "core2"))]
impl<'a> io::Read for HexToBytesIter<'a> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut bytes_read = 0usize;
        for dst in buf {
            match self.next() {
                Some(Ok(src)) => {
                    *dst = src;
                    bytes_read += 1;
                }
                _ => break,
            }
        }
        Ok(bytes_read)
    }
}

impl<'a> DoubleEndedIterator for HexToBytesIter<'a> {
    fn next_back(&mut self) -> Option<Result<u8, HexToBytesError>> {
        let lo = self.iter.next_back()?;
        let hi = self.iter.next_back().unwrap();
        Some(chars_to_hex(hi, lo))
    }
}

impl<'a> ExactSizeIterator for HexToBytesIter<'a> {}

impl<'a> core::iter::FusedIterator for HexToBytesIter<'a> {}
