# Plan JSON v0.3 (Lift Logbook) — Definitive Spec (Weight Training Only)

This is the **authoritative** schema for training plans. It supersedes v0.2. Backwards compatibility is **not required**. All new plans must use this spec.

> **Scope:** Resistance / weight training only. Cardio/conditioning protocols are **out of scope** for v0.3. If a session requires a non-lifting action (e.g., “go to track for cardio”), insert a `comment` segment between lifting segments.

## Design Goals

* Encode real programming patterns cleanly (top set + back‑off, drop sets, rep/rest ranges, **anchored prescriptions**, **complexes**, **RIR**, **VBT**, **cluster** details, **choose/rotate**, **optional**, **technique & equipment constraints**, **test days**).
* Keep substitutions separate from progression: **`alt_group` is for replacement only**.
* Make UI/CSV logging lossless: everything needed to render, time, and tag a set is represented.
* Use **Segment** to describe each executable part of a day (formerly called “block”).

---

## 1. Top‑Level Plan Object

```json
{
  "name": "Essentials — Segment Plan A",
  "author": "Program Author",
  "source_url": "https://example.com/plan",
  "license_note": "For personal use; see source.",
  "unit": "kg",                                   // "kg" | "lb" | "bw"
  "dictionary": { "BP.DB.FLAT": "Flat Dumbbell Press" },
  "groups": { "GROUP_CHEST_PRESS": ["BP.DB.FLAT", "BP.MACH.CHEST", "DIP.WT.STND"] },
  "exercise_meta": {                                // optional UI hints
    "LEG.PRESS.45": { "equipment": ["machine"], "home_friendly": false },
    "SQUAT.DB.GOBLET": { "equipment": ["dumbbell"], "home_friendly": true }
  },
  "phase": { "index": 1, "weeks": [1,2,3,4] },  // optional phase metadata
  "week_overrides": {                               // optional deload/tweak hooks
    "4": [ { "target": { "day": 2, "segment_idx": 0 }, "modifier": { "rpe_cap": 8.0 } } ]
  },
  "equipment_policy": {                              // optional plan-wide constraints
    "allowed": ["barbell", "dumbbell", "bands"],
    "forbidden": ["machines"]
  },
  "progression": {                                   // optional plan-level hints
    "mode": "double_progression",
    "reps_first": true,
    "load_increment_kg": 2.5,
    "cap_rpe": 9.0
  },
  "warmup": {                                        // optional warm-up generator hints
    "pattern": "percent_of_top",
    "stages": [ {"pct_1rm":0.30, "reps":8}, {"pct_1rm":0.50, "reps":5}, {"pct_1rm":0.70, "reps":3} ],
    "round_to": 2.5,
    "merge_after_rounding": true
  },
  "schedule": [ /* days... */ ]
}
```

**Notes**

* `unit` maps to the runtime `Unit` enum (`kg|lb|bw`).
* `dictionary` + `groups` define exercise codes and legal swaps. Every `alt_group` must reference a key in `groups` and all members must exist in `dictionary`.
* `exercise_meta` is non-functional metadata used by the UI for filtering/suggestions.

---

## 2. Day Object

```json
{
  "day": 1,
  "label": "Upper A",
  "time_cap_min": 45,                     // optional: total time cap for day
  "goal": "Push strength + chest volume",// optional: UI hint
  "equipment_policy": { "allowed": ["barbell","dumbbell"], "forbidden": ["machines"] },
  "segments": [ /* list of segment objects (oneOf below) */ ]
}
```

---

## 3. Common Data Types

### 3.1 Reps

Single representation across the spec. Fixed reps are expressed with identical `min/max`.

```json
{ "min": 8, "max": 12, "target": 12 } // target optional
```

### 3.2 Rest

`rest_sec` accepts either a number or a range object:

```json
90
// or
{ "min": 90, "max": 180 }
```

### 3.3 Effort Target: RPE / RIR

At least one of `rpe` or `rir` may be provided where effort is needed.

```json
"rpe": 8.5
// or
"rir": 2
// or ranges (for `scheme` sets):
"rpe": { "min": 8.0, "max": 9.0 }
```

> If both are present, **RPE takes precedence** for load guidance; UI may display both.

### 3.4 Intensifier

Machine-readable set modifiers.

```json
{
  "kind": "dropset",                 // "dropset" | "rest_pause" | "myo" | "cluster"
  "when": "last_set",                // "each_set" | "last_set"
  "drop_pct": 0.15,                   // for dropsets (optional)
  "steps": 1,                         // number of drops (optional)
  "clusters": 3,                      // for cluster sets (optional)
  "reps_per_cluster": 2,              // for cluster sets (optional)
  "intra_rest_sec": 20                // for cluster sets (optional)
}
```

### 3.5 Tempo (structured)

```json
"tempo": { "ecc": 3, "bottom": 0, "con": 1, "top": 0, "units": "sec" }
```

### 3.6 VBT (Velocity‑Based Training)

```json
"vbt": { "target_mps": 0.35, "loss_cap_pct": 20 }
```

### 3.7 Load Mode (bodyweight variants)

```json
"load_mode": "added"   // "added" | "assisted" | "bodyweight_only"
```

### 3.8 Set Count Range & Auto‑Stop

```json
"sets_range": { "min": 3, "max": 5 },           // mutually exclusive with `sets`
"auto_stop": { "reason": "velocity_loss_pct", "threshold": 20 } // or "technique"
```

### 3.9 Anchors (percent of a prior set in this segment)

Use to express back‑off work as a % of a **top set performed earlier in the same segment**.

```json
"anchor": { "of_set_index": 0, "multiplier": 0.80 }  // 80% of set[0] actual load
```

### 3.10 Optionality & Rotation

* `optional`: boolean on **any segment**; UI treats as skippable and excludes from failure counters.
* `rotation`: string on `choose` segments; how choices rotate across sessions.

  * Allowed: `"weekly" | "session" | "random" | "none"` (default `"none"`).

### 3.11 Technique Modifiers

Declarative technique cues (assist printing and constraints).

```json
"technique": { "stance": "sumo", "grip": "clean", "rom": "2\" deficit" }
```

### 3.12 Equipment Policy (per segment)

```json
"equipment_policy": { "allowed": ["barbell"], "forbidden": ["machines"] }
```

### 3.13 Timed Set

Use `time_sec` as an alternative to `reps`. Accepts a number or a range object. Exactly one of `reps` **or** `time_sec` must be present for any executable set item.

```json
"time_sec": 45
// or
"time_sec": { "min": 30, "max": 60, "target": 45 }
```

**Semantics**

* A set with `time_sec` is a **hold/work timer**; UI shows a **countdown** instead of a rep counter.
* `rpe/rir` remain optional and (if present) refer to perceived effort **at the end of the interval**.
* `tempo` may be omitted for isometric holds; if supplied, it's advisory (e.g., "breathe: 4-4").

---

## 4. Segment Types (oneOf)

### 4.1 `straight`

"Classic" fixed scheme for volume work.

```json
{
  "type": "straight",
  "ex": "BP.DB.FLAT",
  "alt_group": "GROUP_CHEST_PRESS",   // replacement palette only
  "sets": 3,                           // or `sets_range`
  "reps": { "min": 10, "max": 12 },   // or `time_sec` (XOR)
  "rest_sec": 120,
  "rir": 2,                            // or `rpe`
  "tempo": { "ecc": 3, "bottom": 0, "con": 1, "top": 0, "units": "sec" },
  "vbt": { "target_mps": 0.45 },     // optional
  "load_mode": "added",               // optional (for dips/pull-ups)
  "label": "Back‑off volume",
  "optional": true,
  "technique": { "stance": "high_bar" },
  "equipment_policy": { "allowed": ["dumbbell"], "forbidden": [] },
  "intensifier": { "kind": "rest_pause", "steps": 1 }
}
```

**Timed example:**

```json
{
  "type": "straight",
  "ex": "CORE.BW.PLNK",
  "sets": 4,
  "time_sec": { "min": 30, "max": 60, "target": 45 },
  "rest_sec": 60
}
```

### 4.2 `rpe`

RPE/RIR‑targeted work; supports ranges and anchors.

```json
{
  "type": "rpe",
  "ex": "OHP.BB.STND",
  "sets": 3,
  "reps": { "min": 5, "max": 5 },     // or `time_sec` (XOR)
  "rpe": 8.0,
  "rest_sec": { "min": 120, "max": 180 },
  "anchor": { "of_set_index": 0, "multiplier": 0.85 }, // optional
  "label": "Top work"
}
```

**Timed example:**

```json
{
  "type": "rpe",
  "ex": "CORE.BW.SIDEPLANK",
  "sets": 3,
  "time_sec": 45,
  "rpe": 8.0,
  "rest_sec": 60
}
```

### 4.3 `percentage`

```json
{
  "type": "percentage",
  "ex": "SQ.BB.BACK",
  "prescriptions": [
    { "sets": 1, "reps": 5, "pct_1rm": 0.70 },
    { "sets": 1, "reps": 5, "pct_1rm": 0.75 },
    { "sets": 1, "reps": 5, "pct_1rm": 0.80 }
  ]
}
```

### 4.4 `amrap`

```json
{ "type": "amrap", "ex": "DL.BB.CONV", "base_reps": 8, "cap_reps": 15 }
```

### 4.5 `superset`

Structured pair with pairing semantics (e.g., **contrast/PAP**).

```json
{
  "type": "superset",
  "label": "A1/A2",
  "pairing": "contrast",               // optional: "standard" | "contrast"
  "rounds": 2,
  "rest_sec": 0,                        // between items
  "rest_between_rounds_sec": 120,       // between rounds
  "items": [
    { "ex": "SQ.BB.BACK", "sets": 2, "reps": {"min": 2, "max": 2}, "rpe": 8.5 },
    { "ex": "JUMPS.BW.SQ",  "sets": 2, "reps": {"min": 5, "max": 5} }
  ]
}
```

### 4.6 `circuit`

Like superset but 3+ items; **use only with resistance exercises**, not cardio.

```json
{
  "type": "circuit",
  "rounds": 3,
  "rest_sec": 90,
  "rest_between_rounds_sec": 90,
  "items": [
    { "ex": "CORE.BW.PLNK", "time_sec": 45 },
    { "ex": "CALF.MACH.SEAT", "reps": {"min": 15, "max": 20}, "alt_group": "GROUP_CALF" },
    { "ex": "LUNGE.DB.STND", "reps": {"min": 12, "max": 12} }
  ]
}
```

### 4.7 `scheme` (mixed set prescriptions)

Encodes patterns like **top set + back‑off** in one segment. Supports **anchors** to the top set.

```json
{
  "type": "scheme",
  "ex": "SQ.BB.BACK",
  "sets": [
    { "label": "Top single", "reps": {"min": 1, "max": 1}, "rpe": 8.0, "rest_sec": 180, "track_pr": true },
    { "label": "Back‑offs",  "reps": {"min": 5, "max": 5}, "sets": 5, "anchor": { "of_set_index": 0, "multiplier": 0.80 }, "rest_sec": 150 }
  ]
}
```

**Timed example:**

```json
{
  "type": "scheme",
  "ex": "CORE.PLANK.WEIGHTED",
  "sets": [
    { "label": "Top load test", "reps": { "min": 1, "max": 1 }, "rpe": 8.0, "rest_sec": 120, "track_pr": true },
    { "label": "Anchored holds", "sets": 3, "time_sec": 40, "anchor": { "of_set_index": 0, "multiplier": 0.80 }, "rest_sec": 90 }
  ],
  "load_mode": "added"
}
```

> In a `scheme`, each entry in `sets` may either describe **one set** or a **bundle** via `sets: N`. Each set entry may use `reps` or `time_sec` (XOR).

### 4.8 `complex` (barbell complexes)

One **set** consists of a sequence of sub‑lifts performed under a single load.

```json
{
  "type": "complex",
  "anchor_load": { "mode": "pct_1rm", "ex": "CJ.BB", "pct": 0.70 },  // or {"mode":"fixed_kg","kg":60}
  "sets": 5,
  "rest_sec": 120,
  "sequence": [
    { "ex": "CLEAN.BB",      "reps": {"min": 1, "max": 1} },
    { "ex": "SQ.FRONT.BB",   "reps": {"min": 2, "max": 2} },
    { "ex": "JERK.BB",       "reps": {"min": 1, "max": 1} }
  ]
}
```

> Logging: treat each sub‑lift as its own `SetRecord` with the **same** `segment_id`, `set_num`, and a shared `superset_id` like `"CPLX"` for grouping.

### 4.9 `comment`

Non-executable segment to instruct the user between lifting segments.

```json
{ "type": "comment", "text": "Switch to cable station", "icon": "transition" }
```

### 4.10 `choose` (NEW)

Pick **N** segments from a list at runtime; supports rotation across sessions.

```json
{
  "type": "choose",
  "pick": 1,                              // number to execute
  "rotation": "weekly",                  // "weekly" | "session" | "random" | "none"
  "from": [
    { "type": "straight", "ex": "FACEPULL.CBL", "sets": 3, "reps": {"min": 12, "max": 15} },
    { "type": "straight", "ex": "REARDELTS.DB", "sets": 3, "reps": {"min": 12, "max": 15} },
    { "type": "straight", "ex": "LATRAISE.CBL", "sets": 3, "reps": {"min": 12, "max": 15} }
  ]
}
```

> UI records which variant was chosen and can rotate automatically per `rotation`.

---

## 5. Substitutions (`alt_group`)

* **Definition**: named list in `groups` mapping to exercise codes in `dictionary`.
* **Usage**: may appear on any executable segment and on items inside `superset`/`circuit`/`complex.sequence`.
* **Policy**: *replacement only*; never implies load, progression, or altered timing.

---

## 6. Validation Rules

* Every `ex` must exist in `dictionary`.
* Every `alt_group` must exist in `groups`, and all members must exist in `dictionary`.
* For any executable set/item: **`reps` XOR `time_sec`** must be present.
* `reps` must be an object `{min,max[,target]}` with `1 ≤ min ≤ max`.
* `time_sec` must be a positive integer or `{min,max[,target]}` with `1 ≤ min ≤ max`.
* `rest_sec` must be a positive integer or `{min,max}` with `min ≤ max`.
* If both `rpe` and `rir` appear on a set, RPE takes precedence.
* `sets` **xor** `sets_range`.
* `scheme.sets[*].rpe` may be a number or `{min,max}`.
* `superset.items.length === 2`; `circuit.items.length ≥ 3`.
* `intensifier.kind` ∈ {`dropset`,`rest_pause`,`myo`,`cluster`}.
* `anchor.of_set_index` must point to an earlier set in the same `scheme`.
* `anchor` refers to **load** only; timed sets can be anchored for load but not for time.
* `choose.from.length ≥ pick`; `rotation` ∈ {`weekly`,`session`,`random`,`none`}.
* `week_overrides[*].target.segment_idx` refers to `segments` index (0‑based).

---

## 7. CSV/Logging Mapping (→ `SetRecord`)

* **plan\_name**: `Plan.name`
* **day\_label**: `Day.label`
* **segment\_id**: 1‑based index of the segment within `Day.segments`
* **superset\_id**: `"A1/A2"` for supersets, `"CPLX"` for complexes (or generated), empty otherwise
* **ex\_code**: the resolved code after any swap
* **set\_num**: sequential within the segment (for `complex`, shared across sequence items)
* **reps / time\_sec / weight / unit / rpe / rir / rest\_sec / tempo**: from executed set; ranges resolved by user input and timers. When a set is timed, `reps` is empty and `time_sec` is populated with executed duration (seconds; actual, not target).
* **notes**: include intensifier summary (e.g., `dropset 15% x1`), VBT results if available, timed set indicator (e.g., `"timed"`), and for `choose` add `variant: <index|label>`
* **is\_warmup**: true for auto‑generated warm‑ups
* **tags / effort\_1to5**: user-entered or UI-derived

---

## 8. Worked Examples

### A) Anchored back‑offs after a top single

```json
{
  "type": "scheme",
  "ex": "SQ.BB.BACK",
  "sets": [
    { "label": "Top single", "reps": {"min": 1, "max": 1}, "rpe": 8.0, "rest_sec": 180, "track_pr": true },
    { "label": "Back‑offs",  "sets": 5, "reps": {"min": 5, "max": 5}, "anchor": { "of_set_index": 0, "multiplier": 0.80 }, "rest_sec": 150 }
  ]
}
```

### B) Complex (single load)

```json
{
  "type": "complex",
  "anchor_load": { "mode": "pct_1rm", "ex": "CJ.BB", "pct": 0.70 },
  "sets": 4,
  "rest_sec": 120,
  "sequence": [
    { "ex": "CLEAN.BB", "reps": {"min": 1, "max": 1} },
    { "ex": "SQ.FRONT.BB", "reps": {"min": 2, "max": 2} },
    { "ex": "JERK.BB", "reps": {"min": 1, "max": 1} }
  ]
}
```

### C) Set‑count range with auto‑stop and VBT

```json
{
  "type": "straight",
  "ex": "ROW.BB.PEND",
  "sets_range": { "min": 3, "max": 5 },
  "reps": { "min": 6, "max": 8 },
  "rpe": 8.0,
  "vbt": { "target_mps": 0.40, "loss_cap_pct": 20 },
  "auto_stop": { "reason": "velocity_loss_pct", "threshold": 20 }
}
```

### D) Contrast pairing (PAP)

```json
{
  "type": "superset",
  "label": "A1/A2",
  "pairing": "contrast",
  "rounds": 3,
  "rest_sec": 0,
  "rest_between_rounds_sec": 120,
  "items": [
    { "ex": "SQ.BB.BACK", "sets": 1, "reps": {"min": 2, "max": 2}, "rpe": 8.5 },
    { "ex": "JUMPS.BW.SQ", "sets": 1, "reps": {"min": 6, "max": 6} }
  ]
}
```

### E) Choose/Rotate accessory

```json
{
  "type": "choose",
  "pick": 1,
  "rotation": "weekly",
  "from": [
    { "type": "straight", "ex": "FACEPULL.CBL",  "sets": 3, "reps": {"min": 12, "max": 15} },
    { "type": "straight", "ex": "REARDELTS.DB",  "sets": 3, "reps": {"min": 12, "max": 15} },
    { "type": "straight", "ex": "LATRAISE.CBL",  "sets": 3, "reps": {"min": 12, "max": 15} }
  ]
}
```

---

## 9. Differences: v0.3 vs v0.2

* **Terminology:** `block` → **`segment`** everywhere. Day arrays are now `segments`. Logging `block_id` → **`segment_id`**. Overrides use `segment_idx`.
* **Scope:** cardio/conditioning is **not modeled**. Use `comment` segments to instruct non-lifting actions between segments.
* **Unified reps:** `reps` is **always** `{min,max[,target]}`.
* **New segment types:** `scheme` (mixed set prescriptions), `complex` (barbell complexes), and `choose` (pick/rotate variants).
* **Ranges for rest:** `rest_sec` can be a range `{min,max}` everywhere.
* **Effort targets:** `rpe` and/or `rir` allowed; `rpe` takes precedence when both present.
* **Intensifiers:** expanded structure adds **cluster** details.
* **Supersets/Circuits:** support `label`, `rounds`, `rest_between_rounds_sec`, item‑level `alt_group`/`intensifier`, and `pairing: "contrast"` for PAP work.
* **Anchors:** back‑offs can reference earlier sets within a `scheme`.
* **VBT:** optional `vbt` with `target_mps` and `loss_cap_pct`.
* **Tempo:** structured `tempo` object replaces free‑text.
* **Set counts:** optional `sets_range` plus `auto_stop` criteria (e.g., velocity loss, technique).
* **Bodyweight modes:** `load_mode` clarifies added vs assisted vs bodyweight.
* **Technique & Equipment:** per‑segment `technique` and `equipment_policy` fields; plan/day can also carry policies.
* **Day metadata:** optional `time_cap_min` and `goal` added.
