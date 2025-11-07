#!/bin/bash
set -e

echo "Creating proper macOS app bundle..."

# Build directory
BUILD_DIR="build"
APP_NAME="WeightliftingEditor.app"
APP_PATH="$BUILD_DIR/$APP_NAME"

# Clean and create directories
rm -rf "$BUILD_DIR"
mkdir -p "$APP_PATH/Contents/MacOS"
mkdir -p "$APP_PATH/Contents/Resources"

# Build the Swift executable
echo "Building Swift executable..."
swift build -c release
cp .build/release/WeightliftingEditor "$APP_PATH/Contents/MacOS/"

# Copy Info.plist
echo "Copying Info.plist..."
cp Info.plist "$APP_PATH/Contents/"

# Copy Rust library
echo "Copying Rust FFI library..."
mkdir -p "$APP_PATH/Contents/Frameworks"
cp ../../target/release/libweightlifting_ffi.dylib "$APP_PATH/Contents/Frameworks/"

# Update library path
echo "Updating library paths..."
# Get the actual dylib path from the binary
DYLIB_PATH=$(otool -L "$APP_PATH/Contents/MacOS/WeightliftingEditor" | grep libweightlifting_ffi.dylib | awk '{print $1}')
if [ -n "$DYLIB_PATH" ]; then
    install_name_tool -change \
        "$DYLIB_PATH" \
        @executable_path/../Frameworks/libweightlifting_ffi.dylib \
        "$APP_PATH/Contents/MacOS/WeightliftingEditor"
    echo "Updated dylib path from: $DYLIB_PATH"
else
    echo "Warning: Could not find dylib reference in binary"
fi

echo ""
echo "✅ App bundle created at: $APP_PATH"
echo ""
echo "To run:"
echo "  open $APP_PATH"
echo ""
echo "Or double-click the app in Finder"

# Open the app
open "$APP_PATH"
