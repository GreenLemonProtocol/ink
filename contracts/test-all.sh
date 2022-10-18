#!/usr/bin/env bash

set -eu

cargo test --manifest-path relayer/Cargo.toml
cargo test --manifest-path verifier/Cargo.toml
cargo test --manifest-path erc721/Cargo.toml