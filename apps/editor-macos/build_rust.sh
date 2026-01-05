#!/bin/bash
set -e

# Build script for Rust FFI library
# This should be run as a build phase in Xcode or before building the Swift package

echo "Building Rust FFI library..."

# Navigate to project root
cd "$(dirname "$0")/../.."

LIB_NAME="libweightlifting_ffi.dylib"

build_target() {
    local arch="$1"
    local target="$2"
    echo "Building Rust FFI for $arch ($target)..."
    cargo build --release --target "$target" -p weightlifting-ffi
}

RUST_LIB_PATHS=()

if [ -n "$ARCHS" ]; then
    for arch in $ARCHS; do
        case "$arch" in
            arm64)
                build_target "$arch" "aarch64-apple-darwin"
                RUST_LIB_PATHS+=("target/aarch64-apple-darwin/release/$LIB_NAME")
                ;;
            x86_64)
                build_target "$arch" "x86_64-apple-darwin"
                RUST_LIB_PATHS+=("target/x86_64-apple-darwin/release/$LIB_NAME")
                ;;
            *)
                echo "Warning: Unsupported ARCHS entry '$arch' - skipping"
                ;;
        esac
    done
else
    echo "ARCHS not set; building for native target..."
    cargo build --release -p weightlifting-ffi
    RUST_LIB_PATHS+=("target/release/$LIB_NAME")
fi

mkdir -p target/release
if [ "${#RUST_LIB_PATHS[@]}" -gt 1 ]; then
    echo "Creating universal dylib..."
    lipo -create "${RUST_LIB_PATHS[@]}" -output "target/release/$LIB_NAME"
elif [ "${#RUST_LIB_PATHS[@]}" -eq 1 ]; then
    cp "${RUST_LIB_PATHS[0]}" "target/release/$LIB_NAME"
else
    echo "Error: no Rust dylib produced"
    exit 1
fi

echo "Rust library built successfully at target/release/$LIB_NAME"

# Copy dylib into app bundle if build products are available (Xcode build phase)
if [ -n "$TARGET_BUILD_DIR" ] && [ -n "$FRAMEWORKS_FOLDER_PATH" ]; then
    echo "Copying dylib into app bundle frameworks..."
    mkdir -p "$TARGET_BUILD_DIR/$FRAMEWORKS_FOLDER_PATH"
    cp "target/release/$LIB_NAME" "$TARGET_BUILD_DIR/$FRAMEWORKS_FOLDER_PATH/"
fi

# Copy dylib into local build app bundle if it exists
APP_FRAMEWORKS="apps/editor-macos/build/WeightliftingEditor.app/Contents/Frameworks"
if [ -d "$APP_FRAMEWORKS" ]; then
    echo "Copying dylib into apps/editor-macos build app bundle..."
    cp "target/release/$LIB_NAME" "$APP_FRAMEWORKS/"
fi

echo "Build complete!"
