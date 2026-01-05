#!/bin/bash
set -e

# Build Swift executable for multiple architectures and create universal binary
# Usage: ./build_swift_universal.sh <configuration>
# configuration: debug or release (default: release)

CONFIGURATION="${1:-release}"
BUILD_DIR=".build"

echo "Building Swift universal binary (configuration: $CONFIGURATION)..."

# Build for arm64
echo "Building for arm64..."
swift build -c "$CONFIGURATION" --triple arm64-apple-macosx13.0

# Build for x86_64
echo "Building for x86_64..."
swift build -c "$CONFIGURATION" --triple x86_64-apple-macosx13.0

# Create universal directory
mkdir -p "$BUILD_DIR/universal-apple-macosx/$CONFIGURATION"

# Combine into universal binary
echo "Creating universal binary..."
lipo -create \
  "$BUILD_DIR/arm64-apple-macosx/$CONFIGURATION/WeightliftingEditor" \
  "$BUILD_DIR/x86_64-apple-macosx/$CONFIGURATION/WeightliftingEditor" \
  -output "$BUILD_DIR/universal-apple-macosx/$CONFIGURATION/WeightliftingEditor"

echo "Universal Swift binary created at: $BUILD_DIR/universal-apple-macosx/$CONFIGURATION/WeightliftingEditor"

# Verify with lipo
echo "Verifying architectures:"
lipo -info "$BUILD_DIR/universal-apple-macosx/$CONFIGURATION/WeightliftingEditor"
