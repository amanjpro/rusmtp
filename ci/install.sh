#!/usr/bin/env bash

set -o errexit -o nounset -o pipefail

sudo apt-get update
sudo apt-get install -qq gcc-arm-linux-gnueabihf
sudo apt-get install -y pkg-config libssl-dev
sudo apt-get install -y qemu-user-static


# Add targets to rustup
rustup target add armv7-unknown-linux-gnueabihf


current_directory=$(pwd)

# Install ARM openssl
mkdir -p "$HOME/openssl-1.1.1-armv7"
cd "$HOME/openssl-1.1.1-armv7"
wget https://www.openssl.org/source/openssl-1.1.1.tar.gz
tar xzf openssl-1.1.1.tar.gz
export MACHINE=armv7
export ARCH=arm
export CC=arm-linux-gnueabihf-gcc
cd openssl-1.1.1 && ./config shared && make
cd "$current_directory"

cat >"$HOME/.cargo/config" <<EOF
[target.armv7-unknown-linux-gnueabihf]
linker = "arm-linux-gnueabihf-gcc"
EOF
