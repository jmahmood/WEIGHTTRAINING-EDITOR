import SwiftUI

/// Modeless inline editor for Scheme segments
struct InlineSchemeEditor: View {
    @ObservedObject var plan: PlanDocument
    let segment: SegmentDisplay
    @EnvironmentObject var appState: AppState

    @State private var schemeSets: [SchemeSetData] = []
    @State private var altGroup: String? = nil
    @State private var hasChanges = false

    struct SchemeSetData: Identifiable {
        let id = UUID()
        var label: String
        var sets: Int
        var reps: Int
        var repsMin: Int?
        var repsMax: Int?
        var isRange: Bool
        var rpe: Double?
        var restSec: Int?
        var note: String?
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            // Save/Revert bar
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

            // Exercise name (read-only for schemes)
            InspectorMetricRow(title: "Exercise", value: segment.primaryTitle(with: plan))

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

            Divider()

            // Editable sets
            if schemeSets.isEmpty {
                Text("No sets defined")
                    .font(.caption)
                    .foregroundColor(.secondary)
            } else {
                ForEach($schemeSets) { $set in
                    SchemeSetEditor(set: $set, onChange: { hasChanges = true })
                        .padding(.vertical, 8)
                    Divider()
                }
            }
        }
        .onAppear {
            loadCurrentValues()
        }
    }

    private func loadCurrentValues() {
        guard let sets = segment.segmentDict["sets"] as? [[String: Any]] else {
            schemeSets = []
            return
        }

        altGroup = segment.altGroupCode

        schemeSets = sets.enumerated().map { index, entry in
            let label = entry["label"] as? String ?? "Set \(index + 1)"
            let sets = entry["sets"] as? Int ?? 1
            let rpe = entry["rpe"] as? Double
            let restSec = entry["rest_sec"] as? Int
            let note = entry["note"] as? String

            var isRange = false
            var reps = 8
            var repsMin: Int? = nil
            var repsMax: Int? = nil

            if let repsValue = entry["reps"] {
                if let repsDict = repsValue as? [String: Any],
                   let min = repsDict["min"] as? Int,
                   let max = repsDict["max"] as? Int {
                    isRange = true
                    repsMin = min
                    repsMax = max
                } else if let singleReps = repsValue as? Int {
                    reps = singleReps
                }
            }

            return SchemeSetData(
                label: label,
                sets: sets,
                reps: reps,
                repsMin: repsMin,
                repsMax: repsMax,
                isRange: isRange,
                rpe: rpe,
                restSec: restSec,
                note: note
            )
        }

        hasChanges = false
    }

    private func saveChanges() {
        ErrorLogger.shared.info("Saving scheme edit for segment \(segment.id)")

        var updatedDict = segment.segmentDict

        // Build sets array
        let setsArray: [[String: Any]] = schemeSets.map { set in
            var setDict: [String: Any] = [
                "label": set.label,
                "sets": set.sets
            ]

            // Reps
            if set.isRange, let min = set.repsMin, let max = set.repsMax {
                setDict["reps"] = ["min": min, "max": max]
            } else {
                setDict["reps"] = set.reps
            }

            // Optional fields
            if let rpe = set.rpe {
                setDict["rpe"] = rpe
            }
            if let restSec = set.restSec {
                setDict["rest_sec"] = restSec
            }
            if let note = set.note, !note.isEmpty {
                setDict["note"] = note
            }

            return setDict
        }

        updatedDict["sets"] = setsArray

        if let altGroup = altGroup, !altGroup.isEmpty {
            updatedDict["alt_group"] = altGroup
        } else {
            updatedDict.removeValue(forKey: "alt_group")
        }

        do {
            let jsonData = try JSONSerialization.data(withJSONObject: updatedDict, options: .sortedKeys)
            if let jsonString = String(data: jsonData, encoding: .utf8) {
                appState.pushUndo(plan.planJSON, label: "Edit Scheme")
                try plan.updateSegment(jsonString, at: segment.index, inDayAt: segment.dayIndex)
                hasChanges = false
                ErrorLogger.shared.info("Successfully saved scheme \(segment.id)")
            }
        } catch {
            ErrorLogger.shared.error("Failed to save scheme \(segment.id): \(error.localizedDescription)")
        }
    }
}

struct SchemeSetEditor: View {
    @Binding var set: InlineSchemeEditor.SchemeSetData
    let onChange: () -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            // Label
            HStack {
                Text("Label:")
                    .frame(width: 60, alignment: .leading)
                TextField("", text: $set.label)
                    .textFieldStyle(.roundedBorder)
                    .onChange(of: set.label) { _ in onChange() }
            }

            // Sets
            HStack {
                Text("Sets:")
                    .frame(width: 60, alignment: .leading)
                TextField("", value: $set.sets, format: .number)
                    .textFieldStyle(.roundedBorder)
                    .frame(width: 60)
                    .onChange(of: set.sets) { _ in onChange() }
            }

            // Rep range toggle
            Toggle("Rep Range", isOn: $set.isRange)
                .onChange(of: set.isRange) { _ in onChange() }

            // Reps
            if set.isRange {
                HStack {
                    Text("Min:")
                        .frame(width: 60, alignment: .leading)
                    TextField("", value: Binding(
                        get: { set.repsMin ?? 8 },
                        set: { set.repsMin = $0 }
                    ), format: .number)
                    .textFieldStyle(.roundedBorder)
                    .frame(width: 60)
                    .onChange(of: set.repsMin) { _ in onChange() }

                    Text("Max:")
                        .frame(width: 60, alignment: .leading)
                    TextField("", value: Binding(
                        get: { set.repsMax ?? 12 },
                        set: { set.repsMax = $0 }
                    ), format: .number)
                    .textFieldStyle(.roundedBorder)
                    .frame(width: 60)
                    .onChange(of: set.repsMax) { _ in onChange() }
                }
            } else {
                HStack {
                    Text("Reps:")
                        .frame(width: 60, alignment: .leading)
                    TextField("", value: $set.reps, format: .number)
                        .textFieldStyle(.roundedBorder)
                        .frame(width: 60)
                        .onChange(of: set.reps) { _ in onChange() }
                }
            }

            // RPE (optional)
            HStack {
                Text("RPE:")
                    .frame(width: 60, alignment: .leading)
                if let rpe = set.rpe {
                    Slider(value: Binding(
                        get: { rpe },
                        set: { set.rpe = $0 }
                    ), in: 6...10, step: 0.5)
                    .onChange(of: set.rpe) { _ in onChange() }
                    Text(String(format: "%.1f", rpe))
                        .frame(width: 40)
                    Button("Clear") {
                        set.rpe = nil
                        onChange()
                    }
                    .buttonStyle(.borderless)
                } else {
                    Button("Add RPE") {
                        set.rpe = 8.0
                        onChange()
                    }
                    .buttonStyle(.borderless)
                }
            }

            // Rest (optional)
            HStack {
                Text("Rest:")
                    .frame(width: 60, alignment: .leading)
                if let restSec = set.restSec {
                    TextField("", value: Binding(
                        get: { restSec },
                        set: { set.restSec = $0 }
                    ), format: .number)
                    .textFieldStyle(.roundedBorder)
                    .frame(width: 80)
                    .onChange(of: set.restSec) { _ in onChange() }
                    Text("sec")
                    Button("Clear") {
                        set.restSec = nil
                        onChange()
                    }
                    .buttonStyle(.borderless)
                } else {
                    Button("Add Rest") {
                        set.restSec = 120
                        onChange()
                    }
                    .buttonStyle(.borderless)
                }
            }

            // Notes (optional)
            VStack(alignment: .leading, spacing: 4) {
                if let note = set.note {
                    Text("Notes:")
                        .font(.caption)
                    HStack {
                        TextEditor(text: Binding(
                            get: { note },
                            set: { set.note = $0 }
                        ))
                        .frame(height: 40)
                        .border(Color.gray.opacity(0.3))
                        .onChange(of: set.note) { _ in onChange() }

                        Button("Clear") {
                            set.note = nil
                            onChange()
                        }
                        .buttonStyle(.borderless)
                    }
                } else {
                    Button("Add Notes") {
                        set.note = ""
                        onChange()
                    }
                    .buttonStyle(.borderless)
                }
            }
        }
        .padding(8)
        .background(Color(NSColor.controlBackgroundColor))
        .cornerRadius(6)
    }
}
