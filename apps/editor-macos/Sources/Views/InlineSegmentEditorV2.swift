import SwiftUI

/// Truly modeless inline editor - fields are always editable
/// Follows Jef Raskin's Humane Interface principles
struct InlineSegmentEditorV2: View {
    @ObservedObject var plan: PlanDocument
    let segment: SegmentDisplay
    @EnvironmentObject var appState: AppState

    @State private var sets: Int = 3
    @State private var reps: Int = 8
    @State private var useRepsRange = false
    @State private var repsMin: Int = 8
    @State private var repsMax: Int = 12
    @State private var restSec: Int = 120
    @State private var rpe: Double = 8.0
    @State private var notes: String = ""
    @State private var exerciseCode: String = ""
    @State private var exerciseName: String = ""
    @State private var altGroup: String? = nil
    @State private var groupRole: String? = nil
    @State private var perWeekJSON: String = ""
    @State private var loadAxisTarget: LoadAxisTarget? = nil

    // Track if ANY field has been modified
    @State private var hasChanges = false
    @State private var activeSegment: SegmentDisplay? = nil

    @FocusState private var focusedField: Field?

    enum Field: Hashable {
        case sets, reps, repsMin, repsMax, rest, rpe, notes
    }

    private var isFirstResponder: Bool {
        focusedField != nil
    }

    private var usesExercise: Bool {
        ["straight", "rpe", "percentage", "amrap"].contains(segment.type)
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            // Only show save/revert when there are changes
            if hasChanges {
                HStack {
                    VStack(alignment: .leading, spacing: 2) {
                        Text("Unsaved changes")
                            .font(.caption)
                            .foregroundColor(.orange)
                        Text("⌘↩ to save • Esc to revert")
                            .font(.caption2)
                            .foregroundColor(.secondary)
                    }
                    Spacer()
                    Button("Revert (Esc)") {
                        loadCurrentValues()
                        hasChanges = false
                    }
                    .keyboardShortcut(.escape, modifiers: [])

                    Button("Save (⌘↩)") {
                        saveCurrentChanges()
                    }
                    .buttonStyle(.borderedProminent)
                    .keyboardShortcut(.return, modifiers: [.command])
                }
                .padding(.bottom, 8)
            }

            // Always-editable fields
            if usesExercise {
                VStack(alignment: .leading, spacing: 4) {
                    Text("Exercise")
                        .font(.caption)
                    ExercisePicker(
                        plan: plan,
                        selectedExerciseCode: $exerciseCode,
                        selectedExerciseName: $exerciseName
                    )
                    .onChange(of: exerciseCode) { _ in hasChanges = true }
                }
            }

            HStack {
                Text("Sets:")
                    .frame(width: 60, alignment: .leading)
                TextField("", value: $sets, format: .number)
                    .textFieldStyle(.roundedBorder)
                    .frame(width: 60)
                    .focused($focusedField, equals: .sets)
                    .onChange(of: sets) { _ in hasChanges = true }
                    .onSubmit { focusedField = useRepsRange ? .repsMin : .reps }
            }

            Toggle("Rep Range", isOn: $useRepsRange)
                .onChange(of: useRepsRange) { _ in hasChanges = true }

            if useRepsRange {
                HStack {
                    Text("Min:")
                        .frame(width: 60, alignment: .leading)
                    TextField("", value: $repsMin, format: .number)
                        .textFieldStyle(.roundedBorder)
                        .frame(width: 60)
                        .focused($focusedField, equals: .repsMin)
                        .onChange(of: repsMin) { _ in hasChanges = true }
                        .onSubmit { focusedField = .repsMax }

                    Text("Max:")
                        .frame(width: 60, alignment: .leading)
                    TextField("", value: $repsMax, format: .number)
                        .textFieldStyle(.roundedBorder)
                        .frame(width: 60)
                        .focused($focusedField, equals: .repsMax)
                        .onChange(of: repsMax) { _ in hasChanges = true }
                        .onSubmit { focusedField = .rest }
                }
            } else {
                HStack {
                    Text("Reps:")
                        .frame(width: 60, alignment: .leading)
                    TextField("", value: $reps, format: .number)
                        .textFieldStyle(.roundedBorder)
                        .frame(width: 60)
                        .focused($focusedField, equals: .reps)
                        .onChange(of: reps) { _ in hasChanges = true }
                        .onSubmit { focusedField = .rest }
                }
            }

            HStack {
                Text("Rest (sec):")
                    .frame(width: 80, alignment: .leading)
                TextField("", value: $restSec, format: .number)
                    .textFieldStyle(.roundedBorder)
                    .frame(width: 80)
                    .focused($focusedField, equals: .rest)
                    .onChange(of: restSec) { _ in hasChanges = true }
                    .onSubmit { focusedField = .notes }
            }

            HStack {
                Text("RPE:")
                    .frame(width: 60, alignment: .leading)
                Slider(value: $rpe, in: 6...10, step: 0.5)
                    .onChange(of: rpe) { _ in hasChanges = true }
                Text(String(format: "%.1f", rpe))
                    .frame(width: 40)
            }

            VStack(alignment: .leading, spacing: 4) {
                Text("Notes:")
                    .font(.caption)
                TextEditor(text: $notes)
                    .frame(height: 60)
                    .focused($focusedField, equals: .notes)
                    .border(Color.gray.opacity(0.3))
                    .onChange(of: notes) { _ in hasChanges = true }
            }

            Divider()

            // Alternative Group (optional)
            VStack(alignment: .leading, spacing: 4) {
                Text("Alternative Group (Optional):")
                    .font(.caption)
                GroupPicker(
                    plan: plan,
                    selectedGroup: $altGroup,
                    onChange: {
                        hasChanges = true
                        saveCurrentChanges()
                    }
                )
            }

            VStack(alignment: .leading, spacing: 8) {
                Text("Group Focus")
                    .font(.caption)
                GroupRolePicker(
                    groupId: altGroup,
                    availableRoles: altGroup.map { plan.getRolesForGroup($0) } ?? [],
                    selectedRole: $groupRole
                )
                .onChange(of: groupRole) { _ in
                    hasChanges = true
                    saveCurrentChanges()
                }
                    if groupRole != nil && altGroup == nil {
                        Text("Group focus requires an alternative group.")
                            .font(.caption)
                            .foregroundColor(.orange)
                    }

                Text("Per-Week Overlay (JSON)")
                    .font(.caption)
                TextEditor(text: $perWeekJSON)
                    .font(.system(.body, design: .monospaced))
                    .frame(height: 70)
                    .border(Color.gray.opacity(0.3))
                    .onChange(of: perWeekJSON) { _ in hasChanges = true }

                Text("Resistance Target")
                    .font(.caption)
                if !exerciseCode.isEmpty {
                    let loadAxes = plan.getLoadAxesForExercise(exerciseCode)
                    if loadAxes.isEmpty {
                        Text("No alternative resistance types defined for this exercise.")
                            .font(.caption)
                            .foregroundColor(.secondary)
                    } else {
                        LoadAxisTargetPicker(
                            availableAxes: loadAxes,
                            target: $loadAxisTarget
                        )
                        .onChange(of: loadAxisTarget) { _ in hasChanges = true }
                    }
                    if loadAxisTarget != nil && loadAxes.isEmpty {
                        Text("Resistance target is set but no resistance types are defined.")
                            .font(.caption)
                            .foregroundColor(.orange)
                    }
                } else {
                    Text("Select an exercise to use resistance types.")
                        .font(.caption)
                        .foregroundColor(.secondary)
                }
            }
        }
        .onAppear {
            activeSegment = segment
            loadCurrentValues()
        }
        .onExitCommand {
            if hasChanges {
                loadCurrentValues()
                hasChanges = false
            }
        }
        .onChange(of: segment.id) { _ in
            if hasChanges {
                saveCurrentChanges()
            }
            activeSegment = segment
            loadCurrentValues()
        }
        .onChange(of: appState.shouldFocusInspector) { shouldFocus in
            if shouldFocus {
                focusedField = .sets
                appState.shouldFocusInspector = false
            }
        }
        .onDisappear {
            if hasChanges {
                saveCurrentChanges()
            }
        }
    }

    private func saveCurrentChanges() {
        saveChanges(for: activeSegment ?? segment)
    }

    private func saveChanges(for targetSegment: SegmentDisplay) {
        ErrorLogger.shared.info("Saving inline edit for segment \(targetSegment.id)")
        ErrorLogger.shared.info("altGroup: \(String(describing: altGroup)), groupRole: \(String(describing: groupRole))")

        var updatedDict = targetSegment.segmentDict

        if updatedDict["type"] == nil {
            updatedDict["type"] = targetSegment.type
        }

        updatedDict["sets"] = sets
        if !exerciseCode.isEmpty {
            updatedDict["ex"] = exerciseCode
        } else {
            updatedDict.removeValue(forKey: "ex")
        }

        if useRepsRange {
            updatedDict["reps"] = ["min": repsMin, "max": repsMax]
        } else {
            updatedDict["reps"] = reps
        }

        updatedDict["rest_sec"] = restSec
        updatedDict["rpe"] = rpe

        if !notes.isEmpty {
            updatedDict["note"] = notes
        } else {
            updatedDict.removeValue(forKey: "note")
        }

        if let altGroup = altGroup, !altGroup.isEmpty {
            updatedDict["alt_group"] = altGroup
        } else {
            updatedDict.removeValue(forKey: "alt_group")
        }

        if let groupRole = groupRole,
           !groupRole.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty {
            if let altGroup = altGroup {
                plan.ensureGroupRoleExists(groupId: altGroup, roleId: groupRole)
            }
            updatedDict["group_role"] = groupRole
            ErrorLogger.shared.info("Setting group_role to: \(groupRole)")
        } else {
            updatedDict.removeValue(forKey: "group_role")
            ErrorLogger.shared.info("Removing group_role (was: \(String(describing: groupRole)))")
        }

        if !perWeekJSON.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty {
            if let data = perWeekJSON.data(using: .utf8),
               let obj = try? JSONSerialization.jsonObject(with: data) as? [String: Any] {
                updatedDict["per_week"] = obj
            }
        } else {
            updatedDict.removeValue(forKey: "per_week")
        }

        if let target = loadAxisTarget {
            updatedDict["load_axis_target"] = [
                "axis": target.axis,
                "target": target.target
            ]
        } else {
            updatedDict.removeValue(forKey: "load_axis_target")
        }

        do {
            let jsonData = try JSONSerialization.data(withJSONObject: updatedDict, options: .sortedKeys)
            if let jsonString = String(data: jsonData, encoding: .utf8) {
                ErrorLogger.shared.info("JSON being saved: \(jsonString)")
                appState.pushUndo(plan.planJSON, label: "Edit Segment")
                try plan.updateSegment(jsonString, at: targetSegment.index, inDayAt: targetSegment.dayIndex)
                hasChanges = false
                ErrorLogger.shared.info("Successfully saved segment \(targetSegment.id)")
            }
        } catch {
            ErrorLogger.shared.error("Failed to save segment \(targetSegment.id): \(error.localizedDescription)")
        }
    }

    private func loadCurrentValues() {
        sets = segment.intValue("sets") ?? 3
        exerciseCode = segment.exerciseCode ?? ""
        if !exerciseCode.isEmpty {
            exerciseName = plan.dictionary[exerciseCode] ?? exerciseCode
        } else {
            exerciseName = ""
        }

        if let repsValue = segment.segmentDict["reps"] {
            if let repsDict = repsValue as? [String: Any],
               let min = repsDict["min"] as? Int,
               let max = repsDict["max"] as? Int {
                useRepsRange = true
                repsMin = min
                repsMax = max
                reps = 8
            } else if let singleReps = repsValue as? Int {
                useRepsRange = false
                reps = singleReps
                repsMin = 8
                repsMax = 12
            } else {
                useRepsRange = false
                reps = 8
                repsMin = 8
                repsMax = 12
            }
        } else {
            useRepsRange = false
            reps = 8
            repsMin = 8
            repsMax = 12
        }

        restSec = segment.restSeconds ?? 120
        rpe = segment.rpeValue ?? 8.0
        notes = segment.inspectorNote ?? ""
        altGroup = segment.altGroupCode
        groupRole = segment.segmentDict["group_role"] as? String
        ErrorLogger.shared.info("Loaded segment \(segment.id): altGroup=\(String(describing: altGroup)), groupRole=\(String(describing: groupRole))")
        if let perWeek = segment.segmentDict["per_week"],
           let data = try? JSONSerialization.data(withJSONObject: perWeek, options: [.prettyPrinted, .sortedKeys]),
           let str = String(data: data, encoding: .utf8) {
            perWeekJSON = str
        } else {
            perWeekJSON = ""
        }
        if let axisDict = segment.segmentDict["load_axis_target"] as? [String: Any],
           let axis = axisDict["axis"] as? String,
           let target = axisDict["target"] as? String {
            loadAxisTarget = LoadAxisTarget(axis: axis, target: target)
        } else {
            loadAxisTarget = nil
        }
        hasChanges = false
    }
}
