#!/bin/sh

# Licensed under Creative Commons CC0 1.0 Universal
# <LICENSE-CC or https://creativecommons.org/publicdomain/zero/1.0/legalcode>

set -ex

build_without_clippy() {
    (cd yaml;
        cargo build --verbose)
}

build_with_clippy() {
    (cd yaml;
        cargo build --features clippy --verbose)
}

test_with_macros() {
    (cd yaml;
        cargo test --verbose)
    (cd yaml_tests;
        cargo test --verbose)
}

if [ "$TRAVIS_RUST_VERSION" = nightly ]; then
    build_with_clippy
    test_with_macros
else
    build_without_clippy
fi
