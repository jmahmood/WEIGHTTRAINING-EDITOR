#!/bin/bash
set -e

# Build script for Rust FFI library
# This should be run as a build phase in Xcode or before building the Swift package

echo "Building Rust FFI library..."

# Navigate to project root
cd "$(dirname "$0")/../.."

# Build for current architecture
if [ "$ARCHS" = "arm64" ]; then
    cargo build --release --target aarch64-apple-darwin -p weightlifting-ffi
    RUST_TARGET="aarch64-apple-darwin"
elif [ "$ARCHS" = "x86_64" ]; then
    cargo build --release --target x86_64-apple-darwin -p weightlifting-ffi
    RUST_TARGET="x86_64-apple-darwin"
else
    # Default to native architecture
    cargo build --release -p weightlifting-ffi
    RUST_TARGET="release"
fi

echo "Rust library built successfully for $RUST_TARGET"

# Copy dylib to expected location
if [ "$RUST_TARGET" != "release" ]; then
    cp "target/$RUST_TARGET/release/libweightlifting_ffi.dylib" "target/release/"
fi

# Generate headers with cbindgen
echo "Generating C headers..."
cd crates/ffi
cargo build --release

echo "Build complete!"
