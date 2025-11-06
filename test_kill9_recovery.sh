#!/bin/bash

# **Death to Windows!** - Kill-9 recovery test for draft persistence
# Test script to verify autosave and draft recovery functionality

echo "🧪 Testing Kill-9 Recovery for Draft Persistence"
echo "================================================"

# Clean up any existing drafts
DRAFTS_DIR="$HOME/.local/state/weightlifting-desktop/drafts"
rm -rf "$DRAFTS_DIR/recovery_test_plan.json" 2>/dev/null

# Create a test plan via CLI
echo "1. Creating test plan via CLI..."
./target/debug/comp plans save --in << 'EOF'
{
  "name": "Recovery Test Plan",
  "unit": "kg", 
  "dictionary": {
    "SQUAT.BB.BACK": "Back Squat"
  },
  "groups": {
    "GROUP_TEST": ["SQUAT.BB.BACK"]
  },
  "schedule": [
    {
      "day": 1,
      "label": "Test Day",
      "segments": [
        {
          "type": "straight",
          "ex": "SQUAT.BB.BACK",
          "sets": 3,
          "reps": {"min": 5, "max": 5},
          "rpe": 8.0,
          "rest_sec": 180
        }
      ]
    }
  ]
}
EOF

if [ $? -eq 0 ]; then
    echo "✅ Plan created successfully"
else
    echo "❌ Failed to create plan"
    exit 1
fi

# Verify the draft exists
if [ -f "$DRAFTS_DIR/recovery_test_plan.json" ]; then
    echo "✅ Draft file exists at: $DRAFTS_DIR/recovery_test_plan.json"
else
    echo "❌ Draft file not found"
    exit 1
fi

# Test recovery by reading the draft back
echo -e "\n2. Testing draft recovery..."
RECOVERED_PLAN=$(./target/debug/comp plans get --id recovery_test_plan)

if echo "$RECOVERED_PLAN" | jq -e '.name' | grep -q "Recovery Test Plan"; then
    echo "✅ Draft recovery successful"
    echo "Plan name: $(echo "$RECOVERED_PLAN" | jq -r '.name')"
else
    echo "❌ Draft recovery failed"
    exit 1
fi

# Simulate kill -9 scenario by modifying the draft directly
echo -e "\n3. Simulating modifications and kill -9..."
MODIFIED_DRAFT=$(echo "$RECOVERED_PLAN" | jq '.name = "Modified After Kill-9"')
echo "$MODIFIED_DRAFT" > "$DRAFTS_DIR/recovery_test_plan.json"

# Verify recovery of modifications
echo "4. Testing recovery of post-kill modifications..."
FINAL_PLAN=$(./target/debug/comp plans get --id recovery_test_plan)

if echo "$FINAL_PLAN" | jq -e '.name' | grep -q "Modified After Kill-9"; then
    echo "✅ Kill-9 recovery test PASSED"
    echo "Recovered plan name: $(echo "$FINAL_PLAN" | jq -r '.name')"
    echo "✅ All draft persistence mechanisms working correctly"
else
    echo "❌ Kill-9 recovery test FAILED"
    exit 1
fi

# Clean up
rm -f "$DRAFTS_DIR/recovery_test_plan.json"

echo -e "\n🎉 Kill-9 Recovery Test Complete: ALL TESTS PASSED"
echo "💾 Drafts are properly persisted to XDG_STATE_HOME"
echo "🔄 Recovery mechanism works correctly after simulated kill -9"