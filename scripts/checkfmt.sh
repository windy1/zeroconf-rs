#!/bin/bash

set -e

find \
    examples/browser/src \
    examples/service/src \
    zeroconf/src \
    zeroconf-macros/src \
    -type f \
    -name *.rs \
    -print0 | \
    xargs -0 -n1 rustfmt --check --edition 2018 --verbose
