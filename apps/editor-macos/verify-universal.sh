#!/bin/bash
set -e

APP_PATH="build/WeightliftingEditor.app"

echo "Verifying app bundle architectures..."
echo ""

# Check if app bundle exists
if [ ! -d "$APP_PATH" ]; then
    echo "Error: App bundle not found at $APP_PATH"
    echo "Please run create-app-bundle.sh first"
    exit 1
fi

# Check Swift executable
SWIFT_EXEC="$APP_PATH/Contents/MacOS/WeightliftingEditor"
if [ ! -f "$SWIFT_EXEC" ]; then
    echo "Error: Swift executable not found at $SWIFT_EXEC"
    exit 1
fi

echo "Swift executable:"
lipo -info "$SWIFT_EXEC"
echo ""

# Check Rust dylib
RUST_DYLIB="$APP_PATH/Contents/Frameworks/libweightlifting_ffi.dylib"
if [ ! -f "$RUST_DYLIB" ]; then
    echo "Error: Rust FFI library not found at $RUST_DYLIB"
    exit 1
fi

echo "Rust FFI library:"
lipo -info "$RUST_DYLIB"
echo ""

# Extract architectures and verify they match
SWIFT_ARCHS=$(lipo -info "$SWIFT_EXEC" | sed 's/.*: //')
RUST_ARCHS=$(lipo -info "$RUST_DYLIB" | sed 's/.*: //')

if [ "$SWIFT_ARCHS" = "$RUST_ARCHS" ]; then
    echo "✓ Architectures match: $SWIFT_ARCHS"

    # Verify we have both expected architectures
    if echo "$SWIFT_ARCHS" | grep -q "x86_64" && echo "$SWIFT_ARCHS" | grep -q "arm64"; then
        echo "✓ Universal binary contains both x86_64 and arm64"
    else
        echo "⚠ Warning: Expected both x86_64 and arm64, got: $SWIFT_ARCHS"
        exit 1
    fi
else
    echo "✗ Architecture mismatch!"
    echo "  Swift: $SWIFT_ARCHS"
    echo "  Rust:  $RUST_ARCHS"
    exit 1
fi

echo ""
echo "✓ Universal binary verification passed!"
