// SPDX-License-Identifier: CC0-1.0

//! Demonstrate hexadecimal encoding and decoding for a type with a natural hex representation.
//!
//! For a type where hex is supported but is not the natural representation see `./custom.rs`.
//! To wrap an array see the `./wrap_array.rs` example.

use std::fmt;
use std::str::FromStr;

use hex_conservative::{
    fmt_hex_exact, Case, DecodeFixedLengthBytesError, DisplayHex as _, FromHex as _,
};

fn main() {
    let s = "deadbeefcafebabedeadbeefcafebabedeadbeefcafebabedeadbeefcafebabe";
    println!("Parse hex from string:  {}", s);

    let hexy = s.parse::<Hexy>().expect("the correct number of valid hex digits");
    let display = format!("{}", hexy);
    println!("Display Hexy as string: {}", display);

    assert_eq!(display, s);
}

/// A struct that always uses hex when in string form.
pub struct Hexy {
    // Some opaque data.
    data: [u8; 32],
}

impl Hexy {
    /// Demonstrates getting internal opaque data as a byte slice.
    pub fn as_bytes(&self) -> &[u8] { &self.data }
}

impl fmt::Debug for Hexy {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        fmt::Formatter::debug_struct(f, "Hexy").field("data", &self.data.as_hex()).finish()
    }
}

// We implement `Display`/`FromStr` using `LowerHex`/`FromHex` respectively, if hex was not the
// natural representation for this type this would not be the case.

impl fmt::Display for Hexy {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { fmt::LowerHex::fmt(self, f) }
}

impl FromStr for Hexy {
    type Err = DecodeFixedLengthBytesError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Errors if the input is invalid
        let a = <[u8; 32]>::from_hex(s)?;
        Ok(Hexy { data: a })
    }
}

// Implement conversion to hex by first converting our type to a byte slice.

impl fmt::LowerHex for Hexy {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // This is equivalent to but more performant than:
        // fmt::LowerHex::fmt(&self.as_bytes().as_hex(), f)
        fmt_hex_exact!(f, 32, self.as_bytes(), Case::Lower)
    }
}

impl fmt::UpperHex for Hexy {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // This is equivalent to but more performant than:
        // fmt::UpperHex::fmt(&self.as_bytes().as_hex(), f)
        fmt_hex_exact!(f, 32, self.as_bytes(), Case::Upper)
    }
}
