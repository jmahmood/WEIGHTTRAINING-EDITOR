#!/usr/bin/env python3
import json
import sys
from pathlib import Path

if len(sys.argv) != 2:
    print("Usage: verify_plan_fields.py <plan.json>")
    sys.exit(2)

path = Path(sys.argv[1])
plan = json.loads(path.read_text())
errors = []

for day_index, day in enumerate(plan.get("schedule", [])):
    for seg_index, segment in enumerate(day.get("segments", [])):
        seg_type = segment.get("type")
        prefix = f"day {day_index + 1} segment {seg_index + 1} ({seg_type})"
        if seg_type in {"straight", "rpe", "percentage"}:
            if "rest_sec" not in segment:
                errors.append(f"{prefix} missing rest_sec")
            if "rpe" not in segment:
                errors.append(f"{prefix} missing rpe")
        elif seg_type == "scheme":
            for entry_index, entry in enumerate(segment.get("sets", [])):
                eid = f"{prefix} entry {entry_index + 1}"
                if "rest_sec" not in entry:
                    errors.append(f"{eid} missing rest_sec")
                if "rpe" not in entry:
                    errors.append(f"{eid} missing rpe")

if errors:
    print("Plan validation failed:")
    for err in errors:
        print(" -", err)
    sys.exit(1)

print("Plan looks good: rest/rpe present for all straight and scheme segments.")
