#!/bin/bash

set -e

cargo clippy -- -D warnings
cargo clippy --all-targets --all-features -- -D warnings
(
    cd examples
    cargo clippy -- -D warnings
    cargo clippy --all-targets --all-features -- -D warnings
)
