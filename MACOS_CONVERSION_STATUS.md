# macOS Conversion Status

## Overview

This document tracks the progress of converting the Weightlifting Editor from GTK4 to a native macOS application using SwiftUI + Rust backend.

**Last Updated**: 2025-11-07
**Overall Progress**: ~60% complete (core foundation established)

---

## ✅ Phase 1: Foundation & FFI Bridge (COMPLETED)

### Rust FFI Layer
- ✅ Created `crates/ffi/` with C-compatible interface
- ✅ Implemented 20+ FFI functions for plan/segment/day operations
- ✅ Set up `cbindgen` for automatic C header generation
- ✅ Memory-safe FFI with proper string handling
- ✅ JSON serialization across FFI boundary
- ✅ Added to workspace in `Cargo.toml`

**Key Files**:
- `crates/ffi/src/lib.rs` - FFI implementation
- `crates/ffi/build.rs` - Build script with cbindgen
- `crates/ffi/cbindgen.toml` - Header generation config
- `crates/ffi/include/weightlifting_ffi.h` - Generated C headers

### Platform Abstraction
- ✅ Refactored `crates/core/src/paths.rs` for dual platform support
- ✅ macOS paths: `~/Library/Application Support/weightlifting-desktop/`
- ✅ Linux paths: `~/.local/share/weightlifting-desktop/` (unchanged)
- ✅ Conditional compilation for platform differences

---

## ✅ Phase 2: SwiftUI Application Structure (COMPLETED)

### Project Setup
- ✅ Created `apps/editor-macos/` directory structure
- ✅ Swift Package manifest (`Package.swift`)
- ✅ Info.plist with document type associations
- ✅ Build script (`build_rust.sh`) for Rust library integration
- ✅ Comprehensive README with build instructions

### Core Application Files
- ✅ `WeightliftingEditorApp.swift` - App entry point with DocumentGroup
- ✅ `AppState.swift` - Global state management (selection, dialogs, undo)
- ✅ `WeightliftingDocument.swift` - Document-based architecture with native file I/O

### Data Models
- ✅ `Plan.swift` - Complete Swift models matching Rust structures
  - Plan, Day, Segment (all 10 types)
  - Exercise, ExerciseGroup
  - ValidationError
  - Proper Codable conformance with snake_case conversion

### Rust Bridge
- ✅ `RustBridge.swift` - Swift wrapper for all FFI functions
  - Plan operations (new, open, save, validate)
  - Segment operations (add, remove, update)
  - Day operations (add, remove)
  - Platform path queries
  - Error handling with `RustBridgeError`
- ✅ `BridgingHeader.h` - C header imports

---

## ✅ Phase 3: UI Implementation (MOSTLY COMPLETE)

### Main Views
- ✅ `MainWindowView.swift` - HSplitView layout with toolbar
  - Native macOS menus and toolbar
  - Sheet-based dialogs
  - Keyboard shortcuts (Cmd+S, Cmd+V)
  - Preferences integration
- ✅ `CanvasView.swift` - Plan display canvas
  - Day headers with add segment button
  - Segment rows with all 10 types
  - Selection highlighting
  - Context menus (edit, duplicate, delete)
  - Multi-select (Cmd+Click) and range select (Shift+Click)
- ✅ `RightPanelView.swift` - Tabbed exercise/group browser
  - Exercises tab with search
  - Groups tab with management
  - Drag-and-drop support (prepared)

### Segment Rendering (All 10 Types)
- ✅ `StraightSegmentView` - Standard sets × reps
- ✅ `RPESegmentView` - RPE-based training
- ✅ `PercentageSegmentView` - Percentage-based
- ✅ `AMRAPSegmentView` - As many reps as possible
- ✅ `SupersetSegmentView` - Superset display
- ✅ `CircuitSegmentView` - Circuit training
- ✅ `SchemeSegmentView` - Rep schemes
- ✅ `ComplexSegmentView` - Complex sequences
- ✅ `CommentSegmentView` - Text comments
- ✅ `ChooseSegmentView` - Choose/rotate options

### Dialogs
- ✅ `SegmentEditorView.swift` - Segment editing (partial)
  - Type picker for all 10 segment types
  - Complete form for straight sets
  - Complete form for comments
  - 🚧 Remaining 8 segment type editors needed
- ✅ `ValidationView.swift` - Error display
  - Success/error state
  - Error list with severity icons
  - Path display
- ✅ `GroupsEditorView.swift` - Exercise groups
  - List view with add/delete
  - Basic editing

### Features
- ✅ Document-based architecture (native Open/Save)
- ✅ Selection management (single, multi, range)
- ✅ Context menus (right-click)
- ✅ Keyboard shortcuts (Cmd+N/O/S/V)
- ✅ Validation integration
- ✅ Native macOS look and feel

---

## 🚧 Phase 4: Remaining Work (IN PROGRESS)

### High Priority

#### 1. Complete Segment Editors (8 remaining)
**Status**: 20% complete (2/10 done)
**Files**: `Sources/Dialogs/SegmentEditorView.swift`

Need to implement full editing forms for:
- [ ] RPE segments (sets, reps, RPE, rest, tempo)
- [ ] Percentage segments (sets, reps, %, of, rest, tempo)
- [ ] AMRAP segments (duration, min reps, weight)
- [ ] Superset segments (exercise list, rounds, rest)
- [ ] Circuit segments (exercise list with duration, rounds)
- [ ] Scheme segments (exercise, scheme string, rest)
- [ ] Complex segments (exercise sequence, sets, rest)
- [ ] Choose segments (nested segment options)

**Complexity**: Medium - Requires forms with dynamic fields

#### 2. Exercise Search & Database Integration
**Status**: 0% complete
**Files**: Need `Sources/Services/ExerciseDatabase.swift`

- [ ] SQLite connection from Swift
- [ ] Exercise search by name/code/body part
- [ ] Integration with rusqlite database
- [ ] Real-time search in right panel
- [ ] Exercise selection in segment dialogs

**Complexity**: Medium - Swift SQLite bindings needed

#### 3. Segment Operations (Delete, Duplicate, Reorder)
**Status**: 0% complete
**Files**: Extend `MainWindowView.swift`, `CanvasView.swift`

- [ ] Delete selected segments
- [ ] Duplicate segments
- [ ] Drag-and-drop reordering
- [ ] Move up/down buttons
- [ ] Bulk operations on multi-select

**Complexity**: Low - Straightforward operations

#### 4. Day Management
**Status**: 0% complete
**Files**: Need `Sources/Dialogs/DayEditorView.swift`

- [ ] Add day dialog
- [ ] Edit day label
- [ ] Delete day
- [ ] Reorder days

**Complexity**: Low - Simple CRUD operations

#### 5. Plan Metadata Editor
**Status**: 0% complete
**Files**: Need `Sources/Dialogs/PlanEditorView.swift`

- [ ] Edit plan name, author, description
- [ ] Version management
- [ ] Metadata fields

**Complexity**: Low - Simple form

### Medium Priority

#### 6. Autosave
**Status**: 10% complete (preferences ready)
**Files**: Need `Sources/Services/AutosaveService.swift`

- [ ] Timer-based autosave to drafts directory
- [ ] Conflict detection on open
- [ ] Recovery from drafts
- [ ] Integration with preferences

**Complexity**: Medium - Needs careful state management

#### 7. Undo/Redo
**Status**: 30% complete (stack in AppState)
**Files**: Extend `AppState.swift`

- [ ] Push plan state on changes
- [ ] Undo command (Cmd+Z)
- [ ] Redo command (Cmd+Shift+Z)
- [ ] Menu integration
- [ ] State diff optimization (currently stores full plan)

**Complexity**: Medium - Needs menu commands and efficient storage

#### 8. Recent Files
**Status**: 0% complete
**Files**: Need integration with NSDocumentController

- [ ] Native macOS recent files
- [ ] Clear recent files
- [ ] Menu integration

**Complexity**: Low - Use system APIs

### Lower Priority

#### 9. Additional Dialogs
- [ ] Media attachment management
- [ ] Sync/export workflow
- [ ] Advanced validation options

**Complexity**: Varies by dialog

#### 10. Polish & UX
- [ ] App icon
- [ ] Dark mode refinement
- [ ] Keyboard navigation (Tab, arrows, Enter)
- [ ] Accessibility support
- [ ] Tooltips and help text
- [ ] Error messages
- [ ] Loading indicators

**Complexity**: Low to medium - Incremental improvements

---

## 📋 Testing & Verification Needed

### Build Testing
- [ ] Test Rust FFI library build on Intel Mac
- [ ] Test on Apple Silicon
- [ ] Create universal binary
- [ ] Verify dylib loading
- [ ] Test Swift package build

### Functional Testing
- [ ] Open/save plans (compatibility with GTK version)
- [ ] All segment type display
- [ ] Basic editing workflow
- [ ] Validation
- [ ] File format compatibility
- [ ] Platform paths (verify ~/Library/* usage)

### Integration Testing
- [ ] GTK ↔ macOS file exchange
- [ ] Schema validation v0.3
- [ ] Exercise database access
- [ ] Error handling
- [ ] Memory leaks (Instruments)

---

## Known Issues & Limitations

### Current Limitations
1. **Segment editors incomplete** - Only straight sets and comments work
2. **No exercise search** - Right panel shows empty list
3. **No drag-and-drop** - Prepared but not wired up
4. **Single-level undo** - Stack exists but not used
5. **No autosave** - Preferences exist but service not implemented
6. **Stub implementations** - Some dialogs are minimal

### Technical Debt
1. **Error handling** - Need user-facing alerts for FFI errors
2. **Performance** - Large plans not tested
3. **Memory management** - FFI memory cleanup needs verification
4. **Testing** - Zero unit/integration tests

---

## File Inventory

### Created Files (25 total)

#### Rust FFI (5 files)
```
crates/ffi/
├── Cargo.toml
├── build.rs
├── cbindgen.toml
└── src/
    └── lib.rs
```

#### Swift Application (20 files)
```
apps/editor-macos/
├── Package.swift
├── Info.plist
├── README.md
├── build_rust.sh
└── Sources/
    ├── WeightliftingEditorApp.swift
    ├── AppState.swift
    ├── Models/
    │   ├── Plan.swift
    │   └── WeightliftingDocument.swift
    ├── RustBridge/
    │   ├── RustBridge.swift
    │   └── BridgingHeader.h
    ├── Views/
    │   ├── MainWindowView.swift
    │   ├── CanvasView.swift
    │   └── RightPanelView.swift
    └── Dialogs/
        ├── SegmentEditorView.swift
        ├── ValidationView.swift
        └── GroupsEditorView.swift
```

### Modified Files (2 total)
```
Cargo.toml                    # Added ffi to workspace
crates/core/src/paths.rs      # Added macOS path support
```

---

## Next Steps (Recommended Order)

### Week 1: Core Editing
1. Complete all segment editors in `SegmentEditorView.swift`
2. Implement delete/duplicate segment operations
3. Add day management dialogs
4. Test complete editing workflow

### Week 2: Search & Discovery
1. Integrate SQLite for exercise search
2. Populate exercise list in right panel
3. Add exercise picker to segment editors
4. Test exercise selection workflow

### Week 3: Polish & Features
1. Implement autosave service
2. Wire up undo/redo to menu commands
3. Add plan metadata editor
4. Implement segment reordering

### Week 4: Testing & Deployment
1. Comprehensive functional testing
2. GTK compatibility verification
3. Build universal binary
4. Create app bundle and icon
5. Write user documentation

---

## Building & Running

### Prerequisites
```bash
# Install Rust (if not already)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Verify installations
rustc --version
swift --version
xcodebuild -version
```

### Build Steps
```bash
# 1. Build Rust FFI library
cd /Users/jmahmood/WEIGHTTRAINING-EDITOR
cargo build --release -p weightlifting-ffi

# 2. Build Swift application
cd apps/editor-macos
swift build

# 3. Or open in Xcode
open Package.swift
```

### Troubleshooting
See `apps/editor-macos/README.md` for detailed troubleshooting guide.

---

## Success Criteria

### MVP (Minimum Viable Product)
- [x] File open/save works
- [x] All segment types display correctly
- [ ] All segment types editable (8/10 remaining)
- [ ] Exercise search works
- [ ] Basic editing workflow complete
- [x] Validation works
- [ ] Compatible with GTK version

### Feature Parity
- [ ] All GTK features implemented
- [ ] Keyboard shortcuts match (Ctrl→Cmd)
- [ ] Exercise groups fully functional
- [ ] Autosave working
- [ ] Recent files integration
- [ ] All dialogs complete

### Polish
- [ ] Native macOS look and feel
- [ ] App icon and branding
- [ ] Dark mode support
- [ ] Keyboard navigation
- [ ] Comprehensive documentation
- [ ] User testing complete

---

## Resources

- **Rust FFI Guide**: `crates/ffi/README.md` (create this)
- **Swift API Docs**: `apps/editor-macos/README.md`
- **GTK Reference**: `apps/editor-gtk/` (original implementation)
- **Schema Spec**: JSON schema v0.3 in `crates/validate/`

## Questions?

Contact the project maintainer or file an issue at the GitHub repository.
