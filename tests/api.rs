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

/// A struct that includes all public error types.
// These derives are the policy of `hex-conservative` not Rust API guidelines.
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

const VALID: &str = "deadbeef";
const VALID_UPPER: &str = "DEADBEEF";
const VALID_MIXED: &str = "dEaDBeEf";
const TOO_SHORT: &str = "deadbee";
const TOO_LONG: &str = "deadcaffe";
const INVALID_CHAR: &str = "deadreef";
const EXPECTED: [u8; 4] = [0xde, 0xad, 0xbe, 0xef];

#[test]
fn decode_to_array() {
    assert_eq!(hex_conservative::decode_to_array::<4>(VALID).unwrap(), EXPECTED);
    assert_eq!(hex_conservative::decode_to_array::<4>(VALID_UPPER).unwrap(), EXPECTED);
    assert_eq!(hex_conservative::decode_to_array::<4>(VALID_MIXED).unwrap(), EXPECTED);
    match hex_conservative::decode_to_array::<4>(TOO_SHORT).unwrap_err() {
        DecodeFixedLengthBytesError::InvalidLength(error) => assert_eq!(error.invalid_length(), 7),
        DecodeFixedLengthBytesError::InvalidChar(_) => panic!("unexpected error"),
    }
    match hex_conservative::decode_to_array::<4>(TOO_LONG).unwrap_err() {
        DecodeFixedLengthBytesError::InvalidLength(error) => assert_eq!(error.invalid_length(), 9),
        DecodeFixedLengthBytesError::InvalidChar(_) => panic!("unexpected error"),
    }
    match hex_conservative::decode_to_array::<4>(INVALID_CHAR).unwrap_err() {
        DecodeFixedLengthBytesError::InvalidLength(_) => panic!("unexpected error"),
        DecodeFixedLengthBytesError::InvalidChar(error) => assert_eq!(error.pos(), 4),
    }
}

#[cfg(feature = "alloc")]
#[test]
fn decode_to_vec() {
    assert_eq!(hex_conservative::decode_to_vec(VALID).unwrap(), EXPECTED);
    assert_eq!(hex_conservative::decode_to_vec(VALID_UPPER).unwrap(), EXPECTED);
    assert_eq!(hex_conservative::decode_to_vec(VALID_MIXED).unwrap(), EXPECTED);
    match hex_conservative::decode_to_vec(TOO_SHORT).unwrap_err() {
        DecodeVariableLengthBytesError::OddLengthString(error) => assert_eq!(error.length(), 7),
        DecodeVariableLengthBytesError::InvalidChar(_) => panic!("unexpected error"),
    }
    match hex_conservative::decode_to_vec(TOO_LONG).unwrap_err() {
        DecodeVariableLengthBytesError::OddLengthString(error) => assert_eq!(error.length(), 9),
        DecodeVariableLengthBytesError::InvalidChar(_) => panic!("unexpected error"),
    }
    match hex_conservative::decode_to_vec(INVALID_CHAR).unwrap_err() {
        DecodeVariableLengthBytesError::OddLengthString(_) => panic!("unexpected error"),
        DecodeVariableLengthBytesError::InvalidChar(error) => assert_eq!(error.pos(), 4),
    }
}
