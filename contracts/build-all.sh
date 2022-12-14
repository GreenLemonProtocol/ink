#!/usr/bin/env bash

set -eu

# cargo +nightly-2022-08-15 contract build --manifest-path relayer/Cargo.toml
# cargo +nightly-2022-08-15 contract build --manifest-path verifier/Cargo.toml
# cargo +nightly-2022-08-15 contract build --manifest-path erc721/Cargo.toml

cargo +nightly contract build --manifest-path relayer/Cargo.toml
cargo +nightly contract build --manifest-path verifier/Cargo.toml
cargo +nightly contract build --manifest-path erc721/Cargo.toml

