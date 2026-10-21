#!/usr/bin/env bash

set -euo pipefail

REPO_DIR=$(git rev-parse --show-toplevel)
LOCKFILE=${1:-minimal}

cd "$REPO_DIR"

cargo metadata --no-deps --format-version 1 \
    | jq -r '.packages[] | select(.rust_version != null) | "\(.name) \(.rust_version)"' \
    | while read -r package msrv; do
        echo "Testing $package on Rust $msrv"
        rustup toolchain install "$msrv" --profile minimal --target thumbv7m-none-eabi --no-self-update --no-update
        cargo "+$msrv" rbmt test --toolchain msrv --lockfile "$LOCKFILE" --package "$package"
    done
