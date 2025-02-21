// SPDX-License-Identifier: CC0-1.0

//! Error code for the `hex-conservative` crate.

// We moved the error types to a micro crate so we could stabalize them more quickly.
#[doc(inline)]
pub use hex_conservative_errors::{
    HexToArrayError, HexToBytesError, InvalidCharError, InvalidLengthError, OddLengthStringError,
    ToArrayError, ToBytesError,
};
