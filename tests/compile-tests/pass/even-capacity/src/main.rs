use hex::buf_encoder::BufEncoder;
use hex::Case;

// This should compile, ensuring that the compile error in odd-capacity is from the odd capacity.
fn main() { let _encoder = BufEncoder::<4>::new(Case::Lower); }
