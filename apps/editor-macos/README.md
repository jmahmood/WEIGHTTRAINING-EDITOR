# Weightlifting Editor - macOS Native Version

This is the native macOS version of the Weightlifting Editor, built with SwiftUI and Rust.

## Architecture

The application consists of two main parts:

1. **Rust Backend** (`crates/ffi/`): Provides business logic and data management through a C-compatible FFI interface
2. **Swift Frontend** (`apps/editor-macos/`): Native macOS UI built with SwiftUI

The Swift app communicates with Rust through the FFI bridge, passing JSON data across the boundary.

## Prerequisites

- macOS 13.0 (Ventura) or later
- Xcode 15 or later
- Rust 1.70 or later
- Swift 5.9 or later

## Building

### Step 1: Build the Rust FFI Library

```bash
# From project root
cd /path/to/WEIGHTTRAINING-EDITOR

# Build the FFI library
cargo build --release -p weightlifting-ffi

# This will generate:
# - target/release/libweightlifting_ffi.dylib (the dynamic library)
# - crates/ffi/include/weightlifting_ffi.h (C headers)
```

For universal binary (both Intel and Apple Silicon):

```bash
# Build for both architectures
cargo build --release --target x86_64-apple-darwin -p weightlifting-ffi
cargo build --release --target aarch64-apple-darwin -p weightlifting-ffi

# Create universal binary
lipo -create \
    target/x86_64-apple-darwin/release/libweightlifting_ffi.dylib \
    target/aarch64-apple-darwin/release/libweightlifting_ffi.dylib \
    -output target/release/libweightlifting_ffi.dylib
```

### Step 2: Build the Swift Application

Using Swift Package Manager:

```bash
cd apps/editor-macos
swift build
```

Using Xcode:

1. Open `apps/editor-macos/` in Xcode
2. Ensure the Rust library has been built (see Step 1)
3. Build and run (Cmd+R)

## Project Structure

```
apps/editor-macos/
├── Package.swift                 # Swift package manifest
├── Info.plist                    # App metadata and capabilities
├── build_rust.sh                # Build script for Rust library
├── Sources/
│   ├── WeightliftingEditorApp.swift  # App entry point
│   ├── AppState.swift                # Global app state
│   ├── Models/
│   │   ├── Plan.swift               # Data models matching Rust structs
│   │   └── WeightliftingDocument.swift
│   ├── RustBridge/
│   │   ├── RustBridge.swift        # Swift wrapper for FFI calls
│   │   └── BridgingHeader.h        # C header imports
│   ├── Views/
│   │   ├── MainWindowView.swift    # Main app window
│   │   ├── CanvasView.swift        # Plan display canvas
│   │   └── RightPanelView.swift    # Exercise/group browser
│   └── Dialogs/
│       ├── SegmentEditorView.swift
│       ├── ValidationView.swift
│       └── GroupsEditorView.swift
└── Resources/                    # App icons, assets, etc.
```

## Features

### Implemented
- ✅ Document-based architecture with native file I/O
- ✅ Plan display with all segment types
- ✅ Basic segment editing (straight sets, comments)
- ✅ Validation dialog
- ✅ Exercise groups management
- ✅ Keyboard shortcuts (Cmd+S, Cmd+V)
- ✅ Native macOS menus and toolbars
- ✅ Selection management (Cmd+Click, Shift+Click)
- ✅ Context menus

### In Progress
- 🚧 Complete segment editors for all 10 types
- 🚧 Exercise search with SQLite integration
- 🚧 Drag-and-drop reordering
- 🚧 Undo/redo functionality
- 🚧 Autosave functionality
- 🚧 Day editing
- 🚧 Plan metadata editing

### Planned
- 📋 Recent files integration
- 📋 Preferences window
- 📋 Export functionality
- 📋 Sync workflow
- 📋 Media attachments
- 📋 Dark mode support
- 📋 Keyboard navigation (Tab, arrows)
- 📋 Quick Look preview support
- 📋 Spotlight integration

## FFI Interface

The Rust FFI provides these key functions:

### Plan Operations
- `ffi_plan_new()` - Create new plan
- `ffi_plan_open(path)` - Open plan from file
- `ffi_plan_save(json, path)` - Save plan to file
- `ffi_plan_validate(json)` - Validate plan structure

### Segment Operations
- `ffi_segment_add(plan, day_idx, segment)` - Add segment
- `ffi_segment_remove(plan, day_idx, seg_idx)` - Remove segment
- `ffi_segment_update(plan, day_idx, seg_idx, segment)` - Update segment

### Day Operations
- `ffi_day_add(plan, day)` - Add day
- `ffi_day_remove(plan, day_idx)` - Remove day

### Platform Paths
- `ffi_get_app_support_dir()` - Get ~/Library/Application Support/weightlifting-desktop
- `ffi_get_cache_dir()` - Get ~/Library/Caches/weightlifting-desktop
- `ffi_get_drafts_dir()` - Get drafts directory

## Data Format

The app uses JSON for plan storage, compatible with the GTK version. Plans are validated against the v0.3 schema.

Example plan structure:
```json
{
  "id": "uuid-here",
  "name": "My Training Plan",
  "version": "draft",
  "days": [
    {
      "label": "Day 1",
      "segments": [
        {
          "type": "straight",
          "exercise": {"name": "Squat"},
          "sets": 3,
          "reps": "8"
        }
      ]
    }
  ],
  "exercise_groups": []
}
```

## Development Tips

### Debugging FFI Issues

1. Check that the Rust library is built and in the correct location
2. Verify C headers are up to date (`crates/ffi/include/weightlifting_ffi.h`)
3. Use the Swift debugger to inspect FFI results
4. Check Rust logs for error messages

### Adding New FFI Functions

1. Add function to `crates/ffi/src/lib.rs`
2. Rebuild FFI library: `cargo build -p weightlifting-ffi`
3. Update `RustBridge.swift` with Swift wrapper
4. Use in SwiftUI views

### Hot Reload

SwiftUI supports hot reload in Xcode. Changes to view code are reflected immediately without rebuilding Rust.

## Testing

```bash
# Test Rust FFI
cargo test -p weightlifting-ffi

# Test Swift code
swift test

# Integration tests
# TODO: Add integration tests
```

## Troubleshooting

### Library Not Found Error
```
dyld: Library not loaded: libweightlifting_ffi.dylib
```
**Solution**: Build the Rust library first with `cargo build --release -p weightlifting-ffi`

### Header File Not Found
```
'weightlifting_ffi.h' file not found
```
**Solution**: Run `cargo build -p weightlifting-ffi` to generate headers with cbindgen

### JSON Encoding Errors
**Solution**: Ensure Swift models match Rust models exactly. Check `CodingKeys` for snake_case conversion.

## Platform Differences from GTK Version

| Feature | GTK (Linux) | macOS Native |
|---------|-------------|--------------|
| File paths | XDG (`~/.local/share`) | Apple (`~/Library/Application Support`) |
| Recent files | GNOME RecentManager | NSDocumentController |
| Keyboard | Ctrl+shortcuts | Cmd+shortcuts |
| File dialogs | GTK FileChooser | NSOpenPanel/NSSavePanel |
| UI framework | GTK4 + libadwaita | SwiftUI |
| Preferences | JSON file | UserDefaults |

## Contributing

When adding features, maintain:
1. Parity with GTK version functionality
2. Native macOS look and feel
3. Shared Rust business logic (no duplication)
4. Clean FFI boundary with minimal crossings

## License

Same as parent project.
