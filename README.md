# RED ✪ STAR Plan Studio

*“A desktop PLAN editor, validator, and exporter for Red Star Weightlifting.*

---

## Screenshots

### iOS
<p>
  <img src="./screenshots/Main Menu.png" alt="Main Menu" width="200">
  <img src="./screenshots/Add New Exercise.png" alt="Add New Exercise to Segment" width="200">
  <img src="./screenshots/Add Workout Segment 2.png" alt="Add New Segment" width="200">
  <img src="./screenshots/Modify Exercise Groups.png" alt="Add subtitute exercises" width="200">
  <img src="./screenshots/Alternative Resistance Types.png" alt="Add alternative types of resistance (bands, etc)" width="200">
</p>

<details>
<summary><b>Linux</b></summary>
<br>
<p>(Coming soon)</p>
</details>

---


**Plan Studio** is a native desktop editor for authoring and validating weight training plans used by **RED ✪ STAR Weightlifting**. It is designed for offline-first plan creation, deterministic exports, and long-term durability of strength programming data.

The application targets self-coached lifters who want precise control over programming without relying on cloud services or mobile devices during training.

---

## Installation

### macOS

Download the zipped app file from [here](https://github.com/jmahmood/WEIGHTTRAINING-EDITOR/releases/download/v1.1.1-mac/WeightliftingEditor.zip).  Unzip and drag to your Application Folder

### Linux

Please see build description below.

---

## What This Application Does

- Author structured weight training plans using the Red Star PLAN JSON format (v0.3 and v0.4).
- Validate plans against both schema and semantic rules before use.
- Export plans in a device-ready layout for handheld or watch-based execution.

The editor is intentionally **offline-first**. Network access is optional and only used for explicit file synchronization.

---

## How It Fits Into RED ✪ STAR Weightlifting

Plans authored here are executed elsewhere.

Typical workflow:

1. **Author** a plan on desktop (this project).
2. **Export** the plan to a handheld or watch device.
3. **Execute** the plan in the gym without a phone or internet.
4. **Sync back** completed session logs if you would like to analyze.

Supported execution targets:

- **RED ✪ STAR Weightlifting (watchOS + iOS bridge)**  
  Apple Watch–based execution. Supports a subset of PLAN v0.3 and v0.4.

- **Basic Weightlifting (ROCKNIX / Anbernic devices)**  
  Legacy handheld executor targeting PLAN v0.3. Installation friction is high and device support is limited.

The desktop editor is the **authoritative source of truth** for plans across all targets.

---

## Platform Support

- **macOS**: Feature-complete reference implementation (PLAN v0.4).
- **Linux**: GTK4/libadwaita editor (PLAN v0.3; v0.4 parity planned).

All core logic (validation, export, metrics) is shared across platforms and implemented in Rust.

---

## Key Components

### Desktop Editors
- **GTK4 / libadwaita editor (Linux)**  
  Wide-screen layout with navigator, canvas, inspector, autosave drafts, and validation gating.

- **SwiftUI editor (macOS)**  
  Native universal app (Apple Silicon + Intel) using Rust via FFI for all business logic.

### CLI Companion (`comp`)
A command-line tool for plan lifecycle management:
- Validate, save, retrieve, and export plans.
- Generate chart specifications (Vega-Lite).
- Inspect errors and conflicts deterministically.

### Metrics Indexer
Processes session CSV logs and produces cached metrics:
- Estimated 1RM
- Volume
- PR series

These metrics feed both CLI charts and editor visualizations.


---

## PLAN Format

This project implements the **Red Star PLAN JSON specification**.

- v0.3: Stable, widely supported.
- v0.4: Adds week-dependent overlays, group variants, and non-weight load axes.

**Every v0.3 plan is valid v0.4.**  
Detailed specification lives in `PLAN_04.md`.

---

## Repository Layout (High-Level)

- `apps/editor-gtk/` – Linux desktop editor
- `apps/editor-macos/` – macOS SwiftUI editor
- `crates/core/` – Shared plan models, export logic, charts, paths
- `crates/validate/` – Schema + semantic validation
- `crates/cli/` – CLI companion (`comp`)
- `crates/indexer/` – Metrics ingestion and caching
- `scripts/` – File-based sync utilities

---

## Storage Model

The application uses platform-standard directories and creates them on demand.

- **Plans**: versioned, append-only
- **Drafts**: autosaved working copies
- **Metrics**: cached, reproducible outputs

No opaque databases, no cloud state.

---

## Build & Run (Summary)

### Linux
```bash
cargo run --release -p weightlifting-editor-gtk
cargo run --bin weightlifting-editor
````

### macOS

```bash
cd apps/editor-macos
./create-app-bundle.sh
```

Unsigned builds are sufficient for local use. Code signing is optional and documented.

---

## Status

Active development. macOS is the reference platform for now, but we intend for Linux parity in the near future.

Any breaking PLAN changes are gated behind explicit version bumps and migration notes.

---

## License

See repository license files.

---

## Signing binary for Apple devices

**Code Signing & Notarization (Optional):**
By default, the app bundle is created without code signing. To distribute the app to other Macs, you'll need to sign and notarize it.

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



