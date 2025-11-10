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
    @State private var altGroup: String? = nil

    // Track if ANY field has been modified
    @State private var hasChanges = false

    @FocusState private var focusedField: Field?

    enum Field: Hashable {
        case sets, reps, repsMin, repsMax, rest, rpe, notes
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
                        saveChanges()
                    }
                    .buttonStyle(.borderedProminent)
                    .keyboardShortcut(.return, modifiers: [.command])
                }
                .padding(.bottom, 8)
            }

            // Always-editable fields
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
                    onChange: { hasChanges = true }
                )
            }
        }
        .onAppear {
            loadCurrentValues()
        }
    }

    private func saveChanges() {
        ErrorLogger.shared.info("Saving inline edit for segment \(segment.id)")

        var updatedDict = segment.segmentDict

        if updatedDict["type"] == nil {
            updatedDict["type"] = segment.type
        }

        updatedDict["sets"] = sets

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

        do {
            let jsonData = try JSONSerialization.data(withJSONObject: updatedDict, options: .sortedKeys)
            if let jsonString = String(data: jsonData, encoding: .utf8) {
                appState.pushUndo(plan.planJSON, label: "Edit Segment")
                try plan.updateSegment(jsonString, at: segment.index, inDayAt: segment.dayIndex)
                hasChanges = false
                ErrorLogger.shared.info("Successfully saved segment \(segment.id)")
            }
        } catch {
            ErrorLogger.shared.error("Failed to save segment \(segment.id): \(error.localizedDescription)")
        }
    }

    private func loadCurrentValues() {
        sets = segment.intValue("sets") ?? 3

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
        hasChanges = false
    }
}
