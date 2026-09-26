// SPDX-License-Identifier: CC0-1.0

//! Iterator that converts bytes to hex.

use core::borrow::Borrow;
use core::iter::FusedIterator;

use crate::{Case, Char, Table};

/// Iterator over bytes which encodes the bytes and yields `[Char; 2]` pairs of hex characters.
///
/// Each call to [`Iterator::next`] consumes one byte and returns the two hex digits that encode
/// it as `[high_nibble, low_nibble]`.
///
/// If you want to yield a stream of [`Char`] only, call [`flatten`].
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "alloc")]
/// # {
/// use hex_conservative::{BytesToHexIter, Case};
///
/// let bytes = [0xde, 0xad, 0xbe, 0xef].into_iter();
/// let hex_string: String =
///     BytesToHexIter::new(bytes, Case::Lower).flatten().map(char::from).collect();
/// assert_eq!(hex_string, "deadbeef");
/// # }
///```
///
/// [`flatten`]: Iterator::flatten
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BytesToHexIter<I>
where
    I: Iterator,
    I::Item: Borrow<u8>,
{
    /// The iterator whose next byte will be encoded to yield hex characters.
    iter: I,
    /// The byte-to-hex conversion table.
    table: &'static Table,
}

impl<I> BytesToHexIter<I>
where
    I: Iterator,
    I::Item: Borrow<u8>,
{
    /// Constructs a `BytesToHexIter` that will yield hex character pairs in the given case from a
    /// byte iterator.
    pub fn new(iter: I, case: Case) -> BytesToHexIter<I> { Self { iter, table: case.table() } }
}

impl<I> Iterator for BytesToHexIter<I>
where
    I: Iterator,
    I::Item: Borrow<u8>,
{
    type Item = [Char; 2];

    #[inline]
    fn next(&mut self) -> Option<[Char; 2]> {
        self.iter.next().map(|b| self.table.byte_to_hex_chars(*b.borrow()))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) { self.iter.size_hint() }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<[Char; 2]> {
        self.iter.nth(n).map(|b| self.table.byte_to_hex_chars(*b.borrow()))
    }
}

impl<I> DoubleEndedIterator for BytesToHexIter<I>
where
    I: DoubleEndedIterator,
    I::Item: Borrow<u8>,
{
    #[inline]
    fn next_back(&mut self) -> Option<[Char; 2]> {
        self.iter.next_back().map(|b| self.table.byte_to_hex_chars(*b.borrow()))
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<[Char; 2]> {
        self.iter.nth_back(n).map(|b| self.table.byte_to_hex_chars(*b.borrow()))
    }
}

impl<I> ExactSizeIterator for BytesToHexIter<I>
where
    I: ExactSizeIterator,
    I::Item: Borrow<u8>,
{
    #[inline]
    fn len(&self) -> usize { self.iter.len() }
}

impl<I> FusedIterator for BytesToHexIter<I>
where
    I: FusedIterator,
    I::Item: Borrow<u8>,
{
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "alloc")]
    use alloc::string::String;

    use super::*;
    #[cfg(feature = "alloc")]
    use crate::HexSliceToBytesIter;

    fn nth_slow<I: Iterator>(iter: &mut I, n: usize) -> Option<I::Item> {
        for _ in 0..n {
            iter.next()?;
        }
        iter.next()
    }

    fn nth_back_slow<I: DoubleEndedIterator>(iter: &mut I, n: usize) -> Option<I::Item> {
        for _ in 0..n {
            iter.next_back()?;
        }
        iter.next_back()
    }

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

    #[cfg(feature = "alloc")]
    #[test]
    fn encode_iter() {
        let bytes = [0xde, 0xad, 0xbe, 0xef];
        let lower_want = "deadbeef";
        let upper_want = "DEADBEEF";

        let lower_got: String =
            BytesToHexIter::new(bytes.iter(), Case::Lower).flatten().map(char::from).collect();
        assert_eq!(lower_got, lower_want);
        let upper_got: String =
            BytesToHexIter::new(bytes.iter(), Case::Upper).flatten().map(char::from).collect();
        assert_eq!(upper_got, upper_want);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn encode_iter_backwards() {
        let bytes = [0xde, 0xad, 0xbe, 0xef];
        // .rev().flatten() yields pairs in reverse byte order but each pair remains [high, low].
        let lower_want = "efbeadde";
        let upper_want = "EFBEADDE";

        let lower_got: String = BytesToHexIter::new(bytes.iter(), Case::Lower)
            .rev()
            .flatten()
            .map(char::from)
            .collect();
        assert_eq!(lower_got, lower_want);
        let upper_got: String = BytesToHexIter::new(bytes.iter(), Case::Upper)
            .rev()
            .flatten()
            .map(char::from)
            .collect();
        assert_eq!(upper_got, upper_want);

        // .flatten().rev() yields pairs in reverse byte order and each pair becomes [low, high].
        let lower_want = "feebdaed";
        let upper_want = "FEEBDAED";

        let lower_got: String = BytesToHexIter::new(bytes.iter(), Case::Lower)
            .flatten()
            .rev()
            .map(char::from)
            .collect();
        assert_eq!(lower_got, lower_want);
        let upper_got: String = BytesToHexIter::new(bytes.iter(), Case::Upper)
            .flatten()
            .rev()
            .map(char::from)
            .collect();
        assert_eq!(upper_got, upper_want);
    }

    #[test]
    fn encode_iter_nth() {
        let bytes = [0xde, 0xad, 0xbe, 0xef];

        for n in 0..=bytes.len() + 1 {
            let mut got = BytesToHexIter::new(bytes.iter(), Case::Lower);
            let mut want = BytesToHexIter::new(bytes.iter(), Case::Lower);

            assert_eq!(got.nth(n), nth_slow(&mut want, n));
            assert_eq!(got.len(), want.len());
            assert!(got.eq(want));
        }
    }

    #[test]
    fn encode_iter_nth_after_next_back() {
        let bytes = [0xde, 0xad, 0xbe, 0xef];

        for n in 0..=bytes.len() {
            let mut got = BytesToHexIter::new(bytes.iter(), Case::Lower);
            let mut want = BytesToHexIter::new(bytes.iter(), Case::Lower);

            assert_eq!(got.next_back(), want.next_back());
            assert_eq!(got.nth(n), nth_slow(&mut want, n));
            assert_eq!(got.len(), want.len());
            assert!(got.eq(want));
        }
    }

    #[test]
    fn encode_iter_nth_after_next() {
        let bytes = [0xde, 0xad, 0xbe, 0xef];

        for n in 0..=bytes.len() {
            let mut got = BytesToHexIter::new(bytes.iter(), Case::Lower);
            let mut want = BytesToHexIter::new(bytes.iter(), Case::Lower);

            assert_eq!(got.next(), want.next());
            assert_eq!(got.nth(n), nth_slow(&mut want, n));
            assert_eq!(got.len(), want.len());
            assert!(got.eq(want));
        }
    }

    #[test]
    fn encode_iter_nth_back() {
        let bytes = [0xde, 0xad, 0xbe, 0xef];

        for n in 0..=bytes.len() + 1 {
            let mut got = BytesToHexIter::new(bytes.iter(), Case::Lower);
            let mut want = BytesToHexIter::new(bytes.iter(), Case::Lower);

            assert_eq!(got.nth_back(n), nth_back_slow(&mut want, n));
            assert_eq!(got.len(), want.len());
            assert!(got.eq(want));
        }
    }

    #[test]
    fn encode_iter_nth_back_after_next() {
        let bytes = [0xde, 0xad, 0xbe, 0xef];

        for n in 0..=bytes.len() {
            let mut got = BytesToHexIter::new(bytes.iter(), Case::Lower);
            let mut want = BytesToHexIter::new(bytes.iter(), Case::Lower);

            assert_eq!(got.next(), want.next());
            assert_eq!(got.nth_back(n), nth_back_slow(&mut want, n));
            assert_eq!(got.len(), want.len());
            assert!(got.eq(want));
        }
    }

    #[test]
    fn encode_iter_nth_back_after_next_back() {
        let bytes = [0xde, 0xad, 0xbe, 0xef];

        for n in 0..=bytes.len() {
            let mut got = BytesToHexIter::new(bytes.iter(), Case::Lower);
            let mut want = BytesToHexIter::new(bytes.iter(), Case::Lower);

            assert_eq!(got.next_back(), want.next_back());
            assert_eq!(got.nth_back(n), nth_back_slow(&mut want, n));
            assert_eq!(got.len(), want.len());
            assert!(got.eq(want));
        }
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn roundtrip_forward() {
        let lower_want = "deadbeefcafebabe";
        let upper_want = "DEADBEEFCAFEBABE";
        let lower_bytes_iter =
            HexSliceToBytesIter::new(lower_want).unwrap().map(|res| res.unwrap());
        let lower_got: String =
            BytesToHexIter::new(lower_bytes_iter, Case::Lower).flatten().map(char::from).collect();
        assert_eq!(lower_got, lower_want);
        let upper_bytes_iter =
            HexSliceToBytesIter::new(upper_want).unwrap().map(|res| res.unwrap());
        let upper_got: String =
            BytesToHexIter::new(upper_bytes_iter, Case::Upper).flatten().map(char::from).collect();
        assert_eq!(upper_got, upper_want);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn roundtrip_backward() {
        let lower_want = "deadbeefcafebabe";
        let upper_want = "DEADBEEFCAFEBABE";
        let lower_bytes_iter =
            HexSliceToBytesIter::new(lower_want).unwrap().rev().map(|res| res.unwrap());
        let lower_got: String = BytesToHexIter::new(lower_bytes_iter, Case::Lower)
            .rev()
            .flatten()
            .map(char::from)
            .collect();
        assert_eq!(lower_got, lower_want);
        let upper_bytes_iter =
            HexSliceToBytesIter::new(upper_want).unwrap().rev().map(|res| res.unwrap());
        let upper_got: String = BytesToHexIter::new(upper_bytes_iter, Case::Upper)
            .rev()
            .flatten()
            .map(char::from)
            .collect();
        assert_eq!(upper_got, upper_want);
    }
}
