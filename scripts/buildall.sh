#!/bin/bash

cargo build --workspace --verbose
(
    cd examples
    cargo build --workspace --verbose
)
