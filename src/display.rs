// SPDX-License-Identifier: CC0-1.0

//! Helpers for displaying bytes as hex strings.
//!
//! This module provides a trait for displaying things as hex as well as an implementation for
//! `&[u8]`.
//!
//! For arrays and slices we support padding and precision for length < 512 bytes.
//!
//! # Examples
//!
//! ```
//! use hex_conservative::DisplayHex;
//!
//! // Display as hex.
//! let v = vec![0xde, 0xad, 0xbe, 0xef];
//! assert_eq!(format!("{}", v.as_hex()), "deadbeef");
//!
//! // Get the most significant bytes.
//! let v = vec![0x01, 0x23, 0x45, 0x67];
//! assert_eq!(format!("{0:.4}", v.as_hex()), "0123");
//!
//! // Padding with zeros
//! let v = vec![0xab; 2];
//! assert_eq!(format!("{:0>8}", v.as_hex()), "0000abab");
//!```

#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::string::String;
use core::borrow::Borrow;
use core::fmt;

use super::{Case, Table};
use crate::buf_encoder::BufEncoder;

/// Extension trait for types that can be displayed as hex.
///
/// Types that have a single, obvious text representation being hex should **not** implement this
/// trait and simply implement `Display` instead.
///
/// This trait should be generally implemented for references only. We would prefer to use GAT but
/// that is beyond our MSRV. As a lint we require the `IsRef` trait which is implemented for all
/// references.
pub trait DisplayHex: Copy + sealed::IsRef {
    /// The type providing [`fmt::Display`] implementation.
    ///
    /// This is usually a wrapper type holding a reference to `Self`.
    type Display: fmt::Display + fmt::Debug + fmt::LowerHex + fmt::UpperHex;

    /// Display `Self` as a continuous sequence of ASCII hex chars.
    fn as_hex(self) -> Self::Display;

    /// Create a lower-hex-encoded string.
    ///
    /// A shorthand for `to_hex_string(Case::Lower)`, so that `Case` doesn't need to be imported.
    ///
    /// This may be faster than `.display_hex().to_string()` because it uses `reserve_suggestion`.
    #[cfg(feature = "alloc")]
    fn to_lower_hex_string(self) -> String { self.to_hex_string(Case::Lower) }

    /// Create an upper-hex-encoded string.
    ///
    /// A shorthand for `to_hex_string(Case::Upper)`, so that `Case` doesn't need to be imported.
    ///
    /// This may be faster than `.display_hex().to_string()` because it uses `reserve_suggestion`.
    #[cfg(feature = "alloc")]
    fn to_upper_hex_string(self) -> String { self.to_hex_string(Case::Upper) }

    /// Create a hex-encoded string.
    ///
    /// This may be faster than `.display_hex().to_string()` because it uses `reserve_suggestion`.
    #[cfg(feature = "alloc")]
    fn to_hex_string(self, case: Case) -> String {
        let mut string = String::new();
        self.append_hex_to_string(case, &mut string);
        string
    }

    /// Appends hex-encoded content to an existing `String`.
    ///
    /// This may be faster than `write!(string, "{:x}", self.as_hex())` because it uses
    /// `hex_reserve_sugggestion`.
    #[cfg(feature = "alloc")]
    fn append_hex_to_string(self, case: Case, string: &mut String) {
        use fmt::Write;

        string.reserve(self.hex_reserve_suggestion());
        match case {
            Case::Lower => write!(string, "{:x}", self.as_hex()),
            Case::Upper => write!(string, "{:X}", self.as_hex()),
        }
        .unwrap_or_else(|_| {
            let name = core::any::type_name::<Self::Display>();
            // We don't expect `std` to ever be buggy, so the bug is most likely in the `Display`
            // impl of `Self::Display`.
            panic!("The implementation of Display for {} returned an error when it shouldn't", name)
        })
    }

    /// Hints how much bytes to reserve when creating a `String`.
    ///
    /// Implementors that know the number of produced bytes upfront should override this.
    /// Defaults to 0.
    ///
    // We prefix the name with `hex_` to avoid potential collision with other methods.
    fn hex_reserve_suggestion(self) -> usize { 0 }
}

fn internal_display(bytes: &[u8], f: &mut fmt::Formatter, case: Case) -> fmt::Result {
    use fmt::Write;
    // There are at least two optimizations left:
    //
    // * Reusing the buffer (encoder) which may decrease the number of virtual calls
    // * Not recursing, avoiding another 1024B allocation and zeroing
    //
    // This would complicate the code so I was too lazy to do them but feel free to send a PR!

    let mut encoder = BufEncoder::<1024>::new(case);
    let pad_right = write_pad_left(f, bytes.len(), &mut encoder)?;

    if f.alternate() {
        f.write_str("0x")?;
    }
    match f.precision() {
        Some(max) if bytes.len() > max / 2 => {
            write!(f, "{}", bytes[..(max / 2)].as_hex())?;
            if max % 2 == 1 {
                f.write_char(case.table().byte_to_chars(bytes[max / 2])[0])?;
            }
        }
        Some(_) | None => {
            let mut chunks = bytes.chunks_exact(512);
            for chunk in &mut chunks {
                encoder.put_bytes(chunk);
                f.write_str(encoder.as_str())?;
                encoder.clear();
            }
            encoder.put_bytes(chunks.remainder());
            f.write_str(encoder.as_str())?;
        }
    }

    write_pad_right(f, pad_right, &mut encoder)
}

fn write_pad_left(
    f: &mut fmt::Formatter,
    bytes_len: usize,
    encoder: &mut BufEncoder<1024>,
) -> Result<usize, fmt::Error> {
    let pad_right = if let Some(width) = f.width() {
        // Add space for 2 characters if the '#' flag is set
        let full_string_len = if f.alternate() { bytes_len * 2 + 2 } else { bytes_len * 2 };
        let string_len = match f.precision() {
            Some(max) => core::cmp::min(max, full_string_len),
            None => full_string_len,
        };

        if string_len < width {
            let (left, right) = match f.align().unwrap_or(fmt::Alignment::Left) {
                fmt::Alignment::Left => (0, width - string_len),
                fmt::Alignment::Right => (width - string_len, 0),
                fmt::Alignment::Center => ((width - string_len) / 2, (width - string_len + 1) / 2),
            };
            // Avoid division by zero and optimize for common case.
            if left > 0 {
                let c = f.fill();
                let chunk_len = encoder.put_filler(c, left);
                let padding = encoder.as_str();
                for _ in 0..(left / chunk_len) {
                    f.write_str(padding)?;
                }
                f.write_str(&padding[..((left % chunk_len) * c.len_utf8())])?;
                encoder.clear();
            }
            right
        } else {
            0
        }
    } else {
        0
    };
    Ok(pad_right)
}

fn write_pad_right(
    f: &mut fmt::Formatter,
    pad_right: usize,
    encoder: &mut BufEncoder<1024>,
) -> fmt::Result {
    // Avoid division by zero and optimize for common case.
    if pad_right > 0 {
        encoder.clear();
        let c = f.fill();
        let chunk_len = encoder.put_filler(c, pad_right);
        let padding = encoder.as_str();
        for _ in 0..(pad_right / chunk_len) {
            f.write_str(padding)?;
        }
        f.write_str(&padding[..((pad_right % chunk_len) * c.len_utf8())])?;
    }
    Ok(())
}

mod sealed {
    /// Trait marking a shared reference.
    pub trait IsRef: Copy {}

    impl<T: ?Sized> IsRef for &'_ T {}
}

impl<'a> DisplayHex for &'a [u8] {
    type Display = DisplayByteSlice<'a>;

    #[inline]
    fn as_hex(self) -> Self::Display { DisplayByteSlice { bytes: self } }

    #[inline]
    fn hex_reserve_suggestion(self) -> usize {
        // Since the string wouldn't fit into address space if this overflows (actually even for
        // smaller amounts) it's better to panic right away. It should also give the optimizer
        // better opportunities.
        self.len().checked_mul(2).expect("the string wouldn't fit into address space")
    }
}

#[cfg(feature = "alloc")]
impl<'a> DisplayHex for &'a alloc::vec::Vec<u8> {
    type Display = DisplayByteSlice<'a>;

    #[inline]
    fn as_hex(self) -> Self::Display { DisplayByteSlice { bytes: self } }

    #[inline]
    fn hex_reserve_suggestion(self) -> usize {
        // Since the string wouldn't fit into address space if this overflows (actually even for
        // smaller amounts) it's better to panic right away. It should also give the optimizer
        // better opportunities.
        self.len().checked_mul(2).expect("the string wouldn't fit into address space")
    }
}

/// Displays byte slice as hex.
///
/// Created by [`<&[u8] as DisplayHex>::as_hex`](DisplayHex::as_hex).
pub struct DisplayByteSlice<'a> {
    // pub because we want to keep lengths in sync
    pub(crate) bytes: &'a [u8],
}

impl DisplayByteSlice<'_> {
    fn display(&self, f: &mut fmt::Formatter, case: Case) -> fmt::Result {
        internal_display(self.bytes, f, case)
    }
}

impl fmt::Display for DisplayByteSlice<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { fmt::LowerHex::fmt(self, f) }
}

impl fmt::Debug for DisplayByteSlice<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { fmt::LowerHex::fmt(self, f) }
}

impl fmt::LowerHex for DisplayByteSlice<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { self.display(f, Case::Lower) }
}

impl fmt::UpperHex for DisplayByteSlice<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { self.display(f, Case::Upper) }
}

/// Displays byte array as hex.
///
/// Created by [`<&[u8; CAP / 2] as DisplayHex>::as_hex`](DisplayHex::as_hex).
pub struct DisplayArray<'a, const CAP: usize> {
    array: &'a [u8],
}

impl<'a, const CAP: usize> DisplayArray<'a, CAP> {
    /// Creates the wrapper.
    ///
    /// # Panics
    ///
    /// When the length of array is greater than capacity / 2.
    #[inline]
    fn new(array: &'a [u8]) -> Self {
        assert!(array.len() <= CAP / 2);
        DisplayArray { array }
    }

    fn display(&self, f: &mut fmt::Formatter, case: Case) -> fmt::Result {
        internal_display(self.array, f, case)
    }
}

impl<const LEN: usize> fmt::Display for DisplayArray<'_, LEN> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { fmt::LowerHex::fmt(self, f) }
}

impl<const LEN: usize> fmt::Debug for DisplayArray<'_, LEN> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { fmt::LowerHex::fmt(self, f) }
}

impl<const LEN: usize> fmt::LowerHex for DisplayArray<'_, LEN> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { self.display(f, Case::Lower) }
}

impl<const LEN: usize> fmt::UpperHex for DisplayArray<'_, LEN> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { self.display(f, Case::Upper) }
}

macro_rules! impl_array_as_hex {
    ($($len:expr),*) => {
        $(
            impl<'a> DisplayHex for &'a [u8; $len] {
                type Display = DisplayArray<'a, {$len * 2}>;

                fn as_hex(self) -> Self::Display {
                    DisplayArray::new(self)
                }
            }
        )*
    }
}

impl_array_as_hex!(
    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 20, 32, 33, 64, 65, 128, 256, 512, 1024,
    2048, 4096
);

/// Format known-length array as hex.
///
/// This supports all formatting options of formatter and may be faster than calling `as_hex()` on
/// an arbitrary `&[u8]`. Note that the implementation intentionally keeps leading zeros even when
/// not requested. This is designed to display values such as hashes and keys and removing leading
/// zeros would be confusing.
///
/// Note that the bytes parameter is `IntoIterator` this means that if you would like to do some
/// manipulation to the byte array before formatting then you can. For example `bytes.iter().rev()`
/// to print the array backwards.
///
/// ## Parameters
///
/// * `$formatter` - a [`fmt::Formatter`].
/// * `$len` known length of `$bytes`, must be a const expression.
/// * `$bytes` - bytes to be encoded, most likely a reference to an array.
/// * `$case` - value of type [`Case`] determining whether to format as lower or upper case.
///
/// ## Panics
///
/// This macro panics if `$len` is not equal to `$bytes.len()`. It also fails to compile if `$len`
/// is more than half of `usize::MAX`.
#[macro_export]
macro_rules! fmt_hex_exact {
    ($formatter:expr, $len:expr, $bytes:expr, $case:expr) => {{
        // statically check $len
        #[allow(deprecated)]
        const _: () = [()][($len > usize::MAX / 2) as usize];
        assert_eq!($bytes.len(), $len);
        $crate::display::fmt_hex_exact_fn::<_, { $len * 2 }>($formatter, $bytes, $case)
    }};
}
pub use fmt_hex_exact;

/// Adds `core::fmt` trait implementations to type `$ty`.
///
/// Implements:
///
/// - `fmt::{LowerHex, UpperHex}` using [`fmt_hex_exact`].
/// - `fmt::{Display, Debug}` by calling `LowerHex`.
///
/// Requires:
///
/// - `$ty` must implement `IntoIterator<Item=Borrow<u8>>`.
///
/// ## Parameters
///
/// * `$ty` - the type to implement traits on.
/// * `$len` - known length of `$bytes`, must be a const expression.
/// * `$bytes` - bytes to be encoded, most likely a reference to an array.
/// * `$reverse` - true if you want the array to be displayed backwards.
/// * `$gen: $gent` - optional generic type(s) and trait bound(s) to put on `$ty` e.g, `F: Foo`.
///
/// ## Examples
///
/// ```
/// # use core::borrow::Borrow;
/// # use hex_conservative::impl_fmt_traits;
/// struct Wrapper([u8; 4]);
///
/// impl Borrow<[u8]> for Wrapper {
///     fn borrow(&self) -> &[u8] { &self.0[..] }
/// }
///
/// impl_fmt_traits! {
///     impl fmt_traits for Wrapper {
///         const LENGTH: usize = 4;
///     }
/// }
///
/// let w = Wrapper([0x12, 0x34, 0x56, 0x78]);
/// assert_eq!(format!("{}", w), "12345678");
/// ```
///
/// We support generics on `$ty`:
///
/// ```
/// # use core::borrow::Borrow;
/// # use core::marker::PhantomData;
/// # use hex_conservative::impl_fmt_traits;
/// struct Wrapper<T>([u8; 4], PhantomData<T>);
///
/// // `Clone` is just some arbitrary trait.
/// impl<T: Clone> Borrow<[u8]> for Wrapper<T> {
///     fn borrow(&self) -> &[u8] { &self.0[..] }
/// }
///
/// impl_fmt_traits! {
///     impl<T: Clone> fmt_traits for Wrapper<T> {
///         const LENGTH: usize = 4;
///     }
/// }
///
/// let w = Wrapper([0x12, 0x34, 0x56, 0x78], PhantomData::<u32>);
/// assert_eq!(format!("{}", w), "12345678");
/// ```
///
/// And also, as is required by `rust-bitcoin`, we support displaying
/// the hex string byte-wise backwards:
///
/// ```
/// # use core::borrow::Borrow;
/// # use hex_conservative::impl_fmt_traits;
/// struct Wrapper([u8; 4]);
///
/// impl Borrow<[u8]> for Wrapper {
///     fn borrow(&self) -> &[u8] { &self.0[..] }
/// }
///
/// impl_fmt_traits! {
///     #[display_backward(true)]
///     impl fmt_traits for Wrapper {
///         const LENGTH: usize = 4;
///     }
/// }
/// let w = Wrapper([0x12, 0x34, 0x56, 0x78]);
/// assert_eq!(format!("{}", w), "78563412");
/// ```
#[macro_export]
macro_rules! impl_fmt_traits {
    // Without generic and trait bounds and without display_backward attribute.
    (impl fmt_traits for $ty:ident { const LENGTH: usize = $len:expr; }) => {
        $crate::impl_fmt_traits! {
            #[display_backward(false)]
            impl<> fmt_traits for $ty<> {
                const LENGTH: usize = $len;
            }
        }
    };
    // Without generic and trait bounds and with display_backward attribute.
    (#[display_backward($reverse:expr)] impl fmt_traits for $ty:ident { const LENGTH: usize = $len:expr; }) => {
        $crate::impl_fmt_traits! {
            #[display_backward($reverse)]
            impl<> fmt_traits for $ty<> {
                const LENGTH: usize = $len;
            }
        }
    };
    // With generic and trait bounds and without display_backward attribute.
    (impl<$($gen:ident: $gent:ident),*> fmt_traits for $ty:ident<$($unused:ident),*> { const LENGTH: usize = $len:expr; }) => {
        $crate::impl_fmt_traits! {
            #[display_backward(false)]
            impl<$($gen: $gent),*> fmt_traits for $ty<$($unused),*> {
                const LENGTH: usize = $len;
            }
        }
    };
    // With generic and trait bounds and display_backward attribute.
    (#[display_backward($reverse:expr)] impl<$($gen:ident: $gent:ident),*> fmt_traits for $ty:ident<$($unused:ident),*> { const LENGTH: usize = $len:expr; }) => {
        impl<$($gen: $gent),*> $crate::_export::_core::fmt::LowerHex for $ty<$($gen),*> {
            #[inline]
            fn fmt(&self, f: &mut $crate::_export::_core::fmt::Formatter) -> $crate::_export::_core::fmt::Result {
                let case = $crate::Case::Lower;

                if $reverse {
                    let bytes = $crate::_export::_core::borrow::Borrow::<[u8]>::borrow(self).iter().rev();
                    $crate::fmt_hex_exact!(f, $len, bytes, case)
                } else {
                    let bytes = $crate::_export::_core::borrow::Borrow::<[u8]>::borrow(self).iter();
                    $crate::fmt_hex_exact!(f, $len, bytes, case)
                }
            }
        }

        impl<$($gen: $gent),*> $crate::_export::_core::fmt::UpperHex for $ty<$($gen),*> {
            #[inline]
            fn fmt(&self, f: &mut $crate::_export::_core::fmt::Formatter) -> $crate::_export::_core::fmt::Result {
                let case = $crate::Case::Upper;

                if $reverse {
                    let bytes = $crate::_export::_core::borrow::Borrow::<[u8]>::borrow(self).iter().rev();
                    $crate::fmt_hex_exact!(f, $len, bytes, case)
                } else {
                    let bytes = $crate::_export::_core::borrow::Borrow::<[u8]>::borrow(self).iter();
                    $crate::fmt_hex_exact!(f, $len, bytes, case)
                }
            }
        }

        impl<$($gen: $gent),*> $crate::_export::_core::fmt::Display for $ty<$($gen),*> {
            #[inline]
            fn fmt(&self, f: &mut $crate::_export::_core::fmt::Formatter) -> $crate::_export::_core::fmt::Result {
                $crate::_export::_core::fmt::LowerHex::fmt(self, f)
            }
        }

        impl<$($gen: $gent),*> $crate::_export::_core::fmt::Debug for $ty<$($gen),*> {
            #[inline]
            fn fmt(&self, f: &mut $crate::_export::_core::fmt::Formatter) -> $crate::_export::_core::fmt::Result {
                $crate::_export::_core::fmt::LowerHex::fmt(&self, f)
            }
        }
    };
}
pub use impl_fmt_traits;

// Implementation detail of `write_hex_exact` macro to de-duplicate the code
//
// Whether hex is an integer or a string is debatable, we cater a little bit to each.
// - We support users adding `0x` prefix using "{:#}" (treating hex like an integer).
// - We support limiting the output using precision "{:.10}" (treating hex like a string).
//
// This assumes `bytes.len() * 2 == N`.
#[doc(hidden)]
#[inline]
pub fn fmt_hex_exact_fn<I, const N: usize>(
    f: &mut fmt::Formatter,
    bytes: I,
    case: Case,
) -> fmt::Result
where
    I: IntoIterator,
    I::Item: Borrow<u8>,
{
    let mut padding_encoder = BufEncoder::<1024>::new(case);
    let pad_right = write_pad_left(f, N / 2, &mut padding_encoder)?;

    if f.alternate() {
        f.write_str("0x")?;
    }
    let mut encoder = BufEncoder::<N>::new(case);
    let encoded = match f.precision() {
        Some(p) if p < N => {
            let n = (p + 1) / 2;
            encoder.put_bytes(bytes.into_iter().take(n));
            &encoder.as_str()[..p]
        }
        _ => {
            encoder.put_bytes(bytes);
            encoder.as_str()
        }
    };
    f.write_str(encoded)?;

    write_pad_right(f, pad_right, &mut padding_encoder)
}

/// Given a `T:` [`fmt::Write`], `HexWriter` implements [`std::io::Write`]
/// and writes the source bytes to its inner `T` as hex characters.
#[cfg(any(test, feature = "std"))]
#[cfg_attr(docsrs, doc(cfg(any(test, feature = "std"))))]
pub struct HexWriter<T> {
    writer: T,
    table: &'static Table,
}

#[cfg(any(test, feature = "std"))]
#[cfg_attr(docsrs, doc(cfg(any(test, feature = "std"))))]
impl<T> HexWriter<T> {
    /// Creates a `HexWriter` that writes the source bytes to `dest` as hex characters
    /// in the given `case`.
    pub fn new(dest: T, case: Case) -> Self { Self { writer: dest, table: case.table() } }
    /// Consumes this `HexWriter` returning the inner `T`.
    pub fn into_inner(self) -> T { self.writer }
}

#[cfg(any(test, feature = "std"))]
#[cfg_attr(docsrs, doc(cfg(any(test, feature = "std"))))]
impl<T> std::io::Write for HexWriter<T>
where
    T: core::fmt::Write,
{
    /// # Errors
    ///
    /// If no bytes could be written to this `HexWriter`, and the provided buffer is not empty,
    /// returns [`std::io::ErrorKind::Other`], otherwise returns `Ok`.
    fn write(&mut self, buf: &[u8]) -> Result<usize, std::io::Error> {
        let mut n = 0;
        for byte in buf {
            let mut hex_chars = [0u8; 2];
            let hex_str = self.table.byte_to_str(&mut hex_chars, *byte);
            if self.writer.write_str(hex_str).is_err() {
                break;
            }
            n += 1;
        }
        if n == 0 && !buf.is_empty() {
            Err(std::io::ErrorKind::Other.into())
        } else {
            Ok(n)
        }
    }
    fn flush(&mut self) -> Result<(), std::io::Error> { Ok(()) }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "alloc")]
    use super::*;

    #[cfg(feature = "alloc")]
    mod alloc {
        use core::marker::PhantomData;

        use super::*;

        fn check_encoding(bytes: &[u8]) {
            use core::fmt::Write;

            let s1 = bytes.to_lower_hex_string();
            let mut s2 = String::with_capacity(bytes.len() * 2);
            for b in bytes {
                write!(s2, "{:02x}", b).unwrap();
            }
            assert_eq!(s1, s2);
        }

        #[test]
        fn empty() { check_encoding(b""); }

        #[test]
        fn single() { check_encoding(b"*"); }

        #[test]
        fn two() { check_encoding(b"*x"); }

        #[test]
        fn just_below_boundary() { check_encoding(&[42; 512]); }

        #[test]
        fn just_above_boundary() { check_encoding(&[42; 513]); }

        #[test]
        fn just_above_double_boundary() { check_encoding(&[42; 1025]); }

        #[test]
        fn fmt_exact_macro() {
            use crate::alloc::string::ToString;

            struct Dummy([u8; 32]);

            impl fmt::Display for Dummy {
                fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    fmt_hex_exact!(f, 32, &self.0, Case::Lower)
                }
            }
            let dummy = Dummy([42; 32]);
            assert_eq!(dummy.to_string(), "2a".repeat(32));
            assert_eq!(format!("{:.10}", dummy), "2a".repeat(5));
            assert_eq!(format!("{:.11}", dummy), "2a".repeat(5) + "2");
            assert_eq!(format!("{:.65}", dummy), "2a".repeat(32));
        }

        macro_rules! define_dummy {
            ($len:literal) => {
                struct Dummy([u8; $len]);
                impl fmt::Debug for Dummy {
                    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                        fmt_hex_exact!(f, $len, &self.0, Case::Lower)
                    }
                }
                impl fmt::Display for Dummy {
                    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                        fmt_hex_exact!(f, $len, &self.0, Case::Lower)
                    }
                }
            };
        }

        macro_rules! test_display_hex {
            ($fs: expr, $a: expr, $check: expr) => {
                let array = $a;
                let slice = &$a;
                let vec = Vec::from($a);
                let dummy = Dummy($a);
                assert_eq!(format!($fs, array.as_hex()), $check);
                assert_eq!(format!($fs, slice.as_hex()), $check);
                assert_eq!(format!($fs, vec.as_hex()), $check);
                assert_eq!(format!($fs, dummy), $check);
            };
        }

        #[test]
        fn alternate_flag() {
            define_dummy!(4);

            test_display_hex!("{:#?}", [0xc0, 0xde, 0xca, 0xfe], "0xc0decafe");
            test_display_hex!("{:#}", [0xc0, 0xde, 0xca, 0xfe], "0xc0decafe");
        }

        #[test]
        fn display_short_with_padding() {
            define_dummy!(2);

            test_display_hex!("Hello {:<8}!", [0xbe, 0xef], "Hello beef    !");
            test_display_hex!("Hello {:-<8}!", [0xbe, 0xef], "Hello beef----!");
            test_display_hex!("Hello {:^8}!", [0xbe, 0xef], "Hello   beef  !");
            test_display_hex!("Hello {:>8}!", [0xbe, 0xef], "Hello     beef!");

            test_display_hex!("Hello {:<#8}!", [0xbe, 0xef], "Hello 0xbeef  !");
            test_display_hex!("Hello {:-<#8}!", [0xbe, 0xef], "Hello 0xbeef--!");
            test_display_hex!("Hello {:^#8}!", [0xbe, 0xef], "Hello  0xbeef !");
            test_display_hex!("Hello {:>#8}!", [0xbe, 0xef], "Hello   0xbeef!");
        }

        #[test]
        fn display_long() {
            define_dummy!(512);
            // Note this string is shorter than the one above.
            let a = [0xab; 512];

            let mut want = "0".repeat(2000 - 1024);
            want.extend(core::iter::repeat("ab").take(512));
            test_display_hex!("{:0>2000}", a, want);

            let mut want = "0".repeat(2000 - 1026);
            want.push_str("0x");
            want.extend(core::iter::repeat("ab").take(512));
            test_display_hex!("{:0>#2000}", a, want);
        }

        // Precision and padding act the same as for strings in the stdlib (because we use `Formatter::pad`).

        #[test]
        fn precision_truncates() {
            // Precision gets the most significant bytes.
            // Remember the integer is number of hex chars not number of bytes.
            define_dummy!(4);

            test_display_hex!("{0:.4}", [0x12, 0x34, 0x56, 0x78], "1234");
            test_display_hex!("{0:.5}", [0x12, 0x34, 0x56, 0x78], "12345");

            test_display_hex!("{0:#.4}", [0x12, 0x34, 0x56, 0x78], "0x1234");
            test_display_hex!("{0:#.5}", [0x12, 0x34, 0x56, 0x78], "0x12345");
        }

        #[test]
        fn precision_with_padding_truncates() {
            // Precision gets the most significant bytes.
            define_dummy!(4);

            test_display_hex!("{0:10.4}", [0x12, 0x34, 0x56, 0x78], "1234      ");
            test_display_hex!("{0:10.5}", [0x12, 0x34, 0x56, 0x78], "12345     ");

            test_display_hex!("{0:#10.4}", [0x12, 0x34, 0x56, 0x78], "0x1234      ");
            test_display_hex!("{0:#10.5}", [0x12, 0x34, 0x56, 0x78], "0x12345     ");
        }

        #[test]
        fn precision_with_padding_pads_right() {
            define_dummy!(4);

            test_display_hex!("{0:10.20}", [0x12, 0x34, 0x56, 0x78], "12345678  ");
            test_display_hex!("{0:10.14}", [0x12, 0x34, 0x56, 0x78], "12345678  ");

            test_display_hex!("{0:#12.20}", [0x12, 0x34, 0x56, 0x78], "0x12345678  ");
            test_display_hex!("{0:#12.14}", [0x12, 0x34, 0x56, 0x78], "0x12345678  ");
        }

        #[test]
        fn precision_with_padding_pads_left() {
            define_dummy!(4);

            test_display_hex!("{0:>10.20}", [0x12, 0x34, 0x56, 0x78], "  12345678");

            test_display_hex!("{0:>#12.20}", [0x12, 0x34, 0x56, 0x78], "  0x12345678");
        }

        #[test]
        fn precision_with_padding_pads_center() {
            define_dummy!(4);

            test_display_hex!("{0:^10.20}", [0x12, 0x34, 0x56, 0x78], " 12345678 ");

            test_display_hex!("{0:^#12.20}", [0x12, 0x34, 0x56, 0x78], " 0x12345678 ");
        }

        #[test]
        fn precision_with_padding_pads_center_odd() {
            define_dummy!(4);

            test_display_hex!("{0:^11.20}", [0x12, 0x34, 0x56, 0x78], " 12345678  ");

            test_display_hex!("{0:^#13.20}", [0x12, 0x34, 0x56, 0x78], " 0x12345678  ");
        }

        #[test]
        fn precision_does_not_extend() {
            define_dummy!(4);

            test_display_hex!("{0:.16}", [0x12, 0x34, 0x56, 0x78], "12345678");

            test_display_hex!("{0:#.16}", [0x12, 0x34, 0x56, 0x78], "0x12345678");
        }

        #[test]
        fn padding_extends() {
            define_dummy!(2);

            test_display_hex!("{:0>8}", [0xab; 2], "0000abab");

            test_display_hex!("{:0>#8}", [0xab; 2], "000xabab");
        }

        #[test]
        fn padding_does_not_truncate() {
            define_dummy!(4);

            test_display_hex!("{:0>4}", [0x12, 0x34, 0x56, 0x78], "12345678");
            test_display_hex!("{:0>4}", [0x12, 0x34, 0x56, 0x78], "12345678");

            test_display_hex!("{:0>#4}", [0x12, 0x34, 0x56, 0x78], "0x12345678");
            test_display_hex!("{:0>#4}", [0x12, 0x34, 0x56, 0x78], "0x12345678");
        }

        #[test]
        fn hex_fmt_impl_macro_forward() {
            struct Wrapper([u8; 4]);

            impl Borrow<[u8]> for Wrapper {
                fn borrow(&self) -> &[u8] { &self.0[..] }
            }

            impl_fmt_traits! {
                #[display_backward(false)]
                impl fmt_traits for Wrapper {
                    const LENGTH: usize = 4;
                }
            }

            let tc = Wrapper([0x12, 0x34, 0x56, 0x78]);

            let want = "12345678";
            let got = format!("{}", tc);
            assert_eq!(got, want);
        }

        #[test]
        fn hex_fmt_impl_macro_backwards() {
            struct Wrapper([u8; 4]);

            impl Borrow<[u8]> for Wrapper {
                fn borrow(&self) -> &[u8] { &self.0[..] }
            }

            impl_fmt_traits! {
                #[display_backward(true)]
                impl fmt_traits for Wrapper {
                    const LENGTH: usize = 4;
                }
            }

            let tc = Wrapper([0x12, 0x34, 0x56, 0x78]);

            let want = "78563412";
            let got = format!("{}", tc);
            assert_eq!(got, want);
        }

        #[test]
        fn hex_fmt_impl_macro_gen_forward() {
            struct Wrapper<T>([u8; 4], PhantomData<T>);

            impl<T: Clone> Borrow<[u8]> for Wrapper<T> {
                fn borrow(&self) -> &[u8] { &self.0[..] }
            }

            impl_fmt_traits! {
                #[display_backward(false)]
                impl<T: Clone> fmt_traits for Wrapper<T> {
                    const LENGTH: usize = 4;
                }
            }

            // We just use `u32` here as some arbitrary type that implements some arbitrary trait.
            let tc = Wrapper([0x12, 0x34, 0x56, 0x78], PhantomData::<u32>);

            let want = "12345678";
            let got = format!("{}", tc);
            assert_eq!(got, want);
        }

        #[test]
        fn hex_fmt_impl_macro_gen_backwards() {
            struct Wrapper<T>([u8; 4], PhantomData<T>);

            impl<T: Clone> Borrow<[u8]> for Wrapper<T> {
                fn borrow(&self) -> &[u8] { &self.0[..] }
            }

            impl_fmt_traits! {
                #[display_backward(true)]
                impl<T: Clone> fmt_traits for Wrapper<T> {
                    const LENGTH: usize = 4;
                }
            }

            // We just use `u32` here as some arbitrary type that implements some arbitrary trait.
            let tc = Wrapper([0x12, 0x34, 0x56, 0x78], PhantomData::<u32>);

            let want = "78563412";
            let got = format!("{}", tc);
            assert_eq!(got, want);
        }
    }

    #[cfg(feature = "std")]
    mod std {

        #[test]
        fn hex_writer() {
            use std::io::{ErrorKind, Result, Write};

            use arrayvec::ArrayString;

            use super::Case::{Lower, Upper};
            use super::{DisplayHex, HexWriter};

            macro_rules! test_hex_writer {
                ($cap:expr, $case: expr, $src: expr, $want: expr, $hex_result: expr) => {
                    let dest_buf = ArrayString::<$cap>::new();
                    let mut dest = HexWriter::new(dest_buf, $case);
                    let got = dest.write($src);
                    match $want {
                        Ok(n) => assert_eq!(got.unwrap(), n),
                        Err(e) => assert_eq!(got.unwrap_err().kind(), e.kind()),
                    }
                    assert_eq!(dest.into_inner().as_str(), $hex_result);
                };
            }

            test_hex_writer!(0, Lower, &[], Result::Ok(0), "");
            test_hex_writer!(0, Lower, &[0xab, 0xcd], Result::Err(ErrorKind::Other.into()), "");
            test_hex_writer!(1, Lower, &[0xab, 0xcd], Result::Err(ErrorKind::Other.into()), "");
            test_hex_writer!(2, Lower, &[0xab, 0xcd], Result::Ok(1), "ab");
            test_hex_writer!(3, Lower, &[0xab, 0xcd], Result::Ok(1), "ab");
            test_hex_writer!(4, Lower, &[0xab, 0xcd], Result::Ok(2), "abcd");
            test_hex_writer!(8, Lower, &[0xab, 0xcd], Result::Ok(2), "abcd");
            test_hex_writer!(8, Upper, &[0xab, 0xcd], Result::Ok(2), "ABCD");

            let vec: Vec<_> = (0u8..32).collect();
            let mut writer = HexWriter::new(String::new(), Lower);
            writer.write_all(&vec[..]).unwrap();
            assert_eq!(writer.into_inner(), vec.to_lower_hex_string());
        }
    }
}
