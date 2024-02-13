//! This is based on `examples::hexy`.

use std::fmt;
use std::str::FromStr;

use hex::{fmt_hex_exact, Case, FromHex, FromHexError, HexToArrayError, InvalidCharError};
use honggfuzz::fuzz;

const LEN: usize = 32; // Arbitrary amount of data.

/// A struct that always uses hex when in string form.
pub struct Hexy {
    // Some opaque data, this exampled is explicitly meant to be more than just wrapping an array
    data: [u8; LEN],
}

impl Hexy {
    /// Demonstrates getting internal opaque data as a byte slice.
    pub fn as_bytes(&self) -> &[u8] { &self.data }
}

impl fmt::Display for Hexy {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { fmt::LowerHex::fmt(self, f) }
}

impl FromStr for Hexy {
    type Err = FromHexError<HexToArrayError>;

    fn from_str(s: &str) -> Result<Self, Self::Err> { Hexy::from_hex(s) }
}

impl fmt::LowerHex for Hexy {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt_hex_exact!(f, 32, self.as_bytes(), Case::Lower)
    }
}

impl fmt::UpperHex for Hexy {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt_hex_exact!(f, 32, self.as_bytes(), Case::Upper)
    }
}

impl FromHex for Hexy {
    type Error = HexToArrayError;

    fn from_byte_iter<I>(iter: I) -> Result<Self, Self::Error>
    where
        I: Iterator<Item = Result<u8, InvalidCharError>> + ExactSizeIterator + DoubleEndedIterator,
    {
        // Errors if the iterator is the wrong length.
        let a = <[u8; 32] as FromHex>::from_byte_iter(iter)?;
        Ok(Hexy { data: a })
    }
}

fn do_test(data: &[u8]) {
    match std::str::from_utf8(data) {
        Ok(s) => match Hexy::from_str(s) {
            Ok(hexy) => {
                let got = format!("{:x}", hexy);
                assert_eq!(got, s.to_lowercase());
            }
            Err(_) => return,
        },
        Err(_) => return,
    }
}

fn main() {
    loop {
        fuzz!(|d| { do_test(d) });
    }
}

#[cfg(all(test, fuzzing))]
mod tests {
    fn extend_vec_from_hex(hex: &str, out: &mut Vec<u8>) {
        let mut b = 0;
        for (idx, c) in hex.as_bytes().iter().enumerate() {
            b <<= 4;
            match *c {
                b'A'..=b'F' => b |= c - b'A' + 10,
                b'a'..=b'f' => b |= c - b'a' + 10,
                b'0'..=b'9' => b |= c - b'0',
                _ => panic!("Bad hex"),
            }
            if (idx & 1) == 1 {
                out.push(b);
                b = 0;
            }
        }
    }

    #[test]
    fn duplicate_crash() {
        let mut a = Vec::new();
        extend_vec_from_hex("41414141414141414141414141414141414141414141414141414141414141414141414241414141414141414141414141414141414141414141414141414141", &mut a);
        super::do_test(&a);
    }
}
