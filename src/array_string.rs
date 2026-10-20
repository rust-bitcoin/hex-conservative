// SPDX-License-Identifier: CC0-1.0

//! A minimal string backed by an inline buffer.

use core::fmt::{self, Write};
use core::hash::{Hash, Hasher};
use core::mem::MaybeUninit;

pub(crate) struct ArrayString<const CAP: usize> {
    // Only the first `len` bytes are initialized, and they always contain valid UTF-8.
    buf: [MaybeUninit<u8>; CAP],
    len: usize,
}

impl<const CAP: usize> ArrayString<CAP> {
    #[inline]
    pub(crate) fn new() -> Self { Self { buf: [MaybeUninit::uninit(); CAP], len: 0 } }

    #[inline]
    pub(crate) fn as_str(&self) -> &str {
        // SAFETY: `len <= CAP` and only complete strings are appended. The first `len`
        // bytes are initialized UTF-8; the returned borrow cannot outlive or mutate `self`.
        unsafe {
            let bytes = core::slice::from_raw_parts(self.buf.as_ptr().cast::<u8>(), self.len);
            core::str::from_utf8_unchecked(bytes)
        }
    }

    #[inline]
    #[track_caller]
    pub(crate) fn push_str(&mut self, s: &str) { self.write_str(s).expect("buffer overflow"); }

    #[inline]
    pub(crate) fn clear(&mut self) { self.len = 0; }

    #[inline]
    pub(crate) fn is_full(&self) -> bool { self.len == CAP }

    #[inline]
    pub(crate) fn remaining_capacity(&self) -> usize { CAP - self.len }
}

impl<const CAP: usize> Write for ArrayString<CAP> {
    #[inline]
    fn write_str(&mut self, s: &str) -> fmt::Result {
        if s.len() > self.remaining_capacity() {
            return Err(fmt::Error);
        }
        // SAFETY: the capacity check bounds the destination. The exclusive borrow of
        // `self` prevents overlap with `s`. Copying a complete string preserves UTF-8.
        unsafe {
            core::ptr::copy_nonoverlapping(
                s.as_ptr(),
                self.buf.as_mut_ptr().cast::<u8>().add(self.len),
                s.len(),
            );
        }
        self.len += s.len();
        Ok(())
    }
}

impl<const CAP: usize> Clone for ArrayString<CAP> {
    #[inline]
    fn clone(&self) -> Self { Self { buf: self.buf, len: self.len } }
}

impl<const CAP: usize> PartialEq for ArrayString<CAP> {
    fn eq(&self, other: &Self) -> bool { self.as_str() == other.as_str() }
}

impl<const CAP: usize> Eq for ArrayString<CAP> {}

impl<const CAP: usize> Hash for ArrayString<CAP> {
    fn hash<H: Hasher>(&self, state: &mut H) { self.as_str().hash(state); }
}

impl<const CAP: usize> fmt::Debug for ArrayString<CAP> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { fmt::Debug::fmt(self.as_str(), f) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_capacity() {
        let mut s = ArrayString::<0>::new();
        assert_eq!(s.as_str(), "");
        assert!(s.is_full());
        assert_eq!(s.remaining_capacity(), 0);
        s.write_str("").unwrap();
        assert_eq!(s.write_char('a'), Err(fmt::Error));
        assert_eq!(s.clone(), s);
        s.clear();
        assert_eq!(s.as_str(), "");
    }

    #[test]
    fn utf8_exact_capacity() {
        let mut s = ArrayString::<10>::new();
        for (c, remaining) in [('a', 9), ('é', 7), ('水', 4), ('🦀', 0)] {
            s.write_char(c).unwrap();
            assert_eq!(s.remaining_capacity(), remaining);
        }
        assert_eq!(s.as_str(), "aé水🦀");
        assert!(s.is_full());
        s.push_str("");
    }

    #[test]
    fn rejected_write_preserves_contents() {
        let mut s = ArrayString::<4>::new();
        s.push_str("é");
        assert_eq!(s.write_str("水"), Err(fmt::Error));
        assert_eq!(s.write_char('🦀'), Err(fmt::Error));
        assert_eq!(s.as_str(), "é");
        assert_eq!(s.remaining_capacity(), 2);
        s.write_char('é').unwrap();
        assert_eq!(s.as_str(), "éé");
        assert!(s.is_full());
    }

    #[test]
    fn clear_reuse_and_clone() {
        let mut s = ArrayString::<16>::new();
        s.push_str("longer contents");
        s.clear();
        assert_eq!(s.as_str(), "");
        assert_eq!(s.remaining_capacity(), 16);
        s.push_str("é");
        let mut copy = s.clone();
        assert_eq!(copy, s);
        s.clear();
        s.push_str("other");
        copy.push_str("!");
        assert_eq!(copy.as_str(), "é!");
        assert_eq!(s.as_str(), "other");
        assert_ne!(copy, s);
    }

    #[test]
    #[should_panic(expected = "buffer overflow")]
    fn push_overflow() { ArrayString::<1>::new().push_str("é"); }

    #[cfg(feature = "std")]
    #[test]
    fn traits_use_only_string_contents() {
        use std::collections::hash_map::DefaultHasher;

        let mut reused = ArrayString::<32>::new();
        reused.push_str("discarded contents");
        reused.clear();
        reused.push_str("é\n\"");
        let mut fresh = ArrayString::<32>::new();
        fresh.push_str(reused.as_str());
        assert_eq!(reused, fresh);
        let mut actual = DefaultHasher::new();
        let mut expected = DefaultHasher::new();
        reused.hash(&mut actual);
        fresh.as_str().hash(&mut expected);
        assert_eq!(actual.finish(), expected.finish());
        assert_eq!(format!("{:?}", reused), format!("{:?}", fresh.as_str()));
    }
}
