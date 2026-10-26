// SPDX-License-Identifier: CC0-1.0

//! A minimal string backed by an inline buffer.

use core::fmt::{self, Write};
use core::hash::{Hash, Hasher};
use core::mem::MaybeUninit;

#[cfg(creusot)]
use creusot_std::prelude::*;

pub(crate) struct ArrayString<const CAP: usize> {
    // Only the first `len` bytes are initialized, and they always contain valid UTF-8.
    buf: [MaybeUninit<u8>; CAP],
    len: usize,
}

impl<const CAP: usize> ArrayString<CAP> {
    #[cfg_attr(creusot, check(ghost))]
    #[cfg_attr(creusot, ensures(result@ == Seq::empty()))]
    #[inline]
    pub(crate) fn new() -> Self {
        #[cfg(creusot)]
        proof_assert!(Seq::<char>::empty().to_bytes() == Seq::<u8>::empty());
        Self { buf: [MaybeUninit::uninit(); CAP], len: 0 }
    }

    #[cfg_attr(creusot, check(ghost))]
    #[cfg_attr(creusot, ensures(result@ == self@))]
    #[inline]
    pub(crate) fn as_str(&self) -> &str {
        // SAFETY: `len <= CAP` and only complete strings are appended. The first `len`
        // bytes are initialized UTF-8; the returned borrow cannot outlive or mutate `self`.
        unsafe {
            if self.len > CAP {
                core::hint::unreachable_unchecked()
            }
            let bytes = slice_assume_init_ref(self.buf.split_at(self.len).0);
            #[cfg(creusot)]
            snapshot!(creusot_std::std::string::injective_to_bytes());
            core::str::from_utf8_unchecked(bytes)
        }
    }

    #[cfg_attr(creusot, check(ghost))]
    #[inline]
    #[track_caller]
    #[cfg_attr(creusot, requires(self@.to_bytes().len() + s@.to_bytes().len() <= CAP@))]
    #[cfg_attr(creusot, ensures((^self)@ == (*self)@.concat(s@)))]
    pub(crate) fn push_str(&mut self, s: &str) { self.write_str(s).expect("buffer overflow"); }

    #[cfg_attr(creusot, check(ghost))]
    #[cfg_attr(creusot, ensures((^self)@ == Seq::empty()))]
    #[inline]
    pub(crate) fn clear(&mut self) {
        #[cfg(creusot)]
        proof_assert!(Seq::<char>::empty().to_bytes() == Seq::<u8>::empty());
        self.len = 0;
    }

    #[cfg_attr(creusot, check(ghost))]
    #[cfg_attr(creusot, ensures(result == (self@.to_bytes().len() == CAP@)))]
    #[inline]
    pub(crate) fn is_full(&self) -> bool { self.len == CAP }

    #[cfg_attr(creusot, check(ghost))]
    #[cfg_attr(creusot, ensures(result@ == CAP@ - self@.to_bytes().len()))]
    #[inline]
    pub(crate) fn remaining_capacity(&self) -> usize { CAP - self.len }

    #[cfg(creusot)]
    #[logic(inline)]
    #[requires(self.len@ <= self.buf@.len())]
    #[ensures(result.len() == self.len@)]
    fn to_bytes(self) -> Seq<u8> {
        pearlite! {
            self.buf@.subsequence(0, self.len@).map(|b: MaybeUninit<u8>| match b@ { Some(b) => b, None => creusot_std::logic::any() })
        }
    }
}

impl<const CAP: usize> Write for ArrayString<CAP> {
    #[cfg_attr(creusot, check(ghost))]
    #[inline]
    #[cfg_attr(creusot, ensures(self@.to_bytes().len() + s@.to_bytes().len() <= CAP@ ==> (^self)@ == (*self)@.concat(s@)))]
    #[cfg_attr(creusot, ensures((self@.to_bytes().len() + s@.to_bytes().len() <= CAP@) == (result == Ok(()))))]
    #[cfg_attr(creusot, ensures((self@.to_bytes().len() + s@.to_bytes().len() > CAP@) == (result == Err(fmt::Error))))]
    fn write_str(&mut self, s: &str) -> fmt::Result {
        #[cfg(creusot)]
        snapshot!(creusot_std::std::string::injective_to_bytes());
        #[cfg(creusot)]
        let orig_self = snapshot!(self);
        // SAFETY: Invariant guarantees self.len is in bounds
        let spare_capacity = unsafe { self.buf.get_unchecked_mut(self.len..) };
        // Allow for now, since it may affect future MSRV-related work
        #[allow(clippy::manual_let_else)]
        let dst = match spare_capacity.get_mut(..s.len()) {
            Some(dst) => dst,
            None => {
                return Err(fmt::Error);
            }
        };
        write_copy_of_slice(dst, s.as_bytes());
        self.len += s.len();
        #[cfg(creusot)]
        {
            proof_assert!(forall<i> i >= 0 && i < self.len@ ==> if i < orig_self.to_bytes().len() { self.buf@[i]@ == Some(orig_self.to_bytes()[i]) } else { self.buf@[i]@ == Some(s@.to_bytes()[i - orig_self.to_bytes().len()]) });
            snapshot!(buffer_concat(self.buf@, self.to_bytes(), orig_self.to_bytes(), s@.to_bytes()));
            snapshot!(string_homomorphism(orig_self@, s@, self.to_bytes()));
        }
        // Creusot resolves the borrow right after the last write but this is a problem when
        // proving that the write maintains the invariant is non-trivial and requires some
        // `proof_assert`s. This expression pretends that we're still writing something here,
        // making Creusot resolve the borrow after we have proven that the invariant is upheld.
        let _ = &mut *self;
        Ok(())
    }
}

#[cfg(creusot)]
#[logic(opaque)]
#[requires(buffer.len() >= written.len())]
#[requires(written.len() == left.len() + right.len())]
#[requires(forall<i> i >= 0 && i < left.len() ==> buffer[i]@ == Some(left[i]))]
#[requires(forall<i> i >= 0 && i < right.len() ==> buffer[left.len() + i]@ == Some(right[i]))]
#[requires(forall<i> i >=0 && i < written.len() ==> buffer[i]@ == Some(written[i]))]
#[ensures(written == left.concat(right))]
#[ensures(forall<i> i >= 0 && i < written.len() ==> match buffer[i]@ { Some(_) => true, None => false })]
fn buffer_concat(buffer: Seq<MaybeUninit<u8>>, written: Seq<u8>, left: Seq<u8>, right: Seq<u8>) {}

#[cfg(creusot)]
#[logic(opaque)]
#[requires(c == a.to_bytes().concat(b.to_bytes()))]
#[ensures(exists<s: Seq<char>> s.to_bytes() == c && s == a.concat(b))]
fn string_homomorphism(a: Seq<char>, b: Seq<char>, c: Seq<u8>) {
    a.concat_flat_map(b, |c: char| c.to_utf8());
    creusot_std::std::string::injective_to_bytes();
}

impl<const CAP: usize> Clone for ArrayString<CAP> {
    #[cfg_attr(creusot, check(ghost))]
    #[cfg_attr(creusot, ensures(result@ == self@))]
    #[inline]
    fn clone(&self) -> Self { Self { buf: self.buf, len: self.len } }
}

impl<const CAP: usize> PartialEq for ArrayString<CAP> {
    #[cfg_attr(creusot, ensures(result == (self.deep_model() == other.deep_model())))]
    fn eq(&self, other: &Self) -> bool { self.as_str() == other.as_str() }
}

impl<const CAP: usize> Eq for ArrayString<CAP> {}

impl<const CAP: usize> Hash for ArrayString<CAP> {
    #[cfg_attr(creusot, requires(false))]
    fn hash<H: Hasher>(&self, state: &mut H) { self.as_str().hash(state); }
}

impl<const CAP: usize> fmt::Debug for ArrayString<CAP> {
    #[cfg_attr(creusot, requires(false))]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { fmt::Debug::fmt(self.as_str(), f) }
}

#[cfg(creusot)]
impl<const CAP: usize> View for ArrayString<CAP> {
    type ViewTy = Seq<char>;

    #[logic(inline)]
    #[cfg_attr(creusot, ensures(inv(self) ==> result.to_bytes().len() <= CAP@))]
    fn view(self) -> Self::ViewTy {
        pearlite! {
            if self.invariant() {
                creusot_std::logic::such_that(|s: Seq<char>| {
                    s.to_bytes() == self.to_bytes()
                })
            } else {
                creusot_std::logic::any()
            }
        }
    }
}

#[cfg(creusot)]
impl<const CAP: usize> DeepModel for ArrayString<CAP> {
    type DeepModelTy = Seq<char>;

    #[logic(open, inline)]
    fn deep_model(self) -> Self::DeepModelTy { self.view() }
}

#[cfg(creusot)]
impl<const CAP: usize> Invariant for ArrayString<CAP> {
    #[logic(inline)]
    fn invariant(self) -> bool {
        pearlite! {
            self.len@ <= self.buf@.len() &&
                (forall<i> i >= 0 && i < self.len@ ==> match self.buf[i]@ { Some(_) => true, None => false }) &&
                exists<s: Seq<char>> s.to_bytes() == self.to_bytes()
        }
    }
}

// Polyfill because creusot doesn't support pointer casting and we need const, pre-1.93 MSRV, and
// check(ghost)
#[cfg_attr(creusot, check(ghost))]
#[cfg_attr(creusot, requires(forall<i> i >= 0 && i < slice@.len() ==> match slice[i]@ { Some(_) => true, None => false }))]
#[cfg_attr(creusot, ensures(slice@.len() == result@.len()))]
#[cfg_attr(creusot, ensures(forall<i> i >= 0 && i < slice@.len() ==> slice@[i]@ == Some(result@[i])))]
const unsafe fn slice_assume_init_ref<T>(slice: &[MaybeUninit<T>]) -> &[T] {
    #[cfg(creusot)]
    {
        // SAFETY: the caller must guarantee correctness
        unsafe { slice.assume_init_ref() }
    }
    #[cfg(not(creusot))]
    {
        // SAFETY: the caller must guarantee correctness and the code is literally the copy of
        // `assume_init_ref` from std.
        unsafe { &*(slice as *const [MaybeUninit<T>] as *const [T]) }
    }
}

// Same reason as above
#[cfg_attr(creusot, check(ghost))]
#[cfg_attr(creusot, requires(dest@.len() == src@.len()))]
#[cfg_attr(creusot, ensures((^dest)@.len() == (*dest)@.len()))]
#[cfg_attr(creusot, ensures(forall<i> i >= 0 && i < dest@.len() ==> (^dest)@[i]@ == Some(src[i])))]
fn write_copy_of_slice<T: Copy>(dest: &mut [MaybeUninit<T>], src: &[T]) {
    #[cfg(creusot)]
    {
        dest.write_copy_of_slice(src);
    }
    #[cfg(not(creusot))]
    // We want to use std's code verbatim
    #[allow(clippy::transmute_ptr_to_ptr)]
    {
        // SAFETY: &[T] and &[MaybeUninit<T>] have the same layout
        let uninit_src: &[MaybeUninit<T>] = unsafe { core::mem::transmute(src) };
        dest.copy_from_slice(uninit_src);
    }
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
