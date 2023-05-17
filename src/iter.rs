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
    /// The [`Bytes`] iterator whose next two bytes will be decoded to yield the next byte.
    ///
    /// # Invariants
    ///
    /// `iter` is guaranteed to be of even length.
    ///
    /// [`Bytes`]: core::str::Bytes
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

impl<'a> Iterator for HexToBytesIter<'a> {
    type Item = Result<u8, HexToBytesError>;

    fn next(&mut self) -> Option<Result<u8, HexToBytesError>> {
        let hi = self.iter.next()?;
        let lo = self.iter.next().expect("iter length invariant violated, this is a bug");
        Some(hex_chars_to_byte(hi, lo))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (min, max) = self.iter.size_hint();
        (min / 2, max.map(|x| x / 2))
    }
}

impl<'a> DoubleEndedIterator for HexToBytesIter<'a> {
    fn next_back(&mut self) -> Option<Result<u8, HexToBytesError>> {
        let lo = self.iter.next_back()?;
        let hi = self.iter.next_back().expect("iter length invariant violated, this is a bug");
        Some(hex_chars_to_byte(hi, lo))
    }
}

impl<'a> ExactSizeIterator for HexToBytesIter<'a> {
    fn len(&self) -> usize { self.iter.len() / 2 }
}

impl<'a> core::iter::FusedIterator for HexToBytesIter<'a> {}

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

/// `hi` and `lo` are bytes representing hex characters.
fn hex_chars_to_byte(hi: u8, lo: u8) -> Result<u8, HexToBytesError> {
    let hih = (hi as char).to_digit(16).ok_or(HexToBytesError::InvalidChar(hi))?;
    let loh = (lo as char).to_digit(16).ok_or(HexToBytesError::InvalidChar(lo))?;

    let ret = (hih << 4) + loh;
    Ok(ret as u8)
}
