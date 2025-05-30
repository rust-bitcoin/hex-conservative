// SPDX-License-Identifier: CC0-1.0

//! Implements a buffered encoder.
//!
//! This is a low-level module, most uses should be satisfied by the `display` module instead.
//!
//! The main type in this module is [`BufEncoder`] which provides buffered hex encoding.
//! `BufEncoder` is faster than the usual `write!(f, "{02x}", b)?` in a for loop because it reduces
//! dynamic dispatch and decreases the number of allocations if a `String` is being created.

use core::borrow::Borrow;

use arrayvec::ArrayString;

use super::{Case, Table};

/// Hex-encodes bytes into the provided buffer.
///
/// This is an important building block for fast hex-encoding. Because string writing tools
/// provided by `core::fmt` involve dynamic dispatch and don't allow reserving capacity in strings
/// buffering the hex and then formatting it is significantly faster.
///
/// The buffer has a fixed capacity specified when created. The capacity must be an even number since
/// each byte is encoded as two hex characters.
///
/// # Examples
/// ```
/// # use hex_conservative::buf_encoder::BufEncoder;
/// # use hex_conservative::Case;
/// let mut encoder = BufEncoder::<4>::new(Case::Lower);
/// encoder.put_byte(0xab);
/// assert_eq!(encoder.as_str(), "ab");
/// ```
/// The following code doesn't compile because of odd capacity:
/// ```compile_fail
/// # use hex_conservative::buf_encoder::BufEncoder;
/// # use hex_conservative::Case;
/// let mut encoder = BufEncoder::<3>::new(Case::Lower);
/// # let _ = encoder;
/// ```
#[derive(Debug)]
pub struct BufEncoder<const CAP: usize> {
    buf: ArrayString<CAP>,
    table: &'static Table,
}

impl<const CAP: usize> BufEncoder<CAP> {
    const _CHECK_EVEN_CAPACITY: () = [(); 1][CAP % 2];

    /// Creates an empty `BufEncoder` that will encode bytes to hex characters in the given case.
    #[inline]
    pub fn new(case: Case) -> Self {
        let () = Self::_CHECK_EVEN_CAPACITY;
        BufEncoder { buf: ArrayString::new(), table: case.table() }
    }

    /// Encodes `byte` as hex and appends it to the buffer.
    ///
    /// ## Panics
    ///
    /// The method panics if the buffer is full.
    #[inline]
    #[track_caller]
    pub fn put_byte(&mut self, byte: u8) {
        let mut hex_chars = [0u8; 2];
        let hex_str = self.table.byte_to_str(&mut hex_chars, byte);
        self.buf.push_str(hex_str);
    }

    /// Encodes `bytes` as hex and appends them to the buffer.
    ///
    /// ## Panics
    ///
    /// The method panics if the bytes wouldn't fit the buffer.
    #[inline]
    #[track_caller]
    pub fn put_bytes<I>(&mut self, bytes: I)
    where
        I: IntoIterator,
        I::Item: Borrow<u8>,
    {
        self.put_bytes_inner(bytes.into_iter());
    }

    #[inline]
    #[track_caller]
    fn put_bytes_inner<I>(&mut self, bytes: I)
    where
        I: Iterator,
        I::Item: Borrow<u8>,
    {
        // May give the compiler better optimization opportunity
        if let Some(max) = bytes.size_hint().1 {
            assert!(max <= self.space_remaining());
        }
        for byte in bytes {
            self.put_byte(*byte.borrow());
        }
    }

    /// Encodes as many `bytes` as fit into the buffer as hex and return the remainder.
    ///
    /// This method works just like `put_bytes` but instead of panicking it returns the unwritten
    /// bytes. The method returns an empty slice if all bytes were written
    #[must_use = "this may write only part of the input buffer"]
    #[inline]
    #[track_caller]
    pub fn put_bytes_min<'a>(&mut self, bytes: &'a [u8]) -> &'a [u8] {
        let to_write = self.space_remaining().min(bytes.len());
        self.put_bytes(&bytes[..to_write]);
        &bytes[to_write..]
    }

    /// Returns true if no more bytes can be written into the buffer.
    #[inline]
    pub fn is_full(&self) -> bool { self.buf.is_full() }

    /// Returns the written bytes as a hex `str`.
    #[inline]
    pub fn as_str(&self) -> &str { &self.buf }

    /// Resets the buffer to become empty.
    #[inline]
    pub fn clear(&mut self) { self.buf.clear(); }

    /// How many bytes can be written to this buffer.
    ///
    /// Note that this returns the number of bytes before encoding, not number of hex digits.
    #[inline]
    pub fn space_remaining(&self) -> usize { self.buf.remaining_capacity() / 2 }

    pub(crate) fn put_filler(&mut self, filler: char, max_count: usize) -> usize {
        let mut buf = [0; 4];
        let filler = filler.encode_utf8(&mut buf);
        let max_capacity = self.buf.remaining_capacity() / filler.len();
        let to_write = max_capacity.min(max_count);

        for _ in 0..to_write {
            self.buf.push_str(filler);
        }

        to_write
    }
}

impl<const CAP: usize> Default for BufEncoder<CAP> {
    #[inline]
    fn default() -> Self { Self::new(Case::Lower) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        let encoder = BufEncoder::<2>::new(Case::Lower);
        assert_eq!(encoder.as_str(), "");
        assert!(!encoder.is_full());

        let encoder = BufEncoder::<2>::new(Case::Upper);
        assert_eq!(encoder.as_str(), "");
        assert!(!encoder.is_full());
    }

    #[test]
    fn single_byte_exact_buf() {
        let mut encoder = BufEncoder::<2>::new(Case::Lower);
        assert_eq!(encoder.space_remaining(), 1);
        encoder.put_byte(42);
        assert_eq!(encoder.as_str(), "2a");
        assert_eq!(encoder.space_remaining(), 0);
        assert!(encoder.is_full());
        encoder.clear();
        assert_eq!(encoder.space_remaining(), 1);
        assert!(!encoder.is_full());

        let mut encoder = BufEncoder::<2>::new(Case::Upper);
        assert_eq!(encoder.space_remaining(), 1);
        encoder.put_byte(42);
        assert_eq!(encoder.as_str(), "2A");
        assert_eq!(encoder.space_remaining(), 0);
        assert!(encoder.is_full());
        encoder.clear();
        assert_eq!(encoder.space_remaining(), 1);
        assert!(!encoder.is_full());
    }

    #[test]
    fn single_byte_oversized_buf() {
        let mut encoder = BufEncoder::<4>::new(Case::Lower);
        assert_eq!(encoder.space_remaining(), 2);
        encoder.put_byte(42);
        assert_eq!(encoder.space_remaining(), 1);
        assert_eq!(encoder.as_str(), "2a");
        assert!(!encoder.is_full());
        encoder.clear();
        assert_eq!(encoder.space_remaining(), 2);
        assert!(!encoder.is_full());

        let mut encoder = BufEncoder::<4>::new(Case::Upper);
        assert_eq!(encoder.space_remaining(), 2);
        encoder.put_byte(42);
        assert_eq!(encoder.space_remaining(), 1);
        assert_eq!(encoder.as_str(), "2A");
        assert!(!encoder.is_full());
        encoder.clear();
        assert_eq!(encoder.space_remaining(), 2);
        assert!(!encoder.is_full());
    }

    #[test]
    fn two_bytes() {
        let mut encoder = BufEncoder::<4>::new(Case::Lower);
        assert_eq!(encoder.space_remaining(), 2);
        encoder.put_byte(42);
        assert_eq!(encoder.space_remaining(), 1);
        encoder.put_byte(255);
        assert_eq!(encoder.space_remaining(), 0);
        assert_eq!(encoder.as_str(), "2aff");
        assert!(encoder.is_full());
        encoder.clear();
        assert_eq!(encoder.space_remaining(), 2);
        assert!(!encoder.is_full());

        let mut encoder = BufEncoder::<4>::new(Case::Upper);
        assert_eq!(encoder.space_remaining(), 2);
        encoder.put_byte(42);
        assert_eq!(encoder.space_remaining(), 1);
        encoder.put_byte(255);
        assert_eq!(encoder.space_remaining(), 0);
        assert_eq!(encoder.as_str(), "2AFF");
        assert!(encoder.is_full());
        encoder.clear();
        assert_eq!(encoder.space_remaining(), 2);
        assert!(!encoder.is_full());
    }

    #[test]
    fn put_bytes_min() {
        let mut encoder = BufEncoder::<2>::new(Case::Lower);
        let remainder = encoder.put_bytes_min(b"");
        assert_eq!(remainder, b"");
        assert_eq!(encoder.as_str(), "");
        let remainder = encoder.put_bytes_min(b"*");
        assert_eq!(remainder, b"");
        assert_eq!(encoder.as_str(), "2a");
        encoder.clear();
        let remainder = encoder.put_bytes_min(&[42, 255]);
        assert_eq!(remainder, &[255]);
        assert_eq!(encoder.as_str(), "2a");
    }

    #[test]
    fn put_filler() {
        let mut encoder = BufEncoder::<8>::new(Case::Lower);
        assert_eq!(encoder.put_filler(' ', 0), 0);
        assert_eq!(encoder.as_str(), "");
        assert_eq!(encoder.put_filler('a', 1), 1);
        assert_eq!(encoder.as_str(), "a");
        assert_eq!(encoder.put_filler('é', 2), 2); // Test 2 byte UTF-8
        assert_eq!(encoder.as_str(), "aéé");
        assert_eq!(encoder.put_filler('é', 4), 1); // Try to fill more than fits
        assert_eq!(encoder.as_str(), "aééé");
    }

    #[test]
    fn same_as_fmt() {
        use core::fmt::{self, Write};

        struct Writer {
            buf: [u8; 2],
            pos: usize,
        }

        impl Writer {
            fn as_str(&self) -> &str { core::str::from_utf8(&self.buf[..self.pos]).unwrap() }
        }

        impl Write for Writer {
            fn write_str(&mut self, s: &str) -> fmt::Result {
                assert!(self.pos <= 2);
                if s.len() > 2 - self.pos {
                    Err(fmt::Error)
                } else {
                    self.buf[self.pos..(self.pos + s.len())].copy_from_slice(s.as_bytes());
                    self.pos += s.len();
                    Ok(())
                }
            }
        }

        let mut writer = Writer { buf: [0u8; 2], pos: 0 };

        let mut encoder = BufEncoder::<2>::new(Case::Lower);
        for i in 0..=255 {
            write!(writer, "{:02x}", i).unwrap();
            encoder.put_byte(i);
            assert_eq!(encoder.as_str(), writer.as_str());
            writer.pos = 0;
            encoder.clear();
        }

        let mut encoder = BufEncoder::<2>::new(Case::Upper);
        for i in 0..=255 {
            write!(writer, "{:02X}", i).unwrap();
            encoder.put_byte(i);
            assert_eq!(encoder.as_str(), writer.as_str());
            writer.pos = 0;
            encoder.clear();
        }
    }
}
