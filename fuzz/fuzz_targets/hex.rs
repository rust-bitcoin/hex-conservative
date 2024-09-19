use hex::{DisplayHex, FromHex};
use honggfuzz::fuzz;

const LEN: usize = 32; // Arbitrary amount of data.

fn do_test(data: &[u8]) {
    if let Ok(s) = std::str::from_utf8(data) {
        if let Ok(hexy) = <[u8; LEN]>::from_hex(s) {
            let got = format!("{:x}", hexy.as_hex());
            assert_eq!(got, s.to_lowercase());
        }
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
