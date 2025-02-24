// SPDX-License-Identifier: CC0-1.0

//! Iterator that converts hex to bytes.

use core::borrow::Borrow;
use core::convert::TryInto;
use core::iter::FusedIterator;
use core::str;
#[cfg(feature = "std")]
use std::io;

#[cfg(all(feature = "alloc", not(feature = "std")))]
use crate::alloc::vec::Vec;
use crate::error::{InvalidCharError, OddLengthStringError};
use crate::{Case, Table};

/// Convenience alias for `HexToBytesIter<HexDigitsIter<'a>>`.
pub type HexSliceToBytesIter<'a> = HexToBytesIter<HexDigitsIter<'a>>;

/// Iterator yielding bytes decoded from an iterator of pairs of hex digits.
#[derive(Debug)]
pub struct HexToBytesIter<I>
where
    I: Iterator<Item = [u8; 2]>,
{
    iter: I,
    original_len: usize,
}

impl<'a> HexToBytesIter<HexDigitsIter<'a>> {
    /// Constructs a new `HexToBytesIter` from a string slice.
    ///
    /// # Errors
    ///
    /// If the input string is of odd length.
    #[inline]
    pub fn new(s: &'a str) -> Result<Self, OddLengthStringError> {
        if s.len() % 2 != 0 {
            Err(OddLengthStringError { len: s.len() })
        } else {
            Ok(Self::new_unchecked(s))
        }
    }

    pub(crate) fn new_unchecked(s: &'a str) -> Self {
        Self::from_pairs(HexDigitsIter::new_unchecked(s.as_bytes()))
    }

    /// Writes all the bytes yielded by this `HexToBytesIter` to the provided slice.
    ///
    /// Stops writing if this `HexToBytesIter` yields an `InvalidCharError`.
    ///
    /// # Panics
    ///
    /// Panics if the length of this `HexToBytesIter` is not equal to the length of the provided
    /// slice.
    pub(crate) fn drain_to_slice(self, buf: &mut [u8]) -> Result<(), InvalidCharError> {
        assert_eq!(self.len(), buf.len());
        let mut ptr = buf.as_mut_ptr();
        for byte in self {
            // SAFETY: for loop iterates `len` times, and `buf` has length `len`
            unsafe {
                core::ptr::write(ptr, byte?);
                ptr = ptr.add(1);
            }
        }
        Ok(())
    }

    /// Writes all the bytes yielded by this `HexToBytesIter` to a `Vec<u8>`.
    ///
    /// This is equivalent to the combinator chain `iter().map().collect()` but was found by
    /// benchmarking to be faster.
    #[cfg(any(test, feature = "std", feature = "alloc"))]
    pub(crate) fn drain_to_vec(self) -> Result<Vec<u8>, InvalidCharError> {
        let len = self.len();
        let mut ret = Vec::with_capacity(len);
        let mut ptr = ret.as_mut_ptr();
        for byte in self {
            // SAFETY: for loop iterates `len` times, and `ret` has a capacity of at least `len`
            unsafe {
                // docs: "`core::ptr::write` is appropriate for initializing uninitialized memory"
                core::ptr::write(ptr, byte?);
                ptr = ptr.add(1);
            }
        }
        // SAFETY: `len` elements have been initialized, and `ret` has a capacity of at least `len`
        unsafe {
            ret.set_len(len);
        }
        Ok(ret)
    }
}

impl<I> HexToBytesIter<I>
where
    I: Iterator<Item = [u8; 2]> + ExactSizeIterator,
{
    /// Constructs a custom hex decoding iterator from another iterator.
    #[inline]
    pub fn from_pairs(iter: I) -> Self { Self { original_len: iter.len(), iter } }
}

impl<I> Iterator for HexToBytesIter<I>
where
    I: Iterator<Item = [u8; 2]> + ExactSizeIterator,
{
    type Item = Result<u8, InvalidCharError>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let [hi, lo] = self.iter.next()?;
        Some(hex_chars_to_byte(hi, lo).map_err(|(c, is_high)| InvalidCharError {
            invalid: c,
            pos: if is_high {
                (self.original_len - self.iter.len() - 1) * 2
            } else {
                (self.original_len - self.iter.len() - 1) * 2 + 1
            },
        }))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) { self.iter.size_hint() }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let [hi, lo] = self.iter.nth(n)?;
        Some(hex_chars_to_byte(hi, lo).map_err(|(c, is_high)| InvalidCharError {
            invalid: c,
            pos: if is_high {
                (self.original_len - self.iter.len() - 1) * 2
            } else {
                (self.original_len - self.iter.len() - 1) * 2 + 1
            },
        }))
    }
}

impl<I> DoubleEndedIterator for HexToBytesIter<I>
where
    I: Iterator<Item = [u8; 2]> + DoubleEndedIterator + ExactSizeIterator,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let [hi, lo] = self.iter.next_back()?;
        Some(hex_chars_to_byte(hi, lo).map_err(|(c, is_high)| InvalidCharError {
            invalid: c,
            pos: if is_high { self.iter.len() * 2 } else { self.iter.len() * 2 + 1 },
        }))
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let [hi, lo] = self.iter.nth_back(n)?;
        Some(hex_chars_to_byte(hi, lo).map_err(|(c, is_high)| InvalidCharError {
            invalid: c,
            pos: if is_high { self.iter.len() * 2 } else { self.iter.len() * 2 + 1 },
        }))
    }
}

impl<I> ExactSizeIterator for HexToBytesIter<I> where I: Iterator<Item = [u8; 2]> + ExactSizeIterator
{}

impl<I> FusedIterator for HexToBytesIter<I> where
    I: Iterator<Item = [u8; 2]> + ExactSizeIterator + FusedIterator
{
}

#[cfg(feature = "std")]
impl<I> io::Read for HexToBytesIter<I>
where
    I: Iterator<Item = [u8; 2]> + ExactSizeIterator + FusedIterator,
{
    #[inline]
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

/// An internal iterator returning hex digits from a string.
///
/// Generally you shouldn't need to refer to this or bother with it and just use
/// [`HexToBytesIter::new`] consuming the returned value and use `HexSliceToBytesIter` if you need
/// to refer to the iterator in your types.
#[derive(Debug)]
pub struct HexDigitsIter<'a> {
    // Invariant: the length of the chunks is 2.
    // Technically, this is `iter::Map` but we can't use it because fn is anonymous.
    // We can swap this for actual `ArrayChunks` once it's stable.
    iter: core::slice::ChunksExact<'a, u8>,
}

impl<'a> HexDigitsIter<'a> {
    #[inline]
    fn new_unchecked(digits: &'a [u8]) -> Self { Self { iter: digits.chunks_exact(2) } }
}

impl Iterator for HexDigitsIter<'_> {
    type Item = [u8; 2];

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|digits| digits.try_into().expect("HexDigitsIter invariant"))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) { self.iter.size_hint() }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.iter.nth(n).map(|digits| digits.try_into().expect("HexDigitsIter invariant"))
    }
}

impl DoubleEndedIterator for HexDigitsIter<'_> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(|digits| digits.try_into().expect("HexDigitsIter invariant"))
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.iter.nth_back(n).map(|digits| digits.try_into().expect("HexDigitsIter invariant"))
    }
}

impl ExactSizeIterator for HexDigitsIter<'_> {}

impl core::iter::FusedIterator for HexDigitsIter<'_> {}

/// `hi` and `lo` are bytes representing hex characters.
///
/// Returns the valid byte or the invalid input byte and a bool indicating error for `hi` or `lo`.
fn hex_chars_to_byte(hi: u8, lo: u8) -> Result<u8, (u8, bool)> {
    let hih = (hi as char).to_digit(16).ok_or((hi, true))?;
    let loh = (lo as char).to_digit(16).ok_or((lo, false))?;

    let ret = (hih << 4) + loh;
    Ok(ret as u8)
}

/// Iterator over bytes which encodes the bytes and yields hex characters.
#[derive(Debug)]
pub struct BytesToHexIter<I>
where
    I: Iterator,
    I::Item: Borrow<u8>,
{
    /// The iterator whose next byte will be encoded to yield hex characters.
    iter: I,
    /// The low character of the pair (high, low) of hex characters encoded per byte.
    low: Option<char>,
    /// The byte-to-hex conversion table.
    table: &'static Table,
}

impl<I> BytesToHexIter<I>
where
    I: Iterator,
    I::Item: Borrow<u8>,
{
    /// Constructs a `BytesToHexIter` that will yield hex characters in the given case from a byte
    /// iterator.
    pub fn new(iter: I, case: Case) -> BytesToHexIter<I> {
        Self { iter, low: None, table: case.table() }
    }
}

impl<I> Iterator for BytesToHexIter<I>
where
    I: Iterator,
    I::Item: Borrow<u8>,
{
    type Item = char;

    #[inline]
    fn next(&mut self) -> Option<char> {
        match self.low {
            Some(c) => {
                self.low = None;
                Some(c)
            }
            None => self.iter.next().map(|b| {
                let [high, low] = self.table.byte_to_chars(*b.borrow());
                self.low = Some(low);
                high
            }),
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (min, max) = self.iter.size_hint();
        match self.low {
            Some(_) => (min * 2 + 1, max.map(|max| max * 2 + 1)),
            None => (min * 2, max.map(|max| max * 2)),
        }
    }
}

impl<I> DoubleEndedIterator for BytesToHexIter<I>
where
    I: DoubleEndedIterator,
    I::Item: Borrow<u8>,
{
    #[inline]
    fn next_back(&mut self) -> Option<char> {
        match self.low {
            Some(c) => {
                self.low = None;
                Some(c)
            }
            None => self.iter.next_back().map(|b| {
                let [high, low] = self.table.byte_to_chars(*b.borrow());
                self.low = Some(low);
                high
            }),
        }
    }
}

impl<I> ExactSizeIterator for BytesToHexIter<I>
where
    I: ExactSizeIterator,
    I::Item: Borrow<u8>,
{
    #[inline]
    fn len(&self) -> usize { self.iter.len() * 2 }
}

impl<I> FusedIterator for BytesToHexIter<I>
where
    I: FusedIterator,
    I::Item: Borrow<u8>,
{
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_byte() {
        assert_eq!(Table::LOWER.byte_to_chars(0x00), ['0', '0']);
        assert_eq!(Table::LOWER.byte_to_chars(0x0a), ['0', 'a']);
        assert_eq!(Table::LOWER.byte_to_chars(0xad), ['a', 'd']);
        assert_eq!(Table::LOWER.byte_to_chars(0xff), ['f', 'f']);

        assert_eq!(Table::UPPER.byte_to_chars(0x00), ['0', '0']);
        assert_eq!(Table::UPPER.byte_to_chars(0x0a), ['0', 'A']);
        assert_eq!(Table::UPPER.byte_to_chars(0xad), ['A', 'D']);
        assert_eq!(Table::UPPER.byte_to_chars(0xff), ['F', 'F']);

        let mut buf = [0u8; 2];
        assert_eq!(Table::LOWER.byte_to_str(&mut buf, 0x00), "00");
        assert_eq!(Table::LOWER.byte_to_str(&mut buf, 0x0a), "0a");
        assert_eq!(Table::LOWER.byte_to_str(&mut buf, 0xad), "ad");
        assert_eq!(Table::LOWER.byte_to_str(&mut buf, 0xff), "ff");

        assert_eq!(Table::UPPER.byte_to_str(&mut buf, 0x00), "00");
        assert_eq!(Table::UPPER.byte_to_str(&mut buf, 0x0a), "0A");
        assert_eq!(Table::UPPER.byte_to_str(&mut buf, 0xad), "AD");
        assert_eq!(Table::UPPER.byte_to_str(&mut buf, 0xff), "FF");
    }

    #[test]
    fn decode_iter_forward() {
        let hex = "deadbeef";
        let bytes = [0xde, 0xad, 0xbe, 0xef];

        for (i, b) in HexToBytesIter::new(hex).unwrap().enumerate() {
            assert_eq!(b.unwrap(), bytes[i]);
        }

        let mut iter = HexToBytesIter::new(hex).unwrap();
        for i in (0..=bytes.len()).rev() {
            assert_eq!(iter.len(), i);
            let _ = iter.next();
        }
    }

    #[test]
    fn decode_iter_backward() {
        let hex = "deadbeef";
        let bytes = [0xef, 0xbe, 0xad, 0xde];

        for (i, b) in HexToBytesIter::new(hex).unwrap().rev().enumerate() {
            assert_eq!(b.unwrap(), bytes[i]);
        }

        let mut iter = HexToBytesIter::new(hex).unwrap().rev();
        for i in (0..=bytes.len()).rev() {
            assert_eq!(iter.len(), i);
            let _ = iter.next();
        }
    }

    #[test]
    fn hex_to_digits_size_hint() {
        let hex = "deadbeef";
        let iter = HexDigitsIter::new_unchecked(hex.as_bytes());
        // HexDigitsIter yields two digits at a time `[u8; 2]`.
        assert_eq!(iter.size_hint(), (4, Some(4)));
    }

    #[test]
    fn hex_to_bytes_size_hint() {
        let hex = "deadbeef";
        let iter = HexToBytesIter::new_unchecked(hex);
        assert_eq!(iter.size_hint(), (4, Some(4)));
    }

    #[test]
    fn hex_to_bytes_slice_drain() {
        let hex = "deadbeef";
        let want = [0xde, 0xad, 0xbe, 0xef];
        let iter = HexToBytesIter::new_unchecked(hex);
        let mut got = [0u8; 4];
        iter.drain_to_slice(&mut got).unwrap();
        assert_eq!(got, want);

        let hex = "";
        let want = [];
        let iter = HexToBytesIter::new_unchecked(hex);
        let mut got = [];
        iter.drain_to_slice(&mut got).unwrap();
        assert_eq!(got, want);
    }

    #[test]
    #[should_panic]
    fn hex_to_bytes_slice_drain_panic_empty() {
        let hex = "deadbeef";
        let iter = HexToBytesIter::new_unchecked(hex);
        let mut got = [];
        iter.drain_to_slice(&mut got).unwrap();
    }

    #[test]
    #[should_panic]
    fn hex_to_bytes_slice_drain_panic_too_small() {
        let hex = "deadbeef";
        let iter = HexToBytesIter::new_unchecked(hex);
        let mut got = [0u8; 3];
        iter.drain_to_slice(&mut got).unwrap();
    }

    #[test]
    #[should_panic]
    fn hex_to_bytes_slice_drain_panic_too_big() {
        let hex = "deadbeef";
        let iter = HexToBytesIter::new_unchecked(hex);
        let mut got = [0u8; 5];
        iter.drain_to_slice(&mut got).unwrap();
    }

    #[test]
    fn hex_to_bytes_slice_drain_first_char_error() {
        let hex = "geadbeef";
        let iter = HexToBytesIter::new_unchecked(hex);
        let mut got = [0u8; 4];
        assert_eq!(
            iter.drain_to_slice(&mut got).unwrap_err(),
            InvalidCharError { invalid: b'g', pos: 0 }
        );
    }

    #[test]
    fn hex_to_bytes_slice_drain_middle_char_error() {
        let hex = "deadgeef";
        let iter = HexToBytesIter::new_unchecked(hex);
        let mut got = [0u8; 4];
        assert_eq!(
            iter.drain_to_slice(&mut got).unwrap_err(),
            InvalidCharError { invalid: b'g', pos: 4 }
        );
    }

    #[test]
    fn hex_to_bytes_slice_drain_end_char_error() {
        let hex = "deadbeeg";
        let iter = HexToBytesIter::new_unchecked(hex);
        let mut got = [0u8; 4];
        assert_eq!(
            iter.drain_to_slice(&mut got).unwrap_err(),
            InvalidCharError { invalid: b'g', pos: 7 }
        );
    }

    #[test]
    fn hex_to_bytes_vec_drain() {
        let hex = "deadbeef";
        let want = [0xde, 0xad, 0xbe, 0xef];
        let iter = HexToBytesIter::new_unchecked(hex);
        let got = iter.drain_to_vec().unwrap();
        assert_eq!(got, want);

        let hex = "";
        let iter = HexToBytesIter::new_unchecked(hex);
        let got = iter.drain_to_vec().unwrap();
        assert!(got.is_empty());
    }

    #[test]
    fn hex_to_bytes_vec_drain_first_char_error() {
        let hex = "geadbeef";
        let iter = HexToBytesIter::new_unchecked(hex);
        assert_eq!(iter.drain_to_vec().unwrap_err(), InvalidCharError { invalid: b'g', pos: 0 });
    }

    #[test]
    fn hex_to_bytes_vec_drain_middle_char_error() {
        let hex = "deadgeef";
        let iter = HexToBytesIter::new_unchecked(hex);
        assert_eq!(iter.drain_to_vec().unwrap_err(), InvalidCharError { invalid: b'g', pos: 4 });
    }

    #[test]
    fn hex_to_bytes_vec_drain_end_char_error() {
        let hex = "deadbeeg";
        let iter = HexToBytesIter::new_unchecked(hex);
        assert_eq!(iter.drain_to_vec().unwrap_err(), InvalidCharError { invalid: b'g', pos: 7 });
    }

    #[test]
    fn encode_iter() {
        let bytes = [0xde, 0xad, 0xbe, 0xef];
        let lower_want = "deadbeef";
        let upper_want = "DEADBEEF";

        for (i, c) in BytesToHexIter::new(bytes.iter(), Case::Lower).enumerate() {
            assert_eq!(c, lower_want.chars().nth(i).unwrap());
        }
        for (i, c) in BytesToHexIter::new(bytes.iter(), Case::Upper).enumerate() {
            assert_eq!(c, upper_want.chars().nth(i).unwrap());
        }
    }

    #[test]
    fn encode_iter_backwards() {
        let bytes = [0xde, 0xad, 0xbe, 0xef];
        let lower_want = "efbeadde";
        let upper_want = "EFBEADDE";

        for (i, c) in BytesToHexIter::new(bytes.iter(), Case::Lower).rev().enumerate() {
            assert_eq!(c, lower_want.chars().nth(i).unwrap());
        }
        for (i, c) in BytesToHexIter::new(bytes.iter(), Case::Upper).rev().enumerate() {
            assert_eq!(c, upper_want.chars().nth(i).unwrap());
        }
    }

    #[test]
    fn roundtrip_forward() {
        let lower_want = "deadbeefcafebabe";
        let upper_want = "DEADBEEFCAFEBABE";
        let lower_bytes_iter = HexToBytesIter::new(lower_want).unwrap().map(|res| res.unwrap());
        let lower_got = BytesToHexIter::new(lower_bytes_iter, Case::Lower).collect::<String>();
        assert_eq!(lower_got, lower_want);
        let upper_bytes_iter = HexToBytesIter::new(upper_want).unwrap().map(|res| res.unwrap());
        let upper_got = BytesToHexIter::new(upper_bytes_iter, Case::Upper).collect::<String>();
        assert_eq!(upper_got, upper_want);
    }

    #[test]
    fn roundtrip_backward() {
        let lower_want = "deadbeefcafebabe";
        let upper_want = "DEADBEEFCAFEBABE";
        let lower_bytes_iter =
            HexToBytesIter::new(lower_want).unwrap().rev().map(|res| res.unwrap());
        let lower_got =
            BytesToHexIter::new(lower_bytes_iter, Case::Lower).rev().collect::<String>();
        assert_eq!(lower_got, lower_want);
        let upper_bytes_iter =
            HexToBytesIter::new(upper_want).unwrap().rev().map(|res| res.unwrap());
        let upper_got =
            BytesToHexIter::new(upper_bytes_iter, Case::Upper).rev().collect::<String>();
        assert_eq!(upper_got, upper_want);
    }

    #[test]
    #[cfg(feature = "std")]
    fn hex_to_bytes_iter_read() {
        use std::io::Read;

        let hex = "deadbeef";
        let mut iter = HexToBytesIter::new(hex).unwrap();
        let mut buf = [0u8; 4];
        let bytes_read = iter.read(&mut buf).unwrap();
        assert_eq!(bytes_read, 4);
        assert_eq!(buf, [0xde, 0xad, 0xbe, 0xef]);

        let hex = "deadbeef";
        let mut iter = HexToBytesIter::new(hex).unwrap();
        let mut buf = [0u8; 2];
        let bytes_read = iter.read(&mut buf).unwrap();
        assert_eq!(bytes_read, 2);
        assert_eq!(buf, [0xde, 0xad]);

        let hex = "deadbeef";
        let mut iter = HexToBytesIter::new(hex).unwrap();
        let mut buf = [0u8; 6];
        let bytes_read = iter.read(&mut buf).unwrap();
        assert_eq!(bytes_read, 4);
        assert_eq!(buf[..4], [0xde, 0xad, 0xbe, 0xef]);
    }
}
