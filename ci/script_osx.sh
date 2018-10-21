#!/usr/bin/env bash

set -o errexit -o nounset -o pipefail

cargo test --target=x86_64-apple-darwin --release
cargo build --target=x86_64-apple-darwin --release
