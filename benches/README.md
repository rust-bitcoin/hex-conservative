 # Hex Conservative Benchmarks

 Criterion based benchmarks for `hex-conservative`.

 ## Minimum Supported Rust Version (MSRV)

 This crate's MSRV is determined by its Criterion dependency. It currently requires **Rust 1.81**.
 This higher MSRV applies only to the benches crate and does not affect the main crate.

 ## Running the benchmarks

 Examples below are run from within the crate folder `benches/`, if running from the repo root pass in `--manifest-path benches/Cargo.toml`.

 Run all benchmarks in this crate:

 ```bash
 cargo bench
 ```

 Run the iterator benchmark target:

 ```bash
 cargo bench --bench iter
 ```

 Pass options through to Criterion:

 ```bash
 cargo bench --bench iter -- --save-baseline before-change
 cargo bench --bench iter -- --baseline before-change
 ```

 View reports:

 - Criterion writes detailed html reports that are linked to in `target/criterion/report/index.html`.

 ## Licensing

 The code in this project is licensed under the [Creative Commons CC0 1.0 Universal license](../LICENSE).
 We use the [SPDX license list](https://spdx.org/licenses/) and [SPDX IDs](https://spdx.dev/ids/).
 