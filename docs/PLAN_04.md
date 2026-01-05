# **Plan JSON v0.4 — Full Unified Specification**

---

# **0. Overview**

Plan JSON v0.4 preserves the full v0.3 model and extends it with:

1. **Week-dependent segment overlays** (`per_week`)
2. **Group-level variant schemes** (`group_variants` and per-segment `group_role`)
3. **Non-weight load axes** for exercises (`load_axes` and per-segment `load_axis_target`)

No v0.3 field changes meaning or type.
Every v0.3 plan is valid v0.4.

---

# **1. Top-Level Plan Object**

```json
{
  "name": "Example Plan",
  "author": "Author",
  "source_url": "https://example.com",
  "license_note": "Usage note",
  "unit": "kg",                        // "kg" | "lb" | "bw"

  "dictionary": { "EX.CODE": "Exercise Name" },

  "groups": {
    "GROUP_NAME": ["EX.CODE", "EX.ALT1", "EX.ALT2"]
  },

  "group_variants": {
    "<group_name>": {
      "<role_name>": {
        "<ex_code>": GroupVariantConfig
      }
    }
  },

  "exercise_meta": {
    "EX.CODE": {
      // v0.3 metadata
      "equipment": ["barbell"],
      "home_friendly": true,

      // v0.4 extensions
      "load_axes": {
        "axis_name": {
          "kind": "categorical",        // or "ordinal"
          "values": ["a", "b", "c"]
        }
      }
    }
  },

  "phase": { "index": 1, "weeks": [1,2,3,4] },

  "week_overrides": {
    "4": [
      { "target": { "day": 1, "segment_idx": 0 }, "modifier": { "rpe_cap": 8.0 } }
    ]
  },

  "equipment_policy": { "allowed": [...], "forbidden": [...] },

  "progression": {
    "mode": "double_progression",
    "reps_first": true,
    "load_increment_kg": 2.5,
    "cap_rpe": 9.0
  },

  "warmup": {
    "pattern": "percent_of_top",
    "stages": [ {"pct_1rm":0.3,"reps":8}, {"pct_1rm":0.5,"reps":5} ],
    "round_to": 2.5,
    "merge_after_rounding": true
  },

  "schedule": [ /* Days */ ]
}
```

**Notes**

* `groups` define substitution pools (replacement only).
* `group_variants` define **role-specific per-exercise overrides**.
* `exercise_meta.load_axes` defines **non-weight load categories**.

---

# **2. Day Object**

```json
{
  "day": 1,
  "label": "Upper A",
  "time_cap_min": 45,                 // optional
  "goal": "Strength",                 // optional
  "equipment_policy": { "allowed": [...], "forbidden": [...] },
  "segments": [ /* Segment objects */ ]
}
```

---

# **3. Common Data Types**

### 3.1 Reps

```json
{ "min": 8, "max": 12, "target": 12 }
```

### 3.2 Rest

```json
90
// OR
{ "min": 90, "max": 180 }
```

### 3.3 Effort (RPE/RIR)

```json
"rpe": 8.5
"rir": 2
"rpe": { "min": 8, "max": 9 }
```

### 3.4 Intensifiers

Supports dropsets, rest-pause, myo reps, clusters.

```json
{
  "kind": "dropset", "when": "last_set",
  "drop_pct": 0.15, "steps": 1,
  "clusters": 3, "reps_per_cluster": 2, "intra_rest_sec": 20
}
```

### 3.5 Tempo

```json
{ "ecc":3, "bottom":0, "con":1, "top":0, "units":"sec" }
```

### 3.6 VBT

```json
{ "target_mps": 0.35, "loss_cap_pct": 20 }
```

### 3.7 Load Mode

```json
"load_mode": "added"        // "added" | "assisted" | "bodyweight_only"
```

### 3.8 Set Count Range & Auto-Stop

```json
"sets_range": { "min":3, "max":5 }
"auto_stop": { "reason": "velocity_loss_pct", "threshold": 20 }
```

### 3.9 Anchors

```json
"anchor": { "of_set_index": 0, "multiplier": 0.80 }
```

### 3.10 Optionality / Rotation

```json
"optional": true
"rotation": "weekly"        // for choose segments
```

### 3.11 Technique

```json
"technique": { "stance": "sumo", "rom": "2\" deficit" }
```

### 3.12 Per-Segment Equipment Policy

```json
"equipment_policy": { "allowed": ["barbell"], "forbidden": [] }
```

### 3.13 Timed Set

```json
"time_sec": 45
// OR
"time_sec": { "min":30, "max":60, "target":45 }
```

---

# **4. Segment Types**

The following definitions incorporate v0.4 functionality.

---

## **4.1 `straight`**

```json
{
  "type": "straight",
  "ex": "BP.DB.FLAT",
  "alt_group": "GROUP_CHEST_PRESS",
  "group_role": "heavy",              // NEW (v0.4)
  "sets": 3,
  "reps": { "min":10, "max":12 },     // OR time_sec
  "rest_sec": 120,
  "rir": 2,
  "tempo": { ... },
  "vbt": { ... },
  "load_mode": "added",
  "label": "Backoff",
  "optional": true,
  "technique": { ... },
  "equipment_policy": { ... },
  "intensifier": { ... },

  "per_week": {                       // NEW (v0.4)
    "2": { "reps": { "min":12, "max":15 } },
    "3": { "reps": { "min":8, "max":10 } }
  },

  "load_axis_target": {               // NEW (v0.4)
    "axis": "band_color",
    "target": "green"
  }
}
```

---

## **4.2 `rpe`**

Same fields as v0.3 plus **per_week**, **group_role**, **load_axis_target**.

---

## **4.3 `percentage`**

Now supports **per_week** fully.

```json
{
  "type": "percentage",
  "ex": "SQ.BB.BACK",
  "prescriptions": [
    { "sets":1, "reps":5, "pct_1rm":0.65 },
    { "sets":1, "reps":5, "pct_1rm":0.75 },
    { "sets":1, "reps":5, "pct_1rm":0.85,
      "intensifier": { "kind":"amrap", "when":"last_set" } }
  ],
  "per_week": {
    "2": { "prescriptions": [ ... ] },
    "3": { "prescriptions": [ ... ] }
  }
}
```

---

## **4.4 `amrap`**

Unchanged except that `per_week` and `group_role` may appear if needed.

---

## **4.5 `superset`**

Two items only; each item may use **group_role**, **per_week**, **load_axis_target**.

---

## **4.6 `circuit`**

Three or more items; same extension rules as superset.

---

## **4.7 `scheme`**

Supports anchors and all v0.4 overlays.

---

## **4.8 `complex`**

No changes except permitted v0.4 extensions: per_week, group_role, load_axis_target inside sequence items.

---

## **4.9 `comment`**

Non-executable, unchanged.

---

## **4.10 `choose`**

Unchanged structure; each option inside `from` supports v0.4 fields.

---

# **5. v0.4 Extensions (Detailed)**

---

# **5.1 Week-Dependent Segments: `per_week`**

### Structure:

```json
"per_week": {
  "1": { /* partial segment overlay */ },
  "2": { /* partial */ }
}
```

### Rules:

* Keys are **strings** representing 1-based week numbers.
* Values are **partial segments** of the *same type* as the base segment.
* Merge is **shallow**:

  * Scalar and object fields override.
  * Arrays **replace** the base arrays.
* If no entry for the current week, the base segment is used.

---

# **5.2 Group-Level Variant Schemes**

## **Top-Level: `group_variants`**

```json
"group_variants": {
  "GROUP_CHEST_PRESS": {
    "heavy": {
      "BP.DB.FLAT": { "reps": { "min":3, "max":6 } },
      "DIP.WT.STND": { "reps": { "min":6, "max":8 } }
    },
    "volume": {
      "BP.DB.FLAT": { "reps": { "min":8, "max":12 } }
    }
  }
}
```

## **Per-Segment: `group_role`**

```json
{
  "type": "straight",
  "ex": "BP.DB.FLAT",
  "alt_group": "GROUP_CHEST_PRESS",
  "group_role": "heavy"
}
```

### Runtime behavior:

1. Select the exercise (chosen swap or base).
2. If `group_role` specified and `group_variants[group][role][exercise]` exists:
   → Shallow-merge that variant into the segment (arrays replace).
3. Otherwise no variant applies.

---

# **5.3 Non-Weight Load Axes**

## **Exercise-level axes (`exercise_meta.load_axes`)**

```json
"exercise_meta": {
  "PULLUP.ASSISTED.BAND": {
    "load_axes": {
      "band_color": {
        "kind": "categorical",
        "values": ["red", "black", "purple", "green"]
      }
    }
  }
}
```

## **Segment-level axis targets (`load_axis_target`)**

```json
"load_axis_target": {
  "axis": "band_color",
  "target": "green"
}
```

### Semantics:

* UI should present choices from `values`.
* If `target` present → default selection.
* If the substituted exercise does **not** define that axis → ignore field.

---

# **6. Validation Rules**

* Every `ex` exists in `dictionary`.
* Every `alt_group` exists in `groups`, all members valid.
* For each executable set: **`reps` XOR `time_sec`**.
* `reps.min ≤ reps.max`; `time_sec.min ≤ max`.
* `rest_sec` positive or `{min ≤ max}`.
* If both RPE and RIR present → RPE wins.
* `sets` XOR `sets_range`.
* For schemes: anchors must reference earlier set indices.
* Superset must have exactly two items; circuit ≥ 3.
* Intensifier kind ∈ {dropset, rest_pause, myo, cluster}.
* choose.from.length ≥ pick.
* `group_role` must reference only roles defined under its group in `group_variants` (if present).
* `per_week` overlays must match segment type.

---

# **7. Resolution Order (Recommended Runtime)**

1. Determine current plan week.
2. Apply `per_week` overlay → `S_eff`.
3. Resolve exercise substitution (`alt_group`).
4. Apply `group_variants` based on `group_role` and chosen exercise.
5. Resolve `load_axis_target`.
6. Present final defaults for execution.

---

# **8. Summary of v0.4 Changes**

| Feature                          | Purpose                                                            |
| -------------------------------- | ------------------------------------------------------------------ |
| `per_week`                       | True week-dependent programming (e.g., 5/3/1, peaking cycles).     |
| `group_variants` + `group_role`  | Per-exercise tuning inside groups for different programming roles. |
| `load_axes` + `load_axis_target` | Formal modeling of machine settings, band colors, etc.             |

---

# **9. JSON Schema

```
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "$id": "https://example.com/schemas/plan-v0.4.json",
  "title": "Plan JSON v0.4",
  "type": "object",
  "additionalProperties": false,
  "required": ["name", "unit", "dictionary", "schedule"],
  "properties": {
    "name": { "type": "string" },
    "author": { "type": "string" },
    "source_url": { "type": "string", "format": "uri" },
    "license_note": { "type": "string" },
    "unit": {
      "type": "string",
      "enum": ["kg", "lb", "bw"]
    },
    "dictionary": {
      "type": "object",
      "description": "Exercise code → human-friendly name",
      "additionalProperties": { "type": "string" }
    },
    "groups": {
      "type": "object",
      "description": "Named substitution groups: alt_group name → array of exercise codes",
      "additionalProperties": {
        "type": "array",
        "items": { "type": "string" }
      }
    },
    "group_variants": {
      "type": "object",
      "description": "Per-group, per-role, per-exercise overrides",
      "additionalProperties": {
        "type": "object",
        "description": "role_name → { ex_code → GroupVariantConfig }",
        "additionalProperties": {
          "$ref": "#/definitions/GroupVariantRoleMap"
        }
      }
    },
    "exercise_meta": {
      "type": "object",
      "description": "Per-exercise metadata (equipment, load axes, etc.)",
      "additionalProperties": {
        "type": "object",
        "additionalProperties": false,
        "properties": {
          "equipment": {
            "type": "array",
            "items": { "type": "string" }
          },
          "home_friendly": { "type": "boolean" },
          "load_axes": {
            "type": "object",
            "description": "Non-weight load axes such as band color, machine notch, etc.",
            "additionalProperties": {
              "$ref": "#/definitions/LoadAxis"
            }
          }
        }
      }
    },
    "phase": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "index": { "type": "integer" },
        "weeks": {
          "type": "array",
          "items": { "type": "integer", "minimum": 1 }
        }
      }
    },
    "week_overrides": {
      "type": "object",
      "description": "Week-level tweaks (e.g., deload modifiers)",
      "additionalProperties": {
        "type": "array",
        "items": {
          "type": "object",
          "additionalProperties": false,
          "properties": {
            "target": {
              "type": "object",
              "additionalProperties": false,
              "properties": {
                "day": { "type": "integer", "minimum": 1 },
                "segment_idx": { "type": "integer", "minimum": 0 }
              }
            },
            "modifier": {
              "type": "object",
              "description": "Partial segment-like modifier object; semantics defined by implementation"
            }
          }
        }
      }
    },
    "equipment_policy": { "$ref": "#/definitions/EquipmentPolicy" },
    "progression": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "mode": { "type": "string" },
        "reps_first": { "type": "boolean" },
        "load_increment_kg": { "type": "number" },
        "cap_rpe": { "type": "number" }
      }
    },
    "warmup": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "pattern": { "type": "string" },
        "stages": {
          "type": "array",
          "items": {
            "type": "object",
            "additionalProperties": false,
            "required": ["pct_1rm", "reps"],
            "properties": {
              "pct_1rm": { "type": "number" },
              "reps": { "type": "integer", "minimum": 1 }
            }
          }
        },
        "round_to": { "type": "number" },
        "merge_after_rounding": { "type": "boolean" }
      }
    },
    "schedule": {
      "type": "array",
      "items": { "$ref": "#/definitions/Day" }
    }
  },

  "definitions": {
    "Day": {
      "type": "object",
      "additionalProperties": false,
      "required": ["day", "label", "segments"],
      "properties": {
        "day": { "type": "integer", "minimum": 1 },
        "label": { "type": "string" },
        "time_cap_min": { "type": "integer", "minimum": 1 },
        "goal": { "type": "string" },
        "equipment_policy": { "$ref": "#/definitions/EquipmentPolicy" },
        "segments": {
          "type": "array",
          "items": { "$ref": "#/definitions/Segment" }
        }
      }
    },

    "EquipmentPolicy": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "allowed": {
          "type": "array",
          "items": { "type": "string" }
        },
        "forbidden": {
          "type": "array",
          "items": { "type": "string" }
        }
      }
    },

    "Reps": {
      "type": "object",
      "additionalProperties": false,
      "required": ["min", "max"],
      "properties": {
        "min": { "type": "integer", "minimum": 1 },
        "max": { "type": "integer", "minimum": 1 },
        "target": { "type": "integer", "minimum": 1 }
      }
    },

    "TimeRange": {
      "oneOf": [
        { "type": "integer", "minimum": 1 },
        {
          "type": "object",
          "additionalProperties": false,
          "required": ["min", "max"],
          "properties": {
            "min": { "type": "integer", "minimum": 1 },
            "max": { "type": "integer", "minimum": 1 },
            "target": { "type": "integer", "minimum": 1 }
          }
        }
      ]
    },

    "Rest": {
      "oneOf": [
        { "type": "integer", "minimum": 0 },
        {
          "type": "object",
          "additionalProperties": false,
          "required": ["min", "max"],
          "properties": {
            "min": { "type": "integer", "minimum": 0 },
            "max": { "type": "integer", "minimum": 0 }
          }
        }
      ]
    },

    "Tempo": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "ecc": { "type": "integer", "minimum": 0 },
        "bottom": { "type": "integer", "minimum": 0 },
        "con": { "type": "integer", "minimum": 0 },
        "top": { "type": "integer", "minimum": 0 },
        "units": { "type": "string" }
      }
    },

    "VBT": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "target_mps": { "type": "number" },
        "loss_cap_pct": { "type": "number" }
      }
    },

    "Intensifier": {
      "type": "object",
      "additionalProperties": false,
      "required": ["kind"],
      "properties": {
        "kind": {
          "type": "string",
          "enum": ["dropset", "rest_pause", "myo", "cluster", "amrap"]
        },
        "when": {
          "type": "string",
          "enum": ["each_set", "last_set"]
        },
        "drop_pct": { "type": "number" },
        "steps": { "type": "integer", "minimum": 1 },
        "clusters": { "type": "integer", "minimum": 1 },
        "reps_per_cluster": { "type": "integer", "minimum": 1 },
        "intra_rest_sec": { "type": "integer", "minimum": 0 }
      }
    },

    "SetsRange": {
      "type": "object",
      "additionalProperties": false,
      "required": ["min", "max"],
      "properties": {
        "min": { "type": "integer", "minimum": 1 },
        "max": { "type": "integer", "minimum": 1 }
      }
    },

    "AutoStop": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "reason": { "type": "string" },
        "threshold": { "type": "number" }
      }
    },

    "Anchor": {
      "type": "object",
      "additionalProperties": false,
      "required": ["of_set_index", "multiplier"],
      "properties": {
        "of_set_index": { "type": "integer", "minimum": 0 },
        "multiplier": { "type": "number" }
      }
    },

    "Technique": {
      "type": "object",
      "description": "Free-form technique hints (stance, grip, ROM, etc.)",
      "additionalProperties": { "type": "string" }
    },

    "LoadAxis": {
      "type": "object",
      "additionalProperties": false,
      "required": ["kind", "values"],
      "properties": {
        "kind": {
          "type": "string",
          "enum": ["categorical", "ordinal"]
        },
        "values": {
          "type": "array",
          "items": { "type": "string" },
          "minItems": 1
        }
      }
    },

    "LoadAxisTarget": {
      "type": "object",
      "additionalProperties": false,
      "required": ["axis"],
      "properties": {
        "axis": { "type": "string" },
        "target": { "type": "string" }
      }
    },

    "GroupVariantConfig": {
      "type": "object",
      "description": "Partial segment-like configuration for a specific exercise in a group role",
      "additionalProperties": false,
      "properties": {
        "reps": { "$ref": "#/definitions/Reps" },
        "time_sec": { "$ref": "#/definitions/TimeRange" },
        "rest_sec": { "$ref": "#/definitions/Rest" },
        "rpe": {
          "oneOf": [
            { "type": "number" },
            {
              "type": "object",
              "additionalProperties": false,
              "required": ["min", "max"],
              "properties": {
                "min": { "type": "number" },
                "max": { "type": "number" }
              }
            }
          ]
        },
        "rir": {
          "oneOf": [
            { "type": "number" },
            {
              "type": "object",
              "additionalProperties": false,
              "required": ["min", "max"],
              "properties": {
                "min": { "type": "number" },
                "max": { "type": "number" }
              }
            }
          ]
        },
        "tempo": { "$ref": "#/definitions/Tempo" },
        "vbt": { "$ref": "#/definitions/VBT" },
        "load_mode": {
          "type": "string",
          "enum": ["added", "assisted", "bodyweight_only"]
        },
        "sets": { "type": "integer", "minimum": 1 },
        "sets_range": { "$ref": "#/definitions/SetsRange" },
        "auto_stop": { "$ref": "#/definitions/AutoStop" },
        "intensifier": { "$ref": "#/definitions/Intensifier" }
      }
    },

    "GroupVariantRoleMap": {
      "type": "object",
      "description": "ex_code → GroupVariantConfig",
      "additionalProperties": { "$ref": "#/definitions/GroupVariantConfig" }
    },

    "BaseSegmentCommon": {
      "type": "object",
      "properties": {
        "label": { "type": "string" },
        "alt_group": { "type": "string" },
        "group_role": {
          "type": "string",
          "description": "Role name used to select per-exercise overrides within group_variants"
        },
        "optional": { "type": "boolean" },
        "technique": { "$ref": "#/definitions/Technique" },
        "equipment_policy": { "$ref": "#/definitions/EquipmentPolicy" },
        "load_axis_target": { "$ref": "#/definitions/LoadAxisTarget" }
      }
    },

    "PerWeekOverlay": {
      "type": "object",
      "description": "Week-indexed overlays; keys are 1-based integers as strings",
      "patternProperties": {
        "^[1-9][0-9]*$": {
          "type": "object",
          "description": "Partial segment object for that week; semantics defined by the 'type' of the base segment"
        }
      },
      "additionalProperties": false
    },

    "StraightSegment": {
      "allOf": [
        { "$ref": "#/definitions/BaseSegmentCommon" },
        {
          "type": "object",
          "additionalProperties": false,
          "required": ["type", "ex"],
          "properties": {
            "type": {
              "type": "string",
              "const": "straight"
            },
            "ex": { "type": "string" },
            "sets": { "type": "integer", "minimum": 1 },
            "sets_range": { "$ref": "#/definitions/SetsRange" },
            "reps": { "$ref": "#/definitions/Reps" },
            "time_sec": { "$ref": "#/definitions/TimeRange" },
            "rest_sec": { "$ref": "#/definitions/Rest" },
            "rpe": {
              "oneOf": [
                { "type": "number" },
                {
                  "type": "object",
                  "additionalProperties": false,
                  "required": ["min", "max"],
                  "properties": {
                    "min": { "type": "number" },
                    "max": { "type": "number" }
                  }
                }
              ]
            },
            "rir": {
              "oneOf": [
                { "type": "number" },
                {
                  "type": "object",
                  "additionalProperties": false,
                  "required": ["min", "max"],
                  "properties": {
                    "min": { "type": "number" },
                    "max": { "type": "number" }
                  }
                }
              ]
            },
            "tempo": { "$ref": "#/definitions/Tempo" },
            "vbt": { "$ref": "#/definitions/VBT" },
            "load_mode": {
              "type": "string",
              "enum": ["added", "assisted", "bodyweight_only"]
            },
            "auto_stop": { "$ref": "#/definitions/AutoStop" },
            "intensifier": { "$ref": "#/definitions/Intensifier" },
            "per_week": { "$ref": "#/definitions/PerWeekOverlay" }
          },
          "oneOf": [
            { "required": ["reps"] },
            { "required": ["time_sec"] }
          ],
          "not": { "required": ["sets", "sets_range"] }
        }
      ]
    },

    "RPESegment": {
      "allOf": [
        { "$ref": "#/definitions/BaseSegmentCommon" },
        {
          "type": "object",
          "additionalProperties": false,
          "required": ["type", "ex", "sets"],
          "properties": {
            "type": {
              "type": "string",
              "const": "rpe"
            },
            "ex": { "type": "string" },
            "sets": { "type": "integer", "minimum": 1 },
            "reps": { "$ref": "#/definitions/Reps" },
            "time_sec": { "$ref": "#/definitions/TimeRange" },
            "rpe": {
              "oneOf": [
                { "type": "number" },
                {
                  "type": "object",
                  "additionalProperties": false,
                  "required": ["min", "max"],
                  "properties": {
                    "min": { "type": "number" },
                    "max": { "type": "number" }
                  }
                }
              ]
            },
            "rir": {
              "oneOf": [
                { "type": "number" },
                {
                  "type": "object",
                  "additionalProperties": false,
                  "required": ["min", "max"],
                  "properties": {
                    "min": { "type": "number" },
                    "max": { "type": "number" }
                  }
                }
              ]
            },
            "rest_sec": { "$ref": "#/definitions/Rest" },
            "anchor": { "$ref": "#/definitions/Anchor" },
            "vbt": { "$ref": "#/definitions/VBT" },
            "auto_stop": { "$ref": "#/definitions/AutoStop" },
            "intensifier": { "$ref": "#/definitions/Intensifier" },
            "per_week": { "$ref": "#/definitions/PerWeekOverlay" }
          },
          "oneOf": [
            { "required": ["reps"] },
            { "required": ["time_sec"] }
          ]
        }
      ]
    },

    "PercentagePrescription": {
      "type": "object",
      "additionalProperties": false,
      "required": ["sets", "reps", "pct_1rm"],
      "properties": {
        "sets": { "type": "integer", "minimum": 1 },
        "reps": { "$ref": "#/definitions/Reps" },
        "pct_1rm": { "type": "number" },
        "intensifier": { "$ref": "#/definitions/Intensifier" }
      }
    },

    "PercentageSegment": {
      "allOf": [
        { "$ref": "#/definitions/BaseSegmentCommon" },
        {
          "type": "object",
          "additionalProperties": false,
          "required": ["type", "ex", "prescriptions"],
          "properties": {
            "type": {
              "type": "string",
              "const": "percentage"
            },
            "ex": { "type": "string" },
            "prescriptions": {
              "type": "array",
              "items": { "$ref": "#/definitions/PercentagePrescription" },
              "minItems": 1
            },
            "per_week": { "$ref": "#/definitions/PerWeekOverlay" }
          }
        }
      ]
    },

    "AmrapSegment": {
      "allOf": [
        { "$ref": "#/definitions/BaseSegmentCommon" },
        {
          "type": "object",
          "additionalProperties": false,
          "required": ["type", "ex", "base_reps"],
          "properties": {
            "type": {
              "type": "string",
              "const": "amrap"
            },
            "ex": { "type": "string" },
            "base_reps": { "type": "integer", "minimum": 1 },
            "cap_reps": { "type": "integer", "minimum": 1 },
            "per_week": { "$ref": "#/definitions/PerWeekOverlay" }
          }
        }
      ]
    },

    "SupersetItem": {
      "allOf": [
        { "$ref": "#/definitions/BaseSegmentCommon" },
        {
          "type": "object",
          "additionalProperties": false,
          "required": ["ex"],
          "properties": {
            "ex": { "type": "string" },
            "sets": { "type": "integer", "minimum": 1 },
            "sets_range": { "$ref": "#/definitions/SetsRange" },
            "reps": { "$ref": "#/definitions/Reps" },
            "time_sec": { "$ref": "#/definitions/TimeRange" },
            "rpe": {
              "oneOf": [
                { "type": "number" },
                {
                  "type": "object",
                  "additionalProperties": false,
                  "required": ["min", "max"],
                  "properties": {
                    "min": { "type": "number" },
                    "max": { "type": "number" }
                  }
                }
              ]
            },
            "rir": {
              "oneOf": [
                { "type": "number" },
                {
                  "type": "object",
                  "additionalProperties": false,
                  "required": ["min", "max"],
                  "properties": {
                    "min": { "type": "number" },
                    "max": { "type": "number" }
                  }
                }
              ]
            },
            "rest_sec": { "$ref": "#/definitions/Rest" },
            "intensifier": { "$ref": "#/definitions/Intensifier" },
            "per_week": { "$ref": "#/definitions/PerWeekOverlay" }
          },
          "oneOf": [
            { "required": ["reps"] },
            { "required": ["time_sec"] }
          ]
        }
      ]
    },

    "SupersetSegment": {
      "type": "object",
      "additionalProperties": false,
      "required": ["type", "items", "rounds"],
      "properties": {
        "type": {
          "type": "string",
          "const": "superset"
        },
        "label": { "type": "string" },
        "pairing": {
          "type": "string",
          "enum": ["standard", "contrast"]
        },
        "rounds": { "type": "integer", "minimum": 1 },
        "rest_sec": { "$ref": "#/definitions/Rest" },
        "rest_between_rounds_sec": { "$ref": "#/definitions/Rest" },
        "items": {
          "type": "array",
          "minItems": 2,
          "maxItems": 2,
          "items": { "$ref": "#/definitions/SupersetItem" }
        },
        "per_week": { "$ref": "#/definitions/PerWeekOverlay" }
      }
    },

    "CircuitSegment": {
      "type": "object",
      "additionalProperties": false,
      "required": ["type", "items", "rounds"],
      "properties": {
        "type": {
          "type": "string",
          "const": "circuit"
        },
        "label": { "type": "string" },
        "rounds": { "type": "integer", "minimum": 1 },
        "rest_sec": { "$ref": "#/definitions/Rest" },
        "rest_between_rounds_sec": { "$ref": "#/definitions/Rest" },
        "items": {
          "type": "array",
          "minItems": 3,
          "items": { "$ref": "#/definitions/SupersetItem" }
        },
        "per_week": { "$ref": "#/definitions/PerWeekOverlay" }
      }
    },

    "SchemeSetEntry": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "label": { "type": "string" },
        "sets": { "type": "integer", "minimum": 1 },
        "reps": { "$ref": "#/definitions/Reps" },
        "time_sec": { "$ref": "#/definitions/TimeRange" },
        "rpe": {
          "oneOf": [
            { "type": "number" },
            {
              "type": "object",
              "additionalProperties": false,
              "required": ["min", "max"],
              "properties": {
                "min": { "type": "number" },
                "max": { "type": "number" }
              }
            }
          ]
        },
        "rir": {
          "oneOf": [
            { "type": "number" },
            {
              "type": "object",
              "additionalProperties": false,
              "required": ["min", "max"],
              "properties": {
                "min": { "type": "number" },
                "max": { "type": "number" }
              }
            }
          ]
        },
        "rest_sec": { "$ref": "#/definitions/Rest" },
        "anchor": { "$ref": "#/definitions/Anchor" },
        "track_pr": { "type": "boolean" },
        "intensifier": { "$ref": "#/definitions/Intensifier" }
      },
      "oneOf": [
        { "required": ["reps"] },
        { "required": ["time_sec"] }
      ]
    },

    "SchemeSegment": {
      "allOf": [
        { "$ref": "#/definitions/BaseSegmentCommon" },
        {
          "type": "object",
          "additionalProperties": false,
          "required": ["type", "ex", "sets"],
          "properties": {
            "type": {
              "type": "string",
              "const": "scheme"
            },
            "ex": { "type": "string" },
            "sets": {
              "type": "array",
              "items": { "$ref": "#/definitions/SchemeSetEntry" },
              "minItems": 1
            },
            "load_mode": {
              "type": "string",
              "enum": ["added", "assisted", "bodyweight_only"]
            },
            "per_week": { "$ref": "#/definitions/PerWeekOverlay" }
          }
        }
      ]
    },

    "ComplexSequenceEntry": {
      "type": "object",
      "additionalProperties": false,
      "required": ["ex", "reps"],
      "properties": {
        "ex": { "type": "string" },
        "reps": { "$ref": "#/definitions/Reps" },
        "group_role": { "type": "string" },
        "load_axis_target": { "$ref": "#/definitions/LoadAxisTarget" },
        "per_week": { "$ref": "#/definitions/PerWeekOverlay" }
      }
    },

    "ComplexSegment": {
      "type": "object",
      "additionalProperties": false,
      "required": ["type", "anchor_load", "sets", "sequence"],
      "properties": {
        "type": {
          "type": "string",
          "const": "complex"
        },
        "label": { "type": "string" },
        "anchor_load": {
          "type": "object",
          "additionalProperties": false,
          "properties": {
            "mode": { "type": "string" },
            "ex": { "type": "string" },
            "pct": { "type": "number" },
            "kg": { "type": "number" }
          }
        },
        "sets": { "type": "integer", "minimum": 1 },
        "rest_sec": { "$ref": "#/definitions/Rest" },
        "sequence": {
          "type": "array",
          "minItems": 1,
          "items": { "$ref": "#/definitions/ComplexSequenceEntry" }
        },
        "per_week": { "$ref": "#/definitions/PerWeekOverlay" }
      }
    },

    "CommentSegment": {
      "type": "object",
      "additionalProperties": false,
      "required": ["type", "text"],
      "properties": {
        "type": {
          "type": "string",
          "const": "comment"
        },
        "text": { "type": "string" },
        "icon": { "type": "string" }
      }
    },

    "ChooseSegment": {
      "type": "object",
      "additionalProperties": false,
      "required": ["type", "pick", "from"],
      "properties": {
        "type": {
          "type": "string",
          "const": "choose"
        },
        "pick": { "type": "integer", "minimum": 1 },
        "rotation": {
          "type": "string",
          "enum": ["weekly", "session", "random", "none"]
        },
        "from": {
          "type": "array",
          "minItems": 1,
          "items": {
            "$ref": "#/definitions/Segment"
          }
        }
      }
    },

    "Segment": {
      "description": "One of the supported segment types",
      "oneOf": [
        { "$ref": "#/definitions/StraightSegment" },
        { "$ref": "#/definitions/RPESegment" },
        { "$ref": "#/definitions/PercentageSegment" },
        { "$ref": "#/definitions/AmrapSegment" },
        { "$ref": "#/definitions/SupersetSegment" },
        { "$ref": "#/definitions/CircuitSegment" },
        { "$ref": "#/definitions/SchemeSegment" },
        { "$ref": "#/definitions/ComplexSegment" },
        { "$ref": "#/definitions/CommentSegment" },
        { "$ref": "#/definitions/ChooseSegment" }
      ]
    }
  }
}
```