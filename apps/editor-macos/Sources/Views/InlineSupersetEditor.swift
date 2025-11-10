import SwiftUI

/// Modeless inline editor for Superset and Circuit segments
struct InlineSupersetEditor: View {
    @ObservedObject var plan: PlanDocument
    let segment: SegmentDisplay
    @EnvironmentObject var appState: AppState

    @State private var label: String = ""
    @State private var rounds: Int = 3
    @State private var restBetweenRoundsSec: Int = 120
    @State private var pairing: String = "sequential"
    @State private var exercises: [ExerciseItem] = []
    @State private var hasChanges = false

    struct ExerciseItem: Identifiable {
        let id = UUID()
        var exerciseCode: String
        var sets: Int
        var reps: Int
        var repsMin: Int?
        var repsMax: Int?
        var isRange: Bool
        var restSec: Int?
        var rpe: Double?
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

            // Label
            VStack(alignment: .leading, spacing: 4) {
                Text("Label (Optional):")
                    .font(.caption)
                TextField("", text: $label)
                    .textFieldStyle(.roundedBorder)
                    .onChange(of: label) { _ in hasChanges = true }
            }

            Divider()

            // Rounds
            HStack {
                Text("Rounds:")
                    .frame(width: 100, alignment: .leading)
                TextField("", value: $rounds, format: .number)
                    .textFieldStyle(.roundedBorder)
                    .frame(width: 60)
                    .onChange(of: rounds) { _ in hasChanges = true }
            }

            // Rest between rounds
            HStack {
                Text("Rest between:")
                    .frame(width: 100, alignment: .leading)
                TextField("", value: $restBetweenRoundsSec, format: .number)
                    .textFieldStyle(.roundedBorder)
                    .frame(width: 80)
                    .onChange(of: restBetweenRoundsSec) { _ in hasChanges = true }
                Text("sec")
            }

            // Pairing (only for supersets)
            if segment.type == "superset" {
                VStack(alignment: .leading, spacing: 4) {
                    Text("Pairing:")
                        .font(.caption)
                    Picker("", selection: $pairing) {
                        Text("Sequential").tag("sequential")
                        Text("Alternating").tag("alternating")
                    }
                    .pickerStyle(.segmented)
                    .onChange(of: pairing) { _ in hasChanges = true }
                }
            }

            Divider()

            // Exercise List
            VStack(alignment: .leading, spacing: 8) {
                HStack {
                    Text("Exercises")
                        .font(.headline)
                    Spacer()
                    Button {
                        addExercise()
                    } label: {
                        Label("Add", systemImage: "plus.circle")
                    }
                    .buttonStyle(.borderless)
                }

                if exercises.isEmpty {
                    Text("No exercises defined")
                        .font(.caption)
                        .foregroundColor(.secondary)
                } else {
                    ForEach($exercises) { $exercise in
                        ExerciseEditor(
                            exercise: $exercise,
                            plan: plan,
                            onChange: { hasChanges = true },
                            onDelete: {
                                deleteExercise(exercise)
                            }
                        )
                        .padding(.vertical, 4)
                    }
                }
            }
        }
        .onAppear {
            loadCurrentValues()
        }
        .onChange(of: appState.shouldFocusInspector) { shouldFocus in
            if shouldFocus {
                // Consume the flag - superset editor has complex nested focus
                appState.shouldFocusInspector = false
            }
        }
    }

    private func addExercise() {
        let newExercise = ExerciseItem(
            exerciseCode: "",
            sets: 3,
            reps: 8,
            repsMin: nil,
            repsMax: nil,
            isRange: false,
            restSec: nil,
            rpe: nil,
            note: nil
        )
        exercises.append(newExercise)
        hasChanges = true
    }

    private func deleteExercise(_ exercise: ExerciseItem) {
        exercises.removeAll { $0.id == exercise.id }
        hasChanges = true
    }

    private func loadCurrentValues() {
        label = segment.segmentDict["label"] as? String ?? ""
        rounds = segment.intValue("rounds") ?? 3
        restBetweenRoundsSec = segment.intValue("rest_between_rounds_sec") ?? 120
        pairing = segment.segmentDict["pairing"] as? String ?? "sequential"

        // Load exercises from "items" or "exercises" array
        let exerciseArray = (segment.segmentDict["items"] as? [[String: Any]]) ??
                           (segment.segmentDict["exercises"] as? [[String: Any]]) ?? []

        exercises = exerciseArray.map { entry in
            let exCode = entry["ex"] as? String ?? ""
            let sets = entry["sets"] as? Int ?? 3
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

            return ExerciseItem(
                exerciseCode: exCode,
                sets: sets,
                reps: reps,
                repsMin: repsMin,
                repsMax: repsMax,
                isRange: isRange,
                restSec: restSec,
                rpe: rpe,
                note: note
            )
        }

        hasChanges = false
    }

    private func saveChanges() {
        ErrorLogger.shared.info("Saving superset/circuit edit for segment \(segment.id)")

        var updatedDict = segment.segmentDict

        // Update metadata
        if !label.isEmpty {
            updatedDict["label"] = label
        } else {
            updatedDict.removeValue(forKey: "label")
        }

        updatedDict["rounds"] = rounds
        updatedDict["rest_between_rounds_sec"] = restBetweenRoundsSec

        if segment.type == "superset" {
            updatedDict["pairing"] = pairing
        }

        // Build exercises/items array
        let exercisesArray: [[String: Any]] = exercises.map { ex in
            var exDict: [String: Any] = [
                "ex": ex.exerciseCode,
                "sets": ex.sets
            ]

            // Reps
            if ex.isRange, let min = ex.repsMin, let max = ex.repsMax {
                exDict["reps"] = ["min": min, "max": max]
            } else {
                exDict["reps"] = ex.reps
            }

            // Optional fields
            if let rpe = ex.rpe {
                exDict["rpe"] = rpe
            }
            if let restSec = ex.restSec {
                exDict["rest_sec"] = restSec
            }
            if let note = ex.note, !note.isEmpty {
                exDict["note"] = note
            }

            return exDict
        }

        // Use "items" for both superset and circuit (modern format)
        updatedDict["items"] = exercisesArray
        // Remove old "exercises" key if it exists
        updatedDict.removeValue(forKey: "exercises")

        do {
            let jsonData = try JSONSerialization.data(withJSONObject: updatedDict, options: .sortedKeys)
            if let jsonString = String(data: jsonData, encoding: .utf8) {
                appState.pushUndo(plan.planJSON, label: "Edit \(segment.humanReadableType)")
                try plan.updateSegment(jsonString, at: segment.index, inDayAt: segment.dayIndex)
                hasChanges = false
                ErrorLogger.shared.info("Successfully saved \(segment.type) \(segment.id)")
            }
        } catch {
            ErrorLogger.shared.error("Failed to save \(segment.type) \(segment.id): \(error.localizedDescription)")
        }
    }
}

struct ExerciseEditor: View {
    @Binding var exercise: InlineSupersetEditor.ExerciseItem
    @ObservedObject var plan: PlanDocument
    let onChange: () -> Void
    let onDelete: () -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack {
                Text("Exercise:")
                    .font(.caption)
                    .frame(width: 80, alignment: .leading)

                Picker("", selection: $exercise.exerciseCode) {
                    Text("Select...").tag("")
                    ForEach(plan.dictionary.keys.sorted(), id: \.self) { code in
                        Text(plan.dictionary[code] ?? code).tag(code)
                    }
                }
                .onChange(of: exercise.exerciseCode) { _ in onChange() }

                Button(role: .destructive) {
                    onDelete()
                } label: {
                    Image(systemName: "trash")
                }
                .buttonStyle(.borderless)
            }

            if !exercise.exerciseCode.isEmpty {
                // Sets
                HStack {
                    Text("Sets:")
                        .frame(width: 80, alignment: .leading)
                    TextField("", value: $exercise.sets, format: .number)
                        .textFieldStyle(.roundedBorder)
                        .frame(width: 60)
                        .onChange(of: exercise.sets) { _ in onChange() }
                }

                // Rep range toggle
                Toggle("Rep Range", isOn: $exercise.isRange)
                    .onChange(of: exercise.isRange) { _ in onChange() }

                // Reps
                if exercise.isRange {
                    HStack {
                        Text("Min:")
                            .frame(width: 80, alignment: .leading)
                        TextField("", value: Binding(
                            get: { exercise.repsMin ?? 8 },
                            set: { exercise.repsMin = $0 }
                        ), format: .number)
                        .textFieldStyle(.roundedBorder)
                        .frame(width: 60)
                        .onChange(of: exercise.repsMin) { _ in onChange() }

                        Text("Max:")
                            .frame(width: 60, alignment: .leading)
                        TextField("", value: Binding(
                            get: { exercise.repsMax ?? 12 },
                            set: { exercise.repsMax = $0 }
                        ), format: .number)
                        .textFieldStyle(.roundedBorder)
                        .frame(width: 60)
                        .onChange(of: exercise.repsMax) { _ in onChange() }
                    }
                } else {
                    HStack {
                        Text("Reps:")
                            .frame(width: 80, alignment: .leading)
                        TextField("", value: $exercise.reps, format: .number)
                            .textFieldStyle(.roundedBorder)
                            .frame(width: 60)
                            .onChange(of: exercise.reps) { _ in onChange() }
                    }
                }

                // RPE (optional)
                HStack {
                    Text("RPE:")
                        .frame(width: 80, alignment: .leading)
                    if let rpe = exercise.rpe {
                        Slider(value: Binding(
                            get: { rpe },
                            set: { exercise.rpe = $0 }
                        ), in: 6...10, step: 0.5)
                        .onChange(of: exercise.rpe) { _ in onChange() }
                        Text(String(format: "%.1f", rpe))
                            .frame(width: 40)
                        Button("Clear") {
                            exercise.rpe = nil
                            onChange()
                        }
                        .buttonStyle(.borderless)
                    } else {
                        Button("Add RPE") {
                            exercise.rpe = 8.0
                            onChange()
                        }
                        .buttonStyle(.borderless)
                    }
                }

                // Rest (optional)
                HStack {
                    Text("Rest:")
                        .frame(width: 80, alignment: .leading)
                    if let restSec = exercise.restSec {
                        TextField("", value: Binding(
                            get: { restSec },
                            set: { exercise.restSec = $0 }
                        ), format: .number)
                        .textFieldStyle(.roundedBorder)
                        .frame(width: 80)
                        .onChange(of: exercise.restSec) { _ in onChange() }
                        Text("sec")
                        Button("Clear") {
                            exercise.restSec = nil
                            onChange()
                        }
                        .buttonStyle(.borderless)
                    } else {
                        Button("Add Rest") {
                            exercise.restSec = 0
                            onChange()
                        }
                        .buttonStyle(.borderless)
                    }
                }

                // Notes (optional)
                VStack(alignment: .leading, spacing: 4) {
                    if let note = exercise.note {
                        Text("Notes:")
                            .font(.caption)
                        HStack {
                            TextEditor(text: Binding(
                                get: { note },
                                set: { exercise.note = $0 }
                            ))
                            .frame(height: 40)
                            .border(Color.gray.opacity(0.3))
                            .onChange(of: exercise.note) { _ in onChange() }

                            Button("Clear") {
                                exercise.note = nil
                                onChange()
                            }
                            .buttonStyle(.borderless)
                        }
                    } else {
                        Button("Add Notes") {
                            exercise.note = ""
                            onChange()
                        }
                        .buttonStyle(.borderless)
                    }
                }
            }
        }
        .padding(8)
        .background(Color(NSColor.controlBackgroundColor))
        .cornerRadius(6)
    }
}
