# 0.2.0 - 2023-07-26 Rename library to `hex`

This release adds a `lib` section to the manifest renaming the crate's library to hex. This means
you can now specify the dependency as `hex = 0.2.0` and use it as `use hex::DisplayHex`.

- Re-name the library to `hex`[#28](https://github.com/rust-bitcoin/hex-conservative/pull/28)
- Improve unit testing [#27](https://github.com/rust-bitcoin/hex-conservative/pull/27)

# 0.1.1 - 2023-07-19

- [Add `test_hex_unwrap`](https://github.com/rust-bitcoin/hex-conservative/pull/24) hex parsing macro for test usage.
- [Improve formatting](https://github.com/rust-bitcoin/hex-conservative/pull/25) hex for bytes slices e.g., support padding.

# 0.1.0 - 2023-06-20 Initial Release

- [Import](https://github.com/rust-bitcoin/hex-conservative/pull/1) code from the `bitcoin_hashes` and `bitcoin-internals` crates.
- [Add `Iterator` implementations](https://github.com/rust-bitcoin/hex-conservative/pull/9)
