//! Demonstrate custom hexadecimal encoding and decoding.
//!
//! For basic encoding and decoding see crate level rustdoc in `lib.rs`.

use std::fmt;
use std::str::FromStr;

use hex_conservative::{fmt_hex_exact, Case, FromHex, HexToArrayError, HexToBytesError};

fn main() {
    let s = "deadbeefcafebabedeadbeefcafebabedeadbeefcafebabedeadbeefcafebabe";
    let hexy = Hexy::from_hex(s).expect("the correct number of valid hex digits");
    let display = format!("{}", hexy);

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

// Note we implement `Display` and `FromStr` using `LowerHex`/`FromHex` respectively, if this was a
// not-so-hexy object then these impls would return a different string format.

impl fmt::Display for Hexy {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { fmt::LowerHex::fmt(self, f) }
}

impl FromStr for Hexy {
    type Err = HexToArrayError;

    fn from_str(s: &str) -> Result<Self, Self::Err> { Hexy::from_hex(s) }
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

// And use a fixed size array to convert from hex.

impl FromHex for Hexy {
    type Err = HexToArrayError;

    fn from_byte_iter<I>(iter: I) -> Result<Self, Self::Err>
    where
        I: Iterator<Item = Result<u8, HexToBytesError>> + ExactSizeIterator + DoubleEndedIterator,
    {
        // Errors if the iterator is the wrong length.
        let a = <[u8; 32] as FromHex>::from_byte_iter(iter)?;
        Ok(Hexy { data: a })
    }
}
