#!/bin/sh

set -ex

FEATURES="std alloc core2"
MSRV="1\.48\.0"

cargo --version
rustc --version

# Work out if we are using a nightly toolchain.
NIGHTLY=false
if cargo --version | grep nightly >/dev/null; then
    NIGHTLY=true
fi

if cargo --version | grep ${MSRV}; then
    # memchr 2.6.0 uses edition 2021
    cargo update -p memchr --precise 2.5.0
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
