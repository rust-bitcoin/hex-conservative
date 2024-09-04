use std::fmt;

use hex::DisplayHex;
use honggfuzz::fuzz;

/// A struct that always uses hex when in string form.
pub struct Hexy<'s> {
    // Some opaque data.
    data: &'s [u8],
}

impl<'s> Hexy<'s> {
    /// Demonstrates getting internal opaque data as a byte slice.
    pub fn as_bytes(&self) -> &[u8] { self.data }
}

impl<'s> fmt::LowerHex for Hexy<'s> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::LowerHex::fmt(&self.data.as_hex(), f)
    }
}

impl<'s> fmt::UpperHex for Hexy<'s> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::UpperHex::fmt(&self.data.as_hex(), f)
    }
}

fn do_test(data: &[u8]) {
    let hexy = Hexy { data };

    let lower = format!("{:x}", hexy);
    assert!(lower.len() % 2 == 0);
    println!("lower: {}", lower);
    for c in lower.chars() {
        assert!(c.is_ascii_lowercase() || c.is_ascii_digit());
        assert!(c.is_ascii_digit());
    }

    let lower = format!("{:X}", hexy);
    assert!(lower.len() % 2 == 0);
    for c in lower.chars() {
        assert!(c.is_ascii_uppercase() || c.is_ascii_digit());
        assert!(c.is_ascii_digit());
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
        extend_vec_from_hex("e5952d4fff", &mut a);
        super::do_test(&a);
    }
}
