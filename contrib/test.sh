#!/bin/sh

set -ex

FEATURES="std alloc core2 serde"
MSRV="1\.56\.0"

cargo --version
rustc --version

# Work out if we are using a nightly toolchain.
NIGHTLY=false
if cargo --version | grep nightly >/dev/null; then
    NIGHTLY=true
fi

if cargo --version | grep ${MSRV}; then
    # `Cargo.lock` is committed and gets generated using a modern toolchain, so
    # it can contain multiple dependencies simultaneously too new for the MSRV
    # toolchain to even parse (e.g. ones using the `dep:` optional-dependency
    # syntax). `cargo update -p x --precise y` treats every package other than
    # `x` as frozen at its current (unparsable) version, so pinning them one at
    # a time deadlocks. First bulk-update all of them together, without
    # `--precise`, so the resolver is free to settle on versions the MSRV
    # toolchain can at least parse; only then pin each one precisely.
    cargo update -p memchr -p honggfuzz -p semver -p memmap2 -p syn -p serde_derive -p serde -p quote -p proc-macro2 -p unicode-ident -p ryu -p itoa -p serde_json

    cargo update -p serde_json --precise 1.0.48
    cargo update -p itoa --precise 0.4.3

    cargo update -p serde --precise 1.0.156
    cargo update -p quote --precise 1.0.0
    cargo update -p proc-macro2 --precise 1.0.63
    # memchr 2.6.0 uses edition 2021
    cargo update -p memchr --precise 2.5.0
    cargo update -p ryu --precise 1.0.0
    cargo update -p unicode-ident --precise 1.0.0
    # honggfuzz >= 0.5.56 requires edition 2021 (fuzz/ dev workspace member)
    cargo update -p honggfuzz --precise 0.5.55
    # semver >= 1.0.24 uses the `dep:` optional-dependency syntax
    cargo update -p semver --precise 1.0.23
    # memmap2 >= 0.9.11 requires rustc 1.65; honggfuzz 0.5.55 needs memmap2 0.5.x anyway
    cargo update -p memmap2 --precise 0.5.10
fi

# Make all cargo invocations verbose
export CARGO_TERM_VERBOSE=true

# Defaults / sanity checks
cargo build
cargo test

cargo run --example hexy

if [ "$DO_LINT" = true ]
then
    cargo clippy --all-features --all-targets -- -D warnings
    cargo clippy --locked --example hexy -- -D warnings
    cargo clippy --locked --example serde --features=serde -- -D warnings
fi

if [ "$DO_FEATURE_MATRIX" = true ]; then
    cargo build --locked --no-default-features
    cargo test --locked --no-default-features

    # All features
    cargo build --locked --no-default-features --features="$FEATURES"
    cargo test --locked --no-default-features --features="$FEATURES"
    # Single features
    for feature in ${FEATURES}
    do
        cargo build --locked --no-default-features --features="$feature"
        cargo test --locked --no-default-features --features="$feature"
		# All combos of two features
		for featuretwo in ${FEATURES}; do
			cargo build --locked --no-default-features --features="$feature $featuretwo"
			cargo test --locked --no-default-features --features="$feature $featuretwo"
		done
    done
fi

cargo run --locked --example hexy
cargo run --locked --example custom
cargo run --locked --example wrap_array_display_hex_trait
cargo run --locked --example wrap_array_fmt_traits

# Build the docs if told to (this only works with the nightly toolchain)
if [ "$DO_DOCSRS" = true ]; then
    RUSTDOCFLAGS="--cfg docsrs -D warnings -D rustdoc::broken-intra-doc-links" cargo +nightly doc --all-features
fi

# Build the docs with a stable toolchain, in unison with the DO_DOCSRS command
# above this checks that we feature guarded docs imports correctly.
if [ "$DO_DOCS" = true ]; then
    RUSTDOCFLAGS="-D warnings" cargo +stable doc --all-features
fi

# Run formatter if told to.
if [ "$DO_FMT" = true ]; then
    if [ "$NIGHTLY" = false ]; then
        echo "DO_FMT requires a nightly toolchain (consider using RUSTUP_TOOLCHAIN)"
        exit 1
    fi
    rustup component add rustfmt
    cargo fmt --check
fi
