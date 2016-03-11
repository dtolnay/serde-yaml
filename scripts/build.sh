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

test_with_syntex() {
    (cd yaml;
        cargo test --verbose)
    (cd yaml_tests;
        cargo test --features with-syntex --no-default-features --verbose)
}

generate_doc() {
    # build docs from master of yaml-rust because it has some rustdoc fixes
    master=https://github.com/chyh1990/yaml-rust
    (cd yaml;
        sed -i 's|^yaml-rust = ".*"$|yaml-rust = { git = "'$master'" }|' Cargo.toml
        cargo doc --verbose)
}

if [ "$TRAVIS_RUST_VERSION" = nightly ]; then
    build_with_clippy
    test_with_macros
    test_with_syntex
    generate_doc
else
    build_without_clippy
    test_with_syntex
fi
