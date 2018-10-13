#!/usr/bin/env bash

set -o errexit -o nounset -o pipefail

sudo apt-get update
sudo apt-get install -y clang-3.4
sudo apt-get install -qq gcc-arm-linux-gnueabihf
sudo apt-get install -y pkg-config libssl-dev
sudo apt-get update


# Add targets to rustup
rustup target add x86_64-apple-darwin
rustup target add armv7-unknown-linux-gnueabihf


# Install OSX cross device tools
git clone https://github.com/tpoechtrager/osxcross
cd osxcross
wget https://s3.dockerproject.org/darwin/v2/MacOSX10.11.sdk.tar.xz
mv MacOSX10.11.sdk.tar.xz tarballs/
sed -i -e 's|-march=native||g' build_clang.sh wrapper/build.sh
UNATTENDED=yes OSX_VERSION_MIN=10.7 ./build.sh
sudo mkdir -p /usr/local/osx-ndk-x86
sudo mv target/* /usr/local/osx-ndk-x86
cd ..
rm -rf osxcross


# Install ARM openssl
current_directory=$(pwd)
cd /tmp
wget https://www.openssl.org/source/openssl-1.1.1.tar.gz
tar xzf openssl-1.1.1.tar.gz
export MACHINE=armv7
export ARCH=arm
export CC=arm-linux-gnueabihf-gcc
cd openssl-1.1.1 && ./config shared && make && cd -
cd "$current_directory"

cat >$HOME/.cargo/config <<EOF
[target.x86_64-apple-darwin]
linker = "x86_64-apple-darwin15-cc"
ar = "x86_64-apple-darwin15-ar"

[target.armv7-unknown-linux-gnueabihf]
linker = "arm-linux-gnueabihf-gcc"
EOF
