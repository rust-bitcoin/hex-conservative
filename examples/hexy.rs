//! Demonstrate hexadecimal encoding and decoding for a type with a natural hex representation.
//!
//! For a type where hex is supported but is not the natural representation see `./custom.rs`.
//! To wrap an array see the `./wrap_array_*` examples.

use std::fmt;
use std::str::FromStr;

use hex_conservative::{
    fmt_hex_exact, Case, DisplayHex, FromHex, HexToArrayError, HexToBytesIter, InvalidCharError,
    InvalidLengthError,
};

fn main() {
    let s = "deadbeefcafebabedeadbeefcafebabedeadbeefcafebabedeadbeefcafebabe";
    println!("Parse hex from string:  {}", s);

    let hexy = Hexy::from_hex(s).expect("the correct number of valid hex digits");
    let display = format!("{}", hexy);
    println!("Display Hexy as string: {}", display);

    assert_eq!(display, s);
}

/// A struct that always uses hex when in string form.
pub struct Hexy {
    // Some opaque data, this exampled is explicitly meant to be more than just wrapping an array.
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
    type Err = CustomFromHexError;

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
    type FromByteIterError = CustomFromByteIterError;
    type FromHexError = CustomFromHexError;

    fn from_byte_iter<I>(iter: I) -> Result<Self, Self::FromByteIterError>
    where
        I: Iterator<Item = Result<u8, InvalidCharError>> + ExactSizeIterator + DoubleEndedIterator,
    {
        // Errors if the iterator is the wrong length.
        let a = <[u8; 32] as FromHex>::from_byte_iter(iter)?;

        // An example of some application specific error.
        if a == [0; 32] {
            return Err(CustomFromByteIterError::AllZeros);
        }

        Ok(Hexy { data: a })
    }

    fn from_hex(s: &str) -> Result<Self, Self::FromHexError> {
        let expected = 32 * 2; // 2 hex characters per byte.

        // We don't want to any padding so we check the length.
        if s.len() != expected {
            return Err(
                HexToArrayError::InvalidLength(InvalidLengthError::new(s.len(), expected)).into()
            );
        }

        let iter = HexToBytesIter::new(s);
        Ok(Self::from_byte_iter(iter)?)
    }
}

/// Example error returned `from_bytes_iter`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CustomFromByteIterError {
    /// Invalid hex to bytes conversion.
    Hex(HexToArrayError),
    /// Some other application/type specific error case.
    AllZeros,
}

impl fmt::Display for CustomFromByteIterError {
    fn fmt(&self, _: &mut fmt::Formatter) -> fmt::Result { todo!() }
}

#[cfg(feature = "std")]
impl std::error::Error for CustomFromByteIterError {}

impl From<HexToArrayError> for CustomFromByteIterError {
    fn from(e: HexToArrayError) -> Self { Self::Hex(e) }
}

/// Example error returned `from_hex`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CustomFromHexError {
    /// Invalid hex to bytes conversion.
    Hex(HexToArrayError),
    /// Custom conversion error.
    Custom(CustomFromByteIterError),
}

impl fmt::Display for CustomFromHexError {
    fn fmt(&self, _: &mut fmt::Formatter) -> fmt::Result { todo!() }
}

#[cfg(feature = "std")]
impl std::error::Error for CustomFromHexError {}

impl From<HexToArrayError> for CustomFromHexError {
    fn from(e: HexToArrayError) -> Self { Self::Hex(e) }
}

impl From<CustomFromByteIterError> for CustomFromHexError {
    fn from(e: CustomFromByteIterError) -> Self { Self::Custom(e) }
}
