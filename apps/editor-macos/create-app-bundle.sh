#!/bin/bash
set -e

echo "Creating proper macOS app bundle (universal binary)..."

# Build directory
BUILD_DIR="build"
APP_NAME="WeightliftingEditor.app"
APP_PATH="$BUILD_DIR/$APP_NAME"

# Clean and create directories
rm -rf "$BUILD_DIR"
mkdir -p "$APP_PATH/Contents/MacOS"
mkdir -p "$APP_PATH/Contents/Resources"

# Build the Rust FFI library first (universal binary)
echo "Building Rust FFI library for arm64 and x86_64..."
cd ../../
ARCHS="arm64 x86_64" apps/editor-macos/build_rust.sh
cd apps/editor-macos

# Build the Swift executable (universal binary)
echo "Building Swift executable for arm64 and x86_64..."
./build_swift_universal.sh release
cp .build/universal-apple-macosx/release/WeightliftingEditor "$APP_PATH/Contents/MacOS/"

# Copy Info.plist
echo "Copying Info.plist..."
cp Info.plist "$APP_PATH/Contents/"

# Copy app icon if it exists
if [ -f "AppIcon.icns" ]; then
    echo "Copying app icon..."
    cp AppIcon.icns "$APP_PATH/Contents/Resources/"
else
    echo "Warning: AppIcon.icns not found - app will use default icon"
fi

# Copy Rust library
echo "Copying Rust FFI library..."
mkdir -p "$APP_PATH/Contents/Frameworks"
cp ../../target/release/libweightlifting_ffi.dylib "$APP_PATH/Contents/Frameworks/"

# Update library path and rpath
echo "Updating library paths and rpath..."

# Add rpath if not already present (should be from linker settings, but be safe)
install_name_tool -add_rpath @executable_path/../Frameworks "$APP_PATH/Contents/MacOS/WeightliftingEditor" 2>/dev/null || true

# Get the actual dylib path from the binary
DYLIB_PATH=$(otool -L "$APP_PATH/Contents/MacOS/WeightliftingEditor" | grep libweightlifting_ffi.dylib | awk '{print $1}')
if [ -n "$DYLIB_PATH" ]; then
    echo "Dylib reference found: $DYLIB_PATH"
    # If it's not already @rpath, change it
    if [[ "$DYLIB_PATH" != "@rpath/"* ]]; then
        install_name_tool -change \
            "$DYLIB_PATH" \
            @rpath/libweightlifting_ffi.dylib \
            "$APP_PATH/Contents/MacOS/WeightliftingEditor"
        echo "Updated dylib path from: $DYLIB_PATH to @rpath/libweightlifting_ffi.dylib"
    else
        echo "Dylib path already uses @rpath"
    fi
else
    echo "Warning: Could not find dylib reference in binary"
fi

# === Signing / Notarization settings ===
# Set these environment variables to sign and notarize the app:
# - CODESIGN_IDENTITY: Your Developer ID (e.g., "Developer ID Application: Your Name (TEAMID)")
# - NOTARY_KEYCHAIN_PROFILE: Keychain profile created with `xcrun notarytool store-credentials`
# - BUNDLE_ID: Your app's bundle identifier (optional, defaults to com.weightlifting.editor)
#
# To skip signing/notarization, leave CODESIGN_IDENTITY unset or empty.
#
# Example with signing:
#   export CODESIGN_IDENTITY="Developer ID Application: Your Name (TEAMID)"
#   export NOTARY_KEYCHAIN_PROFILE="NotaryProfile"
#   ./create-app-bundle.sh

CODESIGN_IDENTITY="${CODESIGN_IDENTITY:-}"
NOTARY_KEYCHAIN_PROFILE="${NOTARY_KEYCHAIN_PROFILE:-}"
BUNDLE_ID="${BUNDLE_ID:-com.weightlifting.editor}"

if [ -n "$CODESIGN_IDENTITY" ]; then
    echo ""
    echo "Code signing enabled with identity: $CODESIGN_IDENTITY"
    echo ""

    echo "Signing nested code (dylib)..."
    codesign --force --timestamp --options runtime \
      --sign "$CODESIGN_IDENTITY" \
      "$APP_PATH/Contents/Frameworks/libweightlifting_ffi.dylib"

    echo "Signing main executable..."
    codesign --force --timestamp --options runtime \
      --sign "$CODESIGN_IDENTITY" \
      "$APP_PATH/Contents/MacOS/WeightliftingEditor"

    echo "Signing app bundle..."
    codesign --force --timestamp --options runtime \
      --sign "$CODESIGN_IDENTITY" \
      "$APP_PATH"

    echo "Verifying signature..."
    codesign --verify --strict --verbose=2 "$APP_PATH"
    spctl -a -vv "$APP_PATH" || true

    # Notarization (only if keychain profile is set)
    if [ -n "$NOTARY_KEYCHAIN_PROFILE" ]; then
        echo ""
        echo "Notarization enabled with profile: $NOTARY_KEYCHAIN_PROFILE"
        echo ""

        echo "Creating notarization ZIP..."
        ZIP_PATH="$BUILD_DIR/WeightliftingEditor.zip"
        ditto -c -k --keepParent "$APP_PATH" "$ZIP_PATH"

        echo "Submitting for notarization..."
        xcrun notarytool submit "$ZIP_PATH" \
          --keychain-profile "$NOTARY_KEYCHAIN_PROFILE" \
          --wait

        echo "Stapling notarization ticket..."
        xcrun stapler staple "$APP_PATH"

        echo "Final Gatekeeper assessment..."
        spctl -a -vv "$APP_PATH"
    else
        echo ""
        echo "⚠️  Notarization skipped (NOTARY_KEYCHAIN_PROFILE not set)"
        echo "   App is signed but not notarized - may show warnings on other Macs"
        echo ""
    fi
else
    echo ""
    echo "⚠️  Code signing disabled (CODESIGN_IDENTITY not set)"
    echo "   App will only run on this Mac and will show security warnings"
    echo "   To enable signing, set CODESIGN_IDENTITY environment variable"
    echo ""
fi

echo ""
echo "Verifying universal binary architectures..."
./verify-universal.sh

echo ""
echo "✅ App bundle created at: $APP_PATH"
echo ""
echo "To run:"
echo "  open $APP_PATH"
echo ""
echo "Or double-click the app in Finder"

# Open the app
# open "$APP_PATH"
