#!/bin/bash

set -e

cargo build --workspace --verbose
(
    cd examples
    cargo build --workspace --verbose
)
