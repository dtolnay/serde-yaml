#!/bin/sh

# License: CC0 1.0 Universal
# https://creativecommons.org/publicdomain/zero/1.0/legalcode

set -ex

if [ "$TRAVIS_RUST_VERSION" = nightly ]; then
    cargo build --features clippy --verbose

    cargo test --features clippy --verbose

    cargo doc --verbose
else
    cargo build --verbose
fi
