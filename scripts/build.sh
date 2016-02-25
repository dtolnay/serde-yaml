#!/bin/sh

# License: CC0 1.0 Universal
# https://creativecommons.org/publicdomain/zero/1.0/legalcode

set -ex

cargo build --verbose

cargo test --verbose

if [ "$TRAVIS_RUST_VERSION" = nightly ]; then
    cargo doc --verbose
fi
