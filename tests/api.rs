// SPDX-License-Identifier: CC0-1.0

//! Test the API surface of `hex-conservative`.
//!
//! The point of these tests is to check the API surface as opposed to test the API functionality.
//!
//! ref: <https://rust-lang.github.io/api-guidelines/about.html>

#![allow(dead_code)]
#![allow(unused_imports)]

use hex_conservative::{
    DecodeFixedLengthBytesError, DecodeVariableLengthBytesError, InvalidCharError,
    InvalidLengthError, OddLengthStringError,
};

// Some arbitrary data to use.
const HEX: &str = "deadbeef";
const BYTES: [u8; 8] = [0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07];
const CAP: usize = 16; // BYTES.len() * 2

/// A struct that includes all public error types.
// These derives are the policy of `rust-bitcoin` not Rust API guidelines.
#[derive(Debug, Clone, PartialEq, Eq)] // All public types implement Debug (C-DEBUG).
struct Errors {
    c: DecodeFixedLengthBytesError,
    d: DecodeVariableLengthBytesError,
    e: InvalidCharError,
    f: InvalidLengthError,
    g: OddLengthStringError,
}

// `Debug` representation is never empty (C-DEBUG-NONEMPTY).
#[test]
fn all_types_implement_send_sync() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    // Error types should implement the Send and Sync traits (C-GOOD-ERR).
    assert_send::<Errors>();
    assert_sync::<Errors>();
}
