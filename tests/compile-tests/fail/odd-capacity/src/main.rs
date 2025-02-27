use hex::buf_encoder::BufEncoder;
use hex::Case;

// This should fail to compile because the capacity size is odd.
// The only difference to `even-capacity` is the capacity size.
fn main() { let _encoder = BufEncoder::<3>::new(Case::Lower); }
