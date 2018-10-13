#!/usr/bin/env bash

set -o errexit -o nounset -o pipefail

archive() {
  local version="$0"
  local arch="$1"
  local dist="rusmtp-$version-$arch"

  rm -rf "$dist"
  mkdir -p "$dist"
  cp "target/$arch/release/rusmtpc" "$dist/"
  cp "target/$arch/release/rusmtpd" "$dist/"
  cp distribution/rusmtprc.default "$dist/"
  cp distribution/install "$dist/"
  cp distribution/uninstall "$dist/"
  cp COPYING "$dist/"
  cp README.md "$dist/"
  cp doc/rusmtpd.1 "$dist/"
  cp doc/rusmtpc.1 "$dist/"

  tar -czf "archives/$dist.tar.gz" "$dist"
}

mkdir -p archives
VERSION="$(git describe --tags $(git rev-list --tags --max-count=1))"
archive "$VERSION" x86_64-unknown-linux-gnu
archive "$VERSION" x86_64-apple-darwin
archive "$VERSION" armv7-unknown-linux-gnueabihf
