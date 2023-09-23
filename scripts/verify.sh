#!/bin/bash

set -e
set -o pipefail

./scripts/buildall.sh
./scripts/checkfmt.sh
./scripts/lintall.sh
cargo test
