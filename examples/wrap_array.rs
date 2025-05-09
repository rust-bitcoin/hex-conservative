// SPDX-License-Identifier: CC0-1.0

//! Hex encode/decode a type that wraps an array.
//!
//! Creates a simple array wrapper types using implementations of the standard library `fmt` traits.

use core::fmt;
use core::str::FromStr;

use hex_conservative::{DecodeFixedLengthBytesError, DisplayHex as _, FromHex as _};

fn main() {
    let hex = "deadbeefcafebabedeadbeefcafebabedeadbeefcafebabedeadbeefcafebabe";
    println!("\nParse from hex: {}\n", hex);

    let array = <[u8; 32]>::from_hex(hex).expect("failed to parse array");
    let wrap = Wrap::from_str(hex).expect("failed to parse wrapped array from hex string");

    println!("Print an array using traits from the standard libraries `fmt` module along with the provided implementation of `DisplayHex`:\n");
    println!("LowerHex: {:x}", array.as_hex());
    println!("UpperHex: {:X}", array.as_hex());
    println!("Display: {}", array.as_hex());
    println!("Debug: {:?}", array.as_hex());
    println!("Debug pretty: {:#?}", array.as_hex());

    println!("\n");

    println!(
        "Print the wrapped array directly using traits from the standard libraries `fmt` module:\n"
    );
    println!("LowerHex: {:x}", wrap);
    println!("UpperHex: {:X}", wrap);
    println!("Display: {}", wrap);
    println!("Debug: {:?}", wrap);
    println!("Debug pretty: {:#?}", wrap);

    #[cfg(feature = "alloc")]
    {
        let array_hex = array.to_lower_hex_string();
        let other = array.as_hex().to_string();
        assert_eq!(array_hex, other);

        let wrap_hex = wrap.to_string();
        assert_eq!(array_hex, wrap_hex);
    }
}

pub struct Wrap([u8; 32]);

impl fmt::Debug for Wrap {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        fmt::Formatter::debug_tuple(f, "Wrap").field(&self.0.as_hex()).finish()
    }
}

impl fmt::Display for Wrap {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        fmt::Display::fmt(&self.0.as_hex(), f)
    }
}

impl fmt::LowerHex for Wrap {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        fmt::LowerHex::fmt(&self.0.as_hex(), f)
    }
}

impl fmt::UpperHex for Wrap {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        fmt::LowerHex::fmt(&self.0.as_hex(), f)
    }
}

impl FromStr for Wrap {
    type Err = DecodeFixedLengthBytesError;
    fn from_str(s: &str) -> Result<Self, Self::Err> { Ok(Self(<[u8; 32]>::from_hex(s)?)) }
}
