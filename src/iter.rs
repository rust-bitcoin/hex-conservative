// SPDX-License-Identifier: CC0-1.0

//! Iterator that converts hex to bytes.

// SPDX-License-Identifier: CC0-1.0

//! Iterator that converts hex to bytes.

use core::borrow::Borrow;
use core::iter::FusedIterator;

#[doc(inline)]
pub use hex_stable::iter::{HexSliceToBytesIter, HexToBytesIter};

#[cfg(all(feature = "alloc", not(feature = "std")))]
use crate::alloc::vec::Vec;
use crate::{Case, Table};

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
}
