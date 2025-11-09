import SwiftUI

struct SupersetItemEditorView: View {
    @ObservedObject var plan: PlanDocument
    let item: [String: Any]?
    let onSave: ([String: Any]) -> Void

    @Environment(\.dismiss) private var dismiss

    @State private var exerciseCode = ""
    @State private var exerciseName = ""
    @State private var sets = 1
    @State private var useRepsRange = true
    @State private var reps = 8
    @State private var repsMin = 8
    @State private var repsMax = 12
    @State private var rpe: Double = 8.0
    @State private var useRPE = false
    @State private var restSec = 60
    @State private var altGroup: String?

    var body: some View {
        VStack(spacing: 20) {
            Text(item == nil ? "Add Exercise to Superset" : "Edit Exercise")
                .font(.title2)
                .fontWeight(.bold)

            Form {
                Section("Exercise") {
                    ExercisePicker(
                        plan: plan,
                        selectedExerciseCode: $exerciseCode,
                        selectedExerciseName: $exerciseName
                    )
                }

                Section("Alternative Group (Optional)") {
                    GroupPicker(plan: plan, selectedGroup: $altGroup)
                }

                Section("Sets & Reps") {
                    Stepper("Sets: \(sets)", value: $sets, in: 1...20)

                    Toggle("Use Rep Range", isOn: $useRepsRange)

                    if useRepsRange {
                        Stepper("Min: \(repsMin)", value: $repsMin, in: 1...100)
                        Stepper("Max: \(repsMax)", value: $repsMax, in: 1...100)
                    } else {
                        Stepper("Reps: \(reps)", value: $reps, in: 1...100)
                    }
                }

                Section("Intensity") {
                    Toggle("Set RPE", isOn: $useRPE)

                    if useRPE {
                        HStack {
                            Text("RPE:")
                            Slider(value: $rpe, in: 6.0...10.0, step: 0.5)
                            Text(String(format: "%.1f", rpe))
                                .frame(width: 40)
                        }
                    }
                }

                Section("Rest") {
                    Stepper("Rest Seconds: \(restSec)", value: $restSec, in: 0...180, step: 15)
                }
            }

            Spacer()

            HStack {
                Button("Cancel") {
                    dismiss()
                }
                .keyboardShortcut(.escape)

                Spacer()

                Button("Save") {
                    saveItem()
                }
                .keyboardShortcut(.return)
                .buttonStyle(.borderedProminent)
            }
        }
        .padding()
        .frame(width: 500, height: 500)
        .onAppear {
            loadItem()
        }
    }

    private func loadItem() {
        guard let item = item else { return }

        exerciseCode = item["ex"] as? String ?? ""
        sets = item["sets"] as? Int ?? 1
        altGroup = item["alt_group"] as? String

        // Load exercise name from dictionary
        if !exerciseCode.isEmpty {
            exerciseName = plan.dictionary[exerciseCode] ?? exerciseCode
        }

        // Load reps
        if let repsObj = item["reps"] as? [String: Any],
           let min = repsObj["min"] as? Int,
           let max = repsObj["max"] as? Int {
            useRepsRange = true
            repsMin = min
            repsMax = max
        } else if let repsValue = item["reps"] as? Int {
            useRepsRange = false
            reps = repsValue
        }

        // Load RPE
        if let rpeValue = item["rpe"] as? Double {
            useRPE = true
            rpe = rpeValue
        }
        if let restValue = item["rest_sec"] as? Int {
            restSec = restValue
        }
    }

    private func saveItem() {
        var itemDict: [String: Any] = [
            "ex": exerciseCode,
            "sets": sets
        ]

        // Save reps
        if useRepsRange {
            itemDict["reps"] = [
                "min": repsMin,
                "max": repsMax
            ]
        } else {
            itemDict["reps"] = reps
        }

        // Save RPE
        if useRPE {
            itemDict["rpe"] = rpe
        }

        itemDict["rest_sec"] = restSec

        // Save alt_group
        if let altGroup = altGroup {
            itemDict["alt_group"] = altGroup
        }

        onSave(itemDict)
        dismiss()
    }
}
