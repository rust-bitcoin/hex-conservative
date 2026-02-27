// SPDX-License-Identifier: CC0-1.0

//! Hex encoding and decoding.

use core::{fmt, str};

#[cfg(feature = "alloc")]
use crate::alloc::vec::Vec;

#[rustfmt::skip]                // Keep public re-exports separate.
pub use crate::error::{DecodeVariableLengthBytesError, DecodeFixedLengthBytesError};

/// Trait for objects that can be deserialized from hex strings.
pub trait FromHex: Sized + sealed::Sealed {
    /// Error type returned while parsing hex string.
    type Error: Sized + fmt::Debug + fmt::Display;

    /// Produces an object from a hex string.
    ///
    /// # Errors
    ///
    /// Errors if parsing of hex string fails for any reason.
    fn from_hex(s: &str) -> Result<Self, Self::Error>;
}

#[cfg(feature = "alloc")]
impl FromHex for Vec<u8> {
    type Error = DecodeVariableLengthBytesError;

    #[inline]
    fn from_hex(s: &str) -> Result<Self, Self::Error> { crate::decode_to_vec(s) }
}

impl<const LEN: usize> FromHex for [u8; LEN] {
    type Error = DecodeFixedLengthBytesError;

    fn from_hex(s: &str) -> Result<Self, Self::Error> { crate::decode_to_array(s) }
}

mod sealed {
    /// Used to seal the `FromHex` trait.
    pub trait Sealed {}

    #[cfg(feature = "alloc")]
    impl Sealed for alloc::vec::Vec<u8> {}

    impl<const LEN: usize> Sealed for [u8; LEN] {}
}
