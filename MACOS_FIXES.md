# macOS Conversion - Critical Fixes Applied

## Issue Summary

The initial FFI layer had compilation errors because I made incorrect assumptions about your data model. After examining the actual `crates/core/src/models.rs`, I found significant differences from my assumptions.

## Data Model Differences

### What I Assumed vs. Reality

| Component | Assumed | Actual |
|-----------|---------|--------|
| Days field | `plan.days: Vec<Day>` | `plan.schedule: Vec<Day>` |
| Exercise groups | `plan.exercise_groups: Vec<ExerciseGroup>` | `plan.groups: HashMap<String, Vec<String>>` |
| Exercises | `Exercise` objects with fields | Just strings (`ex: String`) |
| Validation | `validate_plan(&plan)` function | `PlanValidator::new()?.validate(&plan)` |

### Key Rust Structures (Actual)

```rust
pub struct Plan {
    pub name: String,
    pub author: Option<String>,
    pub source_url: Option<String>,
    pub license_note: Option<String>,
    pub unit: Unit,                                    // kg, lb, bw
    pub dictionary: HashMap<String, String>,           // Exercise codes to names
    pub groups: HashMap<String, Vec<String>>,          // Substitution groups
    pub exercise_meta: Option<HashMap<String, ExerciseMeta>>,
    pub phase: Option<Phase>,
    pub week_overrides: Option<HashMap<String, Vec<WeekOverride>>>,
    pub equipment_policy: Option<EquipmentPolicy>,
    pub progression: Option<Progression>,
    pub warmup: Option<WarmupConfig>,
    pub schedule: Vec<Day>,                            // The workout days!
}

pub struct Day {
    pub day: u32,                     // Day number (e.g., 1, 2, 3)
    pub label: String,                // Day name (e.g., "Upper A")
    pub time_cap_min: Option<u32>,
    pub goal: Option<String>,
    pub equipment_policy: Option<EquipmentPolicy>,
    pub segments: Vec<Segment>,
}

// Segments use the "ex" field for exercise codes (strings)
pub struct StraightSegment {
    #[serde(flatten)]
    pub base: BaseSegment,            // Contains ex: String, alt_group, label, etc.
    pub sets: Option<u32>,
    pub sets_range: Option<Range>,
    pub reps: Option<RepsOrRange>,
    // ... many optional fields
}
```

## Fixes Applied to FFI Layer

### 1. Fixed Imports
```rust
// Before
use weightlifting_core::models::{Plan, Segment, Day, Exercise, ExerciseGroup};
use weightlifting_validate::validate_plan;

// After
use weightlifting_core::models::{Plan, Segment, Day};
use weightlifting_validate::PlanValidator;
```

### 2. Fixed Plan Creation
```rust
// Before
let plan = Plan::default();

// After
let plan = Plan::new("New Plan".to_string());
```

### 3. Fixed Validation
```rust
// Before
let errors = validate_plan(&plan);

// After
let validator = PlanValidator::new()?;
let result = validator.validate(&plan);
```

### 4. Fixed All Field References
- Changed `plan.days` → `plan.schedule` (6 occurrences)
- Changed `plan.exercise_groups` → `plan.groups` (3 occurrences)

### 5. Updated Exercise Groups API
Since groups are `HashMap<String, Vec<String>>`, updated functions:

```rust
// New signature
ffi_group_add(
    plan_json: *const c_char,
    group_name: *const c_char,      // Group name as string
    exercises_json: *const c_char,  // JSON array ["ex1", "ex2"]
) -> FFIResult

// Added new function
ffi_group_remove(plan_json, group_name) -> FFIResult
```

## What Still Needs Fixing

### ⚠️ CRITICAL: Swift Models Need Complete Rewrite

The Swift models in `apps/editor-macos/Sources/Models/Plan.swift` are **completely wrong** and need to be rewritten to match the actual Rust structure:

#### Required Changes:

1. **Plan Model**
   ```swift
   // Current (WRONG)
   struct Plan {
       var days: [Day]
       var exerciseGroups: [ExerciseGroup]
   }

   // Should be
   struct Plan {
       var name: String
       var author: String?
       var sourceUrl: String?
       var licenseNote: String?
       var unit: Unit                      // enum: kg, lb, bw
       var dictionary: [String: String]    // exercise codes to names
       var groups: [String: [String]]      // substitution groups
       var exerciseMeta: [String: ExerciseMeta]?
       var phase: Phase?
       var weekOverrides: [String: [WeekOverride]]?
       var equipmentPolicy: EquipmentPolicy?
       var progression: Progression?
       var warmup: WarmupConfig?
       var schedule: [Day]                 // RENAMED from "days"
   }
   ```

2. **Day Model**
   ```swift
   // Add missing fields
   struct Day {
       var day: UInt32           // Day number
       var label: String
       var timeCapMin: UInt32?
       var goal: String?
       var equipmentPolicy: EquipmentPolicy?
       var segments: [Segment]
   }
   ```

3. **Segment Models**
   - Exercises are strings (`ex: String`), not objects
   - Each segment type has `BaseSegment` flattened into it
   - All fields are optional (makes it complex)

   Example:
   ```swift
   struct StraightSegment: Codable {
       // From BaseSegment (flattened)
       var ex: String
       var altGroup: String?
       var label: String?
       var optional: Bool?
       var technique: [String: String]?
       var equipmentPolicy: EquipmentPolicy?

       // Straight-specific fields
       var sets: UInt32?
       var setsRange: Range?
       var reps: RepsOrRange?
       var timeSec: TimeOrRange?
       var restSec: RestOrRange?
       var rir: Double?
       var rpe: Double?
       var tempo: Tempo?
       // ... many more
   }
   ```

4. **Add Missing Types**
   Need to add Swift models for:
   - `Unit` enum (kg, lb, bw)
   - `ExerciseMeta`
   - `Phase`
   - `WeekOverride`
   - `EquipmentPolicy`
   - `Progression`
   - `WarmupConfig`
   - `BaseSegment` (embedded in all executable segments)
   - `RepsOrRange` (can be int or range)
   - `TimeOrRange`, `RestOrRange`
   - `Tempo`, `Vbt`, `LoadMode`, `Intensifier`
   - `Range` struct

### Build Status

✅ **Rust FFI Layer**: Compiles successfully
❌ **Swift Application**: Will NOT compile until models are fixed
❌ **Integration**: Cannot test until Swift side matches Rust

## Recommendations

### Option 1: Minimal Swift Models (Recommended for MVP)

Since the data model is very complex, consider a simpler approach for the initial version:

1. **Keep plan as JSON in Swift**
   - Don't decode the full structure
   - Pass JSON strings directly to/from FFI
   - Only decode specific parts when displaying

2. **Use JSON paths for editing**
   - Edit individual segments by index
   - Don't try to model everything in Swift

Example:
```swift
class PlanDocument: ObservableObject {
    @Published var planJSON: String  // Keep as JSON

    var name: String {
        // Parse just the name field
        let data = Data(planJSON.utf8)
        let json = try? JSONSerialization.jsonObject(with: data) as? [String: Any]
        return json?["name"] as? String ?? "Untitled"
    }

    var days: [Day] {
        // Parse just the schedule
        // ...
    }
}
```

### Option 2: Complete Swift Models (For Full Native Experience)

If you want a fully native SwiftUI experience with proper model binding:

1. Create complete Swift models matching **every** field in Rust
2. Handle all the optional fields and unions
3. Implement comprehensive Codable conformance
4. Deal with complex types like `RepsOrRange` (can be int or struct)

**Estimated effort**:
- Option 1: 1-2 days
- Option 2: 4-5 days (very tedious)

### Option 3: Auto-Generate Swift Models

Use a code generation tool:

1. Export Rust types as TypeScript definitions (using `ts-rs`)
2. Convert TypeScript to Swift (manual or with a tool)
3. Or use `serde-generate` to create Swift Codable structs

This would be the most robust long-term solution.

## Current Status

### What Works
- ✅ Rust FFI compiles
- ✅ C headers generated (`crates/ffi/include/weightlifting_ffi.h`)
- ✅ All FFI functions implemented
- ✅ Plan operations (new, open, save, validate)
- ✅ Segment operations (add, remove, update)
- ✅ Day operations (add, remove)
- ✅ Group operations (get, add, remove)
- ✅ Platform path queries

### What's Broken
- ❌ Swift models don't match Rust
- ❌ Swift app won't compile
- ❌ Can't test integration yet

## Next Steps

1. **Decide on approach** (Option 1, 2, or 3 above)
2. **Rewrite Swift models** or implement JSON-based approach
3. **Test with actual plan files** from GTK version
4. **Verify serialization** roundtrip (Rust → Swift → Rust)
5. **Update UI** to work with real data structure

## Files Modified

### Rust (2 files)
- `crates/ffi/src/lib.rs` - Fixed all compilation errors
- `crates/core/src/paths.rs` - Already updated (no changes needed)

### Swift (NEEDS WORK)
- `apps/editor-macos/Sources/Models/Plan.swift` - **MUST BE REWRITTEN**
- `apps/editor-macos/Sources/RustBridge/RustBridge.swift` - May need adjustments
- All views that reference models - Will need updates

## Testing Strategy

Once models are fixed:

```bash
# 1. Build FFI
cargo build --release -p weightlifting-ffi

# 2. Test with a real plan file
cargo run -p weightlifting-cli -- plans validate --file path/to/plan.json

# 3. Test FFI directly
# Create a simple C test program that calls FFI functions

# 4. Build Swift app
cd apps/editor-macos
swift build

# 5. Test roundtrip
# Open plan in GTK → Save → Open in macOS → Save → Open in GTK
```

## Questions?

The core decision is: **How much of the Rust data model do you want to represent in Swift?**

- **Minimal approach**: Faster to implement, less type-safe
- **Complete approach**: Full type safety, more work, better UX

Let me know which direction you'd like to go, and I can help implement it.
