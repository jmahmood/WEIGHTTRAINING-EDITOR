# Weightlifting Desktop

Tooling for authoring, validating, exporting, and reviewing weight training plans on Linux and macOS. The workspace bundles a GTK4/libadwaita desktop editor, a CLI companion, a metrics indexer, reusable core models, and file-based sync utilities built entirely in Rust.

The project targets modern Linux and macOS desktops (macOS implementation is currently ahead; feature parity planned for v1.5.0). Directory conventions follow XDG on Linux and standard macOS paths on Mac.

## Highlights
- Native desktop editors: GTK4/libadwaita for Linux, SwiftUI universal app for macOS (runs natively on both Apple Silicon and Intel)—both with autosave, validation gates, and rich segment manipulation.
- Command-line interface (`comp`) that validates, persists, exports, and visualises plans through built-in chart specs.
- Metrics indexer that ingests session CSV logs and emits cached E1RM, volume, and PR series for charting.
- Shared `weightlifting-core` library that owns the v0.3 plan schema, export staging, chart builders, attachments, and path management.
- Hardened JSON schema validator with semantic guards supplied by `weightlifting-validate`.
- Scriptable inbox/outbox sync workflow (SSH or USB-FS) for shuttling plans to a device and pulling results back.

## Repository Layout
- `apps/editor-gtk/` – GTK4 + libadwaita plan editor (`weightlifting-editor` binary) for Linux.
- `apps/editor-macos/` – SwiftUI native plan editor (`WeightliftingEditor.app`) for macOS.
- `crates/core/` – shared models (`Plan`, `Segment`, etc.), versioning, export staging, chart builders, attachment logs, and platform path helpers.
- `crates/ffi/` – C-compatible FFI interface for macOS SwiftUI editor integration.
- `crates/validate/` – JSON schema + semantic validation service.
- `crates/cli/` – CLI tool (`comp`) for plan lifecycle management and chart generation.
- `crates/indexer/` – CSV parser, metric calculators, cache persistence, and indexing CLI.
- `scripts/sync.sh` – transport-agnostic inbox/outbox synchronisation helper.
- `tests/` – integration and performance tests (e.g. 500-item validation benchmark).
- `*.json` in the repo root – example plans, sprint fixtures, and minimal templates.

## Plan Data Model
- We are currently migrating from the v0.3 (`PLAN_SPEC.md`) to v0.4 Plan JSON spec (`PLAN_04.md`).  The Linux editor currently only handles the 0.3 spec, while the MacOS version handles v0.4. Our companion application for the Apple Watch, **Red Star Weightlifting**, can handle a subset of both the v0.3 and v0.4 spec.

## Desktop Editor (`weightlifting-editor`)
- Built with GTK4/libadwaita; designed for wide displays with a canvas, inspector, navigator, and preview bar.
- Autosaves every 5 seconds into XDG state drafts, tracks unsaved changes, and persists MRU/last directory in `preferences.json`.
- Validation dialog (`PlanValidator`) runs before user-initiated save/export to block schema or semantic errors.
- Keyboard shortcuts (Ctrl+N/O/S, range selection with Shift, multi-select with Ctrl) and focused navigation for day/segment editing.
- Integrates with RecentManager so plans appear in the desktop shell’s recent documents.
- Sync dialog leverages `scripts/sync.sh` to send the active plan to a remote inbox over SSH or USB.
- Rebuild with `cargo run --release -p weightlifting-editor-gtk` or install the provided `.desktop`/PNG assets.

## macOS Editor (`WeightliftingEditor`)
- Built with SwiftUI; native universal macOS application targeting macOS 13.0+ (Ventura and later).
- Runs natively on both Apple Silicon (arm64) and Intel (x86_64) Macs—no Rosetta translation required.
- Three-column layout with sidebar (exercise list), canvas (plan view), and inspector panel with adjustable widths.
- Leverages native macOS `FileDocument` API for file I/O, recent documents tracking via `NSDocumentController`.
- Autosave with configurable interval stored in `UserDefaults`; unsaved changes tracked per document.
- Validation sheet runs schema and semantic checks via the shared Rust validation engine before save/export.
- Keyboard shortcuts follow macOS conventions (Cmd+N/O/S/Z for new/open/save/undo).
- Modal sheet dialogs for segment editing, group management, and plan properties; inline editing in the inspector for quick tweaks.
- Multi-select and range-select support for batch segment operations.
- Communicates with Rust core via FFI bridge (`RustBridge.swift`) using JSON string interchange—all business logic remains in Rust.
- Build with `swift build` or `xcodebuild`; create `.app` bundle via provided packaging scripts.

### macOS Storage Paths
`weightlifting-core::AppPaths` resolves to macOS-standard directories:
- `~/Library/Application Support/weightlifting-desktop/` – published plans (`plans/<plan_id>/<version>.json`) and media attachments.
- `~/Library/Application Support/weightlifting-desktop/` – draft copies (`drafts/<plan_id>.json`) and autosave state.
- `~/Library/Caches/weightlifting-desktop/` – metrics cache (`metrics/*.json`) for charting and indexer output.

The editor creates these directories on demand. User preferences are stored in `UserDefaults`.

### Runtime Storage
`weightlifting-core::AppPaths` resolves to:
- **Linux:** `~/.local/share/weightlifting-desktop/` – published plans (`plans/<plan_id>/<version>.json`) and media attachments.
- **Linux:** `~/.local/state/weightlifting-desktop/` – draft copies (`drafts/<plan_id>.json`) and autosave state.
- **Linux:** `~/.cache/weightlifting-desktop/` – metrics cache (`metrics/*.json`) for charting and indexer output.

Ensure these directories exist automatically; the editor and CLI create them on demand.

## CLI Companion (`comp`)

```
cargo run -p weightlifting-cli -- <command> <subcommand> [flags]
```

### Plan Commands
- `plans validate [--in | --file path]` – schema + semantic validation. Prints JSON diagnostics to stdout and human summary to stderr. Exits non-zero when errors are present.
- `plans save [--in | --file path]` – persists JSON plans to the drafts directory, generating an ID from the plan name.
- `plans get --id PLAN_ID [--version 1.2.3]` – emits the stored plan JSON (defaults to the latest draft).
- `plans export --id PLAN_ID [--version 1.2.3] --mount /mnt/device [--dry-run]` – stages an export manifest, detects conflicts (duplicate IDs, exercise mismatches), and writes files into `<mount>/plans/<plan_id>/`.

### Chart Commands
The chart subcommands expect cached metrics generated by the indexer.

- `chart emit-spec --chart-type {e1rm|volume|pr|heatmap} [--exercise CODE] [--start-date YYYY-MM-DD] [--end-date YYYY-MM-DD] [--output spec.json]` – produces Vega-Lite JSON specifications via `BuiltinCharts`.
- `chart render ...` – placeholder for future headless rendering (currently exits with explainers).
- `chart export-csv --chart-type {e1rm|volume|pr} [filters] --output metrics.csv` – dumps CSV summaries (kg + lb columns).

Exit codes:
- `0` success, `2` recoverable plan/chart errors, `3` export conflicts, `4` missing paths.

## Metrics Indexer

```
cargo run -p weightlifting-indexer -- <command>
```

- `process --input path/to/sessions.csv [--force]` – parses session logs (v0.3 format), computes E1RM/volume/PR series, and caches them.
- `status` – reports entry counts, last update timestamp, and cache size.
- `clear` – removes cached metrics files.

The CSV parser enforces XOR reps/time, effort bounds, unit validation (`kg|lb|bw`), and other integrity checks. Cached JSON lives under `~/.cache/weightlifting-desktop/metrics/` and feeds the CLI chart commands and editor widgets.

## Sync Workflow
- `scripts/sync.sh` implements an inbox/outbox protocol with `.part` staging, SHA-256 verification, and archives. Transports: `ssh`, `usb-fs`, or automatic selection.
- Common flags: `--transport ssh`, `--local-root ~/.local/share/weightlifting-desktop`, `--remote-host user@host`, `--remote-root /srv/plans`, `--usb-mount /run/media/...`, `--dry-run 1`.
- Subcommands: `send`, `receive`, `setup-remote`, `discover-remote-dirs`, `auto`.
- See `docs/inbox-outbox.md` for directory semantics and expected behaviour on both sides.
- The GTK editor shells out to this script; override with `WEIGHTLIFTING_SYNC_SCRIPT` if needed.

## Getting Started

### Prerequisites

#### Linux
- Rust toolchain ≥ 1.75 (`rustup` recommended).
- System packages for GTK4 + libadwaita development, e.g. on Debian/Ubuntu:
  ```
  sudo apt install build-essential pkg-config libgtk-4-dev libadwaita-1-dev libglib2.0-dev
  ```
- Optional: `sha256sum`/`openssl`, `rsync`, `ssh` for sync flows; `sqlite3`/`rusqlite` dependencies are vendored with the `bundled` feature.

#### macOS
- Rust toolchain ≥ 1.75 (`rustup` recommended).
- Xcode Command Line Tools or Xcode 15+ for Swift 5.9+ compiler.
- macOS 13.0+ (Ventura or later) for running the editor.
- Optional: `openssl`, `rsync`, `ssh` for sync flows (typically pre-installed).

##### macOS Universal Binary Support
The macOS editor is built as a universal binary and runs natively on both Apple Silicon and Intel Macs. To build from source:
- Install x86_64 Rust target: `rustup target add x86_64-apple-darwin`
- The build process automatically creates universal binaries for both the Rust FFI library and Swift executable

### Build & Run

#### Linux
- Build workspace: `cargo build --workspace --release`
- Launch GTK editor: `cargo run --release -p weightlifting-editor-gtk`

#### macOS

Prerequisites:
- Install x86_64 Rust target: `rustup target add x86_64-apple-darwin`

Build and run:
- Build workspace: `cargo build --workspace --release`
- Quick dev: `swift build && swift run` (from `apps/editor-macos/`, native arch only)
- Create universal app bundle: `cd apps/editor-macos && ./create-app-bundle.sh`
- Verify universal binary: `cd apps/editor-macos && ./verify-universal.sh`
- App bundle at: `apps/editor-macos/build/WeightliftingEditor.app`

**Note:** The app bundle created by `create-app-bundle.sh` is always a universal binary (arm64 + x86_64) that runs natively on both Apple Silicon and Intel Macs without requiring Rosetta 2.

**Code Signing & Notarization (Optional):**
By default, the app bundle is created without code signing. To distribute the app to other Macs, you'll need to sign and notarize it:

```bash
# Option 1: Set environment variables directly
export CODESIGN_IDENTITY="Developer ID Application: Your Name (TEAMID)"
export NOTARY_KEYCHAIN_PROFILE="YourProfileName"
cd apps/editor-macos && ./create-app-bundle.sh

# Option 2: Use the example config file (recommended)
cd apps/editor-macos
cp .env.signing.example .env.signing
# Edit .env.signing with your credentials
source .env.signing && ./create-app-bundle.sh
```

To set up notarization credentials:
```bash
# Store your Apple ID credentials in keychain (one-time setup)
xcrun notarytool store-credentials "YourProfileName" \
  --apple-id "your-apple-id@example.com" \
  --team-id "TEAMID" \
  --password "app-specific-password"
```

For local development and testing, signing is not required.

#### CLI & Tools (Cross-platform)
- Validate a plan: `cargo run -p weightlifting-cli -- plans validate --file test_plan.json`
- Export to draft: `cat test_plan.json | cargo run -p weightlifting-cli -- plans save --in`
- Generate chart spec: `cargo run -p weightlifting-cli -- chart emit-spec --chart-type e1rm --output e1rm.json`
- Index session data: `cargo run -p weightlifting-indexer -- process --input logs/sessions.csv`

The repo ships with `test_plan.json`, `test_sprint1.json`, and other fixtures for experimentation.

## Development Notes
- Rust edition 2021 with workspace-level dependencies pinned in root `Cargo.toml`.
- Follow `rustfmt` defaults. Run `cargo fmt --all` and `cargo clippy --all-targets --all-features` before submitting patches.
- Tests: `cargo test --workspace` (includes validator performance test under `tests/`).
- **GTK UI code** (Linux) lives under `apps/editor-gtk/src/`; canvases and widgets are decomposed into modules (`ui/components`, `canvas`, `operations`, `state`).
- **SwiftUI code** (macOS) lives under `apps/editor-macos/Sources/`; organised into `Views/`, `Dialogs/`, `Models/`, and `RustBridge/`. Follow Swift API design guidelines.
- **FFI layer** in `crates/ffi/` provides C-compatible interface; all business logic stays in Rust core—SwiftUI and GTK frontends are thin presentation layers.
- Validation rules are centralised in `crates/validate/src/validator.rs`; extend semantic checks there alongside schema updates.
- Export manifests (`ExportStager`) and metrics calculators have unit tests in their respective crates—add coverage when modifying behaviour.
- **Code signing** (macOS): The build script supports optional code signing via environment variables. For local development, unsigned builds work fine. For distribution, set `CODESIGN_IDENTITY` and `NOTARY_KEYCHAIN_PROFILE` before building.

## Troubleshooting

### Universal Binary Build Issues

**Missing Rust target:**
```bash
# Install x86_64 target
rustup target add x86_64-apple-darwin

# Verify installation
rustup target list --installed
# Should show both: aarch64-apple-darwin, x86_64-apple-darwin
```

**Verify app is universal:**
```bash
# Check Swift executable architectures
lipo -info apps/editor-macos/build/WeightliftingEditor.app/Contents/MacOS/WeightliftingEditor
# Expected output: Architectures in the fat file: ... are: x86_64 arm64

# Check Rust FFI library architectures
lipo -info apps/editor-macos/build/WeightliftingEditor.app/Contents/Frameworks/libweightlifting_ffi.dylib
# Expected output: Architectures in the fat file: ... are: x86_64 arm64

# Or use the verification script
cd apps/editor-macos && ./verify-universal.sh
```

**Library loading errors:**
If the app crashes with "Library not loaded" errors, the dylib install name or rpath may be incorrect. Rebuild from scratch:
```bash
rm -rf apps/editor-macos/build apps/editor-macos/.build target/release/libweightlifting_ffi.dylib
cd apps/editor-macos && ./create-app-bundle.sh
```

**Code signing issues:**
If you see "app is damaged" or code signature errors:
- For local use: Unsigned builds work fine, just right-click → Open the first time
- For distribution: Ensure your Developer ID certificate is valid and not expired
- Check signing: `codesign -dv --verbose=4 build/WeightliftingEditor.app`
- Remove quarantine attribute: `xattr -d com.apple.quarantine build/WeightliftingEditor.app` (local testing only)

## Additional Resources
- `weightlifting-editor.desktop` & `weightlifting.png` – GTK launchers and branding assets for Linux desktop integration.
- `apps/editor-macos/AppIcon.icns` – macOS application icon for `.app` bundle.

---

## Using Plans

The plans generated are meant to allow you to run them on a handheld application for when you are in the gym.  It goes without saying, but we need to 

I have a application that runs with v0.3 plans on ROCKNIX (https://github.com/jmahmood/basic_weightlifting) but it is annoying to install, and it's annoying to carry another device if you don't have to.  Moreover, it only works on a subset of devices (the Anbernic family; RG28XX, RG35XXSP, RG34XXSP, RG35XX Plus).  In retrospect, I should have targetted the Raspberry Pi.

but the integration w/ the iOS and WatchOS apps are much easier with the MacOS version.  I am exploring ways to simplify integration, but right now, 