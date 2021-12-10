#!/bin/sh

set -eu

TARGET_DIR="target/release"
X64_TOOLCHAIN="nightly-x86_64-apple-darwin"
CARGO_OUTPUT="$TARGET_DIR/secret_store_cli"
X64_TARGET="$TARGET_DIR/secret_store_cli_x64.dylib"
FAT_TARGET="$TARGET_DIR/secret_store_cli.bundle"

rustup run $X64_TOOLCHAIN cargo build --release && mv $CARGO_OUTPUT $X64_TARGET
lipo $X64_TARGET -output $FAT_TARGET -create