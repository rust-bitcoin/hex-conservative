# Hex decoding

The decoding half of [`hex-conservative`](https://crates.io/crates/hex-conservative): hex to bytes
iterators, `decode_to_vec`, `decode_to_array` and their error types.

This crate exists so that `hex-conservative` `0.2.x` and `1.x` can share the same decoding error
types without forcing the MSRV of the `1.x` encoding code (which uses GATs) on the `0.2.x` line.
`hex-conservative` re-exports everything here, so most users should depend on that crate instead.

```
let bytes = hex_conservative_decoding::decode_to_array::<4>("deadbeef").expect("valid hex");
assert_eq!(bytes, [0xde, 0xad, 0xbe, 0xef]);
```

## Crate feature flags

* `std` - enables the standard library, on by default.
* `alloc` - enables decoding into `Vec<u8>`, implied by `std`.
* `newer-rust-version` - enables Rust version detection so that the error types implement
  `core::error::Error` on Rust 1.81 and newer when `std` is disabled.

## Minimum Supported Rust Version (MSRV)

The MSRV is Rust `1.60.0`, lower than `hex-conservative` itself, so that the older release lines can
depend on this crate without raising their MSRV.
