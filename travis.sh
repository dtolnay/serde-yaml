#!/bin/bash

set -e

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

channel() {
    if [ -n "${TRAVIS}" ]; then
        if [ "${TRAVIS_RUST_VERSION}" = "${CHANNEL}" ]; then
            pwd
            (set -x; cargo "$@")
        fi
    else
        pwd
        (set -x; cargo "+${CHANNEL}" "$@")
    fi
}

if [ -n "${CLIPPY}" ]; then
    # cached installation will not work on a later nightly
    if [ -n "${TRAVIS}" ] && ! cargo install clippy --debug --force; then
        echo "COULD NOT COMPILE CLIPPY, IGNORING CLIPPY TESTS"
        exit
    fi

    cd "$DIR/yaml"
    cargo clippy -- -Dclippy

    cd "$DIR/yaml_tests"
    cargo clippy -- -Dclippy
else
    for CHANNEL in nightly beta stable; do
        cargo clean
        cd "$DIR/yaml"
        channel build
        channel test
        cd "$DIR/yaml_tests"
        channel test
    done
fi
