// SPDX-License-Identifier: CC0-1.0

//! Test the API surface of `hex-conservative`.
//!
//! The point of these tests is to check the API surface as opposed to test the API functionality.
//!
//! ref: <https://rust-lang.github.io/api-guidelines/about.html>

#![allow(dead_code)]
#![allow(unused_imports)]

use core::borrow::Borrow;
use core::marker::PhantomData;
use core::{fmt, slice};

#[cfg(feature = "serde")]
use hex_conservative::serde;
// These imports test "typical" usage by user code.
use hex_conservative::{
    buf_encoder, display, BytesToHexIter, Case, DisplayHex as _, HexToArrayError, HexToBytesError,
    InvalidCharError, InvalidLengthError, OddLengthStringError, ToArrayError, ToBytesError,
};

/// A struct that includes all public non-error enums.
// C-COMMON-TRAITS excluding `Display`.
// All public types implement Debug (C-DEBUG).
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
struct Enums {
    a: Case,
}

impl Enums {
    fn new() -> Self { Self { a: Case::Lower } }
}

// Some arbitrary data to use.
const HEX: &str = "deadbeef";
const BYTES: [u8; 8] = [0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07];
const CAP: usize = 16; // BYTES.len() * 2

/// A struct that includes all public non-error structs.
#[derive(Debug)] // All public types implement Debug (C-DEBUG).
struct Structs<'a, I, T>
where
    I: Iterator,
    I::Item: Borrow<u8>,
    T: fmt::Write,
{
    a: BytesToHexIter<I>,
    b: buf_encoder::BufEncoder<CAP>,
    c: display::DisplayArray<'a, CAP>,
    d: display::DisplayByteSlice<'a>,
    #[cfg(feature = "std")]
    e: display::HexWriter<T>,
    #[cfg(feature = "serde")]
    f: serde::SerializeBytesAsHex<'a>,
    #[cfg(feature = "serde")]
    g: serde::SerializeBytesAsHexLower<'a>,
    #[cfg(feature = "serde")]
    h: serde::SerializeBytesAsHexUpper<'a>,
    _i: PhantomData<T>, // For when `std` is not enabled.
}

impl Structs<'_, slice::Iter<'_, u8>, String> {
    /// Constructs an arbitrary instance.
    fn new() -> Self {
        let iter = BYTES.iter();
        Self {
            a: BytesToHexIter::new(iter, Case::Lower),
            b: buf_encoder::BufEncoder::new(Case::Lower),
            c: BYTES.as_hex(),
            d: BYTES[..].as_hex(),
            #[cfg(feature = "std")]
            e: display::HexWriter::new(String::new(), Case::Lower),
            #[cfg(feature = "serde")]
            f: serde::SerializeBytesAsHex(&BYTES),
            #[cfg(feature = "serde")]
            g: serde::SerializeBytesAsHexLower(&BYTES),
            #[cfg(feature = "serde")]
            h: serde::SerializeBytesAsHexUpper(&BYTES),
            _i: PhantomData,
        }
    }
}

/// A struct that includes all public error types.
// These derives are the policy of `rust-bitcoin` not Rust API guidelines.
#[derive(Debug, Clone, PartialEq, Eq)] // All public types implement Debug (C-DEBUG).
struct Errors {
    a: ToArrayError,
    b: ToBytesError,
    c: HexToArrayError,
    d: HexToBytesError,
    e: InvalidCharError,
    f: InvalidLengthError,
    g: OddLengthStringError,
}

// `Debug` representation is never empty (C-DEBUG-NONEMPTY).
#[test]
fn api_all_non_error_types_have_non_empty_debug() {
    let debug = format!("{:?}", Case::Lower);
    assert!(!debug.is_empty());

    let t = Structs::new();

    let debug = format!("{:?}", t.a);
    assert!(!debug.is_empty());
    let debug = format!("{:?}", t.b);
    assert!(!debug.is_empty());
    let debug = format!("{:?}", t.c);
    assert!(!debug.is_empty());
    let debug = format!("{:?}", t.d);
    assert!(!debug.is_empty());
    #[cfg(feature = "std")]
    let debug = format!("{:?}", t.e);
    assert!(!debug.is_empty());
    #[cfg(feature = "serde")]
    {
        let debug = format!("{:?}", t.f);
        assert!(!debug.is_empty());
        let debug = format!("{:?}", t.g);
        assert!(!debug.is_empty());
        let debug = format!("{:?}", t.h);
        assert!(!debug.is_empty());
    }
}

#[test]
fn all_types_implement_send_sync() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    //  Types are `Send` and `Sync` where possible (C-SEND-SYNC).
    assert_send::<Enums>();
    assert_sync::<Enums>();
    assert_send::<Structs<'_, slice::Iter<'_, u8>, String>>();
    assert_sync::<Structs<'_, slice::Iter<'_, u8>, String>>();

    // Error types should implement the Send and Sync traits (C-GOOD-ERR).
    assert_send::<Errors>();
    assert_sync::<Errors>();
}
