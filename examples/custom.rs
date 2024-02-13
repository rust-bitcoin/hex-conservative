//! Demonstrate hexadecimal encoding and decoding for a type where hex is not the natural hex
//! representation but the type can still be encoded/decoded to/from hex.
//!
//! For a type where hex is the natural representation see `./hexy.rs`.
//! To wrap an array see the `./wrap_array_*` examples.

use core::fmt;
use core::str::FromStr;

use hex_conservative::{DisplayHex, FromHex, HexToArrayError, InvalidCharError};

fn main() {
    let s = "deadbeefcafebabedeadbeefcafebabedeadbeefcafebabedeadbeefcafebabe";
    println!("Parse from hex: {}", s);

    let hexy = ALittleBitHexy::from_hex(s).expect("the correct number of valid hex digits");
    println!("Display ALittleBitHexy as string: {}", hexy);
    println!("Display ALittleBitHexy as a hex: {:x}", hexy.as_hex());

    #[cfg(feature = "alloc")]
    {
        let hex = hexy.to_lower_hex_string();
        let from_hex = ALittleBitHexy::from_hex(&hex).expect("failed to parse hex");
        assert_eq!(from_hex, hexy);
    }
}

/// A struct that displays using some application specific format but also supports printing as hex.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ALittleBitHexy {
    // Some opaque data that should be printed as hex.
    data: [u8; 32],
    // Some other application data.
    x: usize,
}

impl ALittleBitHexy {
    /// Example constructor.
    pub fn new(x: usize) -> Self { Self { x, data: [0_u8; 32] } }
}

impl fmt::Debug for ALittleBitHexy {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Formatter::debug_struct(f, "ALittleBitHexy")
            .field("data", &self.data.as_hex())
            .field("x", &self.x)
            .finish()
    }
}

/// `Display` uses some application specific format (and roundtrips with `FromStr`).
impl fmt::Display for ALittleBitHexy {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Some application specific format:{}", self.x)
    }
}

impl FromStr for ALittleBitHexy {
    type Err = FromStrError;
    fn from_str(_: &str) -> Result<Self, Self::Err> {
        todo!("Parse a string as formatted by `Display`")
    }
}

// If the object can be parsed from hex, implement `FromHex`.

impl FromHex for ALittleBitHexy {
    type Error = CustomError;

    fn from_byte_iter<I>(iter: I) -> Result<Self, Self::Error>
    where
        I: Iterator<Item = Result<u8, InvalidCharError>> + ExactSizeIterator + DoubleEndedIterator,
    {
        // Errors if the iterator is the wrong length.
        let data = <[u8; 32] as FromHex>::from_byte_iter(iter).map_err(CustomError::Hex)?;

        // An example of some application specific error.
        if data == [0; 32] {
            return Err(CustomError::AllZeros);
        }

        // This is a contrived example (using x==0).
        Ok(ALittleBitHexy { data, x: 0 })
    }
}

// Implement conversion to hex by implementing `DisplayHex` on a wrapper type.

impl<'a> DisplayHex for &'a ALittleBitHexy {
    type Display = DisplayALittleBitHexy<'a>;

    fn as_hex(self) -> Self::Display { DisplayALittleBitHexy { data: &self.data } }

    fn hex_reserve_suggestion(self) -> usize {
        self.data.len().checked_mul(2).expect("the string wouldn't fit into address space")
    }
}

/// Displays `ALittleBitHexy` as hex.
///
/// Created by [`<&ALittleBitHexy as DisplayHex>::as_hex`](DisplayHex::as_hex).
pub struct DisplayALittleBitHexy<'a> {
    data: &'a [u8],
}

impl<'a> fmt::Display for DisplayALittleBitHexy<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { fmt::LowerHex::fmt(self, f) }
}

impl<'a> fmt::Debug for DisplayALittleBitHexy<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { fmt::LowerHex::fmt(self, f) }
}

impl<'a> fmt::LowerHex for DisplayALittleBitHexy<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::LowerHex::fmt(&self.data.as_hex(), f)
    }
}

impl<'a> fmt::UpperHex for DisplayALittleBitHexy<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::UpperHex::fmt(&self.data.as_hex(), f)
    }
}

/// Example error returned by `FromStr`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FromStrError {}

impl fmt::Display for FromStrError {
    fn fmt(&self, _: &mut fmt::Formatter) -> fmt::Result { todo!() }
}

#[cfg(feature = "std")]
impl std::error::Error for FromStrError {}

/// Example error used by `FromHex` implementation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CustomError {
    /// Invalid hex to bytes conversion.
    Hex(HexToArrayError),
    /// Some other application/type specific error case.
    AllZeros,
}

impl fmt::Display for CustomError {
    fn fmt(&self, _: &mut fmt::Formatter) -> fmt::Result { todo!() }
}

#[cfg(feature = "std")]
impl std::error::Error for CustomError {}
