import SwiftUI

/// Inline editable view for basic segment types (straight, rpe, percentage, amrap)
/// Implements modeless editing with keyboard navigation
struct InlineSegmentEditor: View {
    @ObservedObject var plan: PlanDocument
    let segment: SegmentDisplay
    @EnvironmentObject var appState: AppState

    private var isEditing: Bool {
        appState.inlineEditingSegmentId == segment.id
    }

    @State private var sets: Int = 3
    @State private var reps: Int = 8
    @State private var useRepsRange = false
    @State private var repsMin: Int = 8
    @State private var repsMax: Int = 12
    @State private var restSec: Int = 120
    @State private var rpe: Double = 8.0
    @State private var notes: String = ""

    @FocusState private var focusedField: Field?

    enum Field: Hashable {
        case sets, reps, repsMin, repsMax, rest, rpe, notes
    }

    var body: some View {
        let _ = NSLog("InlineSegmentEditor.body - isEditing: %d, segmentId: %@", isEditing, segment.id)

        return VStack(alignment: .leading, spacing: 16) {
            HStack {
                Text("Edit Mode")
                    .font(.caption)
                    .foregroundColor(.secondary)
                Spacer()
                Button(action: {
                    NSLog("*** BUTTON ACTION TRIGGERED *** isEditing = %d", isEditing)
                    if isEditing {
                        NSLog("*** Calling saveChanges() ***")
                        saveChanges()
                    } else {
                        NSLog("*** Calling startEditing() ***")
                        startEditing()
                    }
                }) {
                    Text(isEditing ? "Save" : "Edit")
                }
                .buttonStyle(.borderedProminent)

                if isEditing {
                    Button("Cancel") {
                        cancelEditing()
                    }
                    .keyboardShortcut(.escape, modifiers: [])
                }
            }

            if isEditing {
                editableFields
            } else {
                readOnlyFields
            }
        }
        .onAppear {
            loadCurrentValues()
        }
    }

    @ViewBuilder
    private var editableFields: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Sets & Reps")
                .font(.headline)

            VStack(alignment: .leading, spacing: 8) {
                HStack {
                    Text("Sets:")
                        .frame(width: 80, alignment: .leading)
                    TextField("", value: $sets, format: .number)
                        .textFieldStyle(.roundedBorder)
                        .focused($focusedField, equals: .sets)
                        .onSubmit { advanceToNext(from: .sets) }
                }

                Toggle("Use Rep Range", isOn: $useRepsRange)

                if useRepsRange {
                    HStack {
                        Text("Min Reps:")
                            .frame(width: 80, alignment: .leading)
                        TextField("", value: $repsMin, format: .number)
                            .textFieldStyle(.roundedBorder)
                            .focused($focusedField, equals: .repsMin)
                            .onSubmit { advanceToNext(from: .repsMin) }
                    }
                    HStack {
                        Text("Max Reps:")
                            .frame(width: 80, alignment: .leading)
                        TextField("", value: $repsMax, format: .number)
                            .textFieldStyle(.roundedBorder)
                            .focused($focusedField, equals: .repsMax)
                            .onSubmit { advanceToNext(from: .repsMax) }
                    }
                } else {
                    HStack {
                        Text("Reps:")
                            .frame(width: 80, alignment: .leading)
                        TextField("", value: $reps, format: .number)
                            .textFieldStyle(.roundedBorder)
                            .focused($focusedField, equals: .reps)
                            .onSubmit { advanceToNext(from: .reps) }
                    }
                }
            }

            Divider()

            Text("Rest & Effort")
                .font(.headline)

            VStack(alignment: .leading, spacing: 8) {
                HStack {
                    Text("Rest (sec):")
                        .frame(width: 80, alignment: .leading)
                    TextField("", value: $restSec, format: .number)
                        .textFieldStyle(.roundedBorder)
                        .focused($focusedField, equals: .rest)
                        .onSubmit { advanceToNext(from: .rest) }
                }

                HStack {
                    Text("RPE:")
                        .frame(width: 80, alignment: .leading)
                    Slider(value: $rpe, in: 6...10, step: 0.5)
                    Text(String(format: "%.1f", rpe))
                        .frame(width: 40, alignment: .trailing)
                }
                .focused($focusedField, equals: .rpe)
            }

            Divider()

            Text("Notes")
                .font(.headline)

            TextEditor(text: $notes)
                .frame(height: 60)
                .focused($focusedField, equals: .notes)
                .border(Color.gray.opacity(0.3))
        }
        .padding()
    }

    @ViewBuilder
    private var readOnlyFields: some View {
        VStack(alignment: .leading, spacing: 10) {
            InspectorMetricRow(title: "Sets × Reps", value: repsDisplay)
            InspectorMetricRow(title: "Rest", value: "\(restSec)s")
            InspectorMetricRow(title: "RPE", value: String(format: "%.1f", rpe))
            if !notes.isEmpty {
                InspectorMetricRow(title: "Notes", value: notes)
            }
        }
    }

    private var repsDisplay: String {
        if useRepsRange {
            return "\(sets) × \(repsMin)-\(repsMax)"
        } else {
            return "\(sets) × \(reps)"
        }
    }

    private func startEditing() {
        NSLog("*** startEditing() for segment %@", segment.id)
        loadCurrentValues()
        appState.inlineEditingSegmentId = segment.id
        focusedField = .sets
    }

    private func cancelEditing() {
        NSLog("*** cancelEditing() for segment %@", segment.id)
        appState.inlineEditingSegmentId = nil
        focusedField = nil
        loadCurrentValues() // Reset to original values
    }

    private func saveChanges() {
        NSLog("=== InlineSegmentEditor.saveChanges() called ===")
        NSLog("Segment type: %@, dayIndex: %d, index: %d", segment.type, segment.dayIndex, segment.index)
        NSLog("Current values - sets: %d, reps: %d, rest: %d, rpe: %.1f", sets, reps, restSec, rpe)

        // Build updated segment JSON - preserve all existing fields
        var updatedDict = segment.segmentDict

        // Ensure type is preserved
        if updatedDict["type"] == nil {
            updatedDict["type"] = segment.type
        }

        // Update editable fields
        updatedDict["sets"] = sets

        // Reps field - must match Rust's RepsOrRange enum format
        if useRepsRange {
            // Range format: {"min": X, "max": Y}
            updatedDict["reps"] = ["min": repsMin, "max": repsMax]
            updatedDict.removeValue(forKey: "reps_min")
            updatedDict.removeValue(forKey: "reps_max")
        } else {
            // Single value format: just the number
            updatedDict["reps"] = reps
            updatedDict.removeValue(forKey: "reps_min")
            updatedDict.removeValue(forKey: "reps_max")
        }

        updatedDict["rest_sec"] = restSec
        updatedDict["rpe"] = rpe

        if !notes.isEmpty {
            updatedDict["note"] = notes
        } else {
            updatedDict.removeValue(forKey: "note")
        }

        // Convert to JSON and save
        do {
            let jsonData = try JSONSerialization.data(withJSONObject: updatedDict, options: .sortedKeys)
            if let jsonString = String(data: jsonData, encoding: .utf8) {
                NSLog("Generated JSON: %@", jsonString)
                appState.pushUndo(plan.planJSON, label: "Edit Segment")
                NSLog("Calling plan.updateSegment...")
                try plan.updateSegment(jsonString, at: segment.index, inDayAt: segment.dayIndex)
                NSLog("Update successful!")
                appState.inlineEditingSegmentId = nil
                focusedField = nil
            }
        } catch {
            let errorMsg = "Failed to save inline edit for segment \(segment.id): \(error.localizedDescription)"
            ErrorLogger.shared.error(errorMsg)
            NSLog("ERROR: %@", errorMsg)
        }
    }

    private func loadCurrentValues() {
        sets = segment.intValue("sets") ?? 3

        // Check if reps is a range (dict with min/max) or a single value
        if let repsValue = segment.segmentDict["reps"] {
            if let repsDict = repsValue as? [String: Any],
               let min = repsDict["min"] as? Int,
               let max = repsDict["max"] as? Int {
                // It's a range
                useRepsRange = true
                repsMin = min
                repsMax = max
                reps = 8 // Default for when switching to single value
            } else if let singleReps = repsValue as? Int {
                // It's a single value
                useRepsRange = false
                reps = singleReps
                repsMin = 8  // Defaults for when switching to range
                repsMax = 12
            } else {
                // Fallback
                useRepsRange = false
                reps = 8
                repsMin = 8
                repsMax = 12
            }
        } else {
            // No reps field, use defaults
            useRepsRange = false
            reps = 8
            repsMin = 8
            repsMax = 12
        }

        restSec = segment.restSeconds ?? 120
        rpe = segment.rpeValue ?? 8.0
        notes = segment.inspectorNote ?? ""
    }

    private func advanceToNext(from current: Field) {
        switch current {
        case .sets:
            focusedField = useRepsRange ? .repsMin : .reps
        case .reps:
            focusedField = .rest
        case .repsMin:
            focusedField = .repsMax
        case .repsMax:
            focusedField = .rest
        case .rest:
            focusedField = .rpe
        case .rpe:
            focusedField = .notes
        case .notes:
            saveChanges()
        }
    }
}
