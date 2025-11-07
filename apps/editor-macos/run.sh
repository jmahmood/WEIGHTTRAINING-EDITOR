#!/bin/bash
set -e

echo "Building Weightlifting Editor for macOS..."

# Build with xcodebuild for proper .app bundle
xcodebuild -scheme WeightliftingEditor \
    -configuration Debug \
    build

echo ""
echo "Build complete! Opening app..."
echo ""

# Find and run the built app
APP_PATH=$(find ~/Library/Developer/Xcode/DerivedData -name "WeightliftingEditor.app" -type d 2>/dev/null | head -1)

if [ -n "$APP_PATH" ]; then
    echo "Found app at: $APP_PATH"
    open "$APP_PATH"
else
    echo "❌ Could not find built app"
    echo ""
    echo "Try opening in Xcode instead:"
    echo "  open Package.swift"
    echo "  Then press Cmd+R to build and run"
fi
