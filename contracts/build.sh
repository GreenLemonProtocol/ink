#!/usr/bin/env bash
set -eu
cd `dirname $0`
cargo +nightly contract build --manifest-path anonymous/Cargo.toml
cargo +nightly contract build --manifest-path verifier/Cargo.toml
# cargo +nightly contract build