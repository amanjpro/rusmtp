#!/usr/bin/env bash

set -o errexit -o nounset -o pipefail

cargo test --target=x86_64-unknown-linux-gnu
cargo build --target=x86_64-unknown-linux-gnu --release
export OPENSSL_LIB_DIR=$HOME/openssl-1.1.1-armv7/openssl-1.1.1
export OPENSSL_INCLUDE_DIR=$HOME/openssl-1.1.1-armv7/openssl-1.1.1/include
export QEMU_LD_PREFIX=/usr/arm-linux-gnueabihf
export RUST_TEST_THREADS=1
cargo test --target=armv7-unknown-linux-gnueabihf
cargo build --target=armv7-unknown-linux-gnueabihf --release

