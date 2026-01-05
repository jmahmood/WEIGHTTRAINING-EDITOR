import SwiftUI

struct PlanPropertiesEditorView: View {
    @ObservedObject var plan: PlanDocument
    @EnvironmentObject var appState: AppState
    @Environment(\.dismiss) private var dismiss

    @State private var planName = ""
    @State private var author = ""
    @State private var unit = "kg"
    @State private var progressionMode = ""
    @State private var progressionRepsFirst = false
    @State private var progressionLoadIncrement = ""
    @State private var progressionCapRpe = ""

    var body: some View {
        VStack(spacing: 16) {
            Text("Edit Plan Details")
                .font(.title2)
                .fontWeight(.bold)

            Form {
                Section("Plan") {
                    TextField("Plan Name", text: $planName)
                    TextField("Author", text: $author)
                    Picker("Unit", selection: $unit) {
                        Text("kg").tag("kg")
                        Text("lb").tag("lb")
                        Text("bw").tag("bw")
                    }
                    .pickerStyle(.segmented)
                }

                Section("Progression") {
                    TextField("Mode (e.g., double_progression)", text: $progressionMode)
                    Toggle("Reps First", isOn: $progressionRepsFirst)
                    TextField("Load Increment (kg)", text: $progressionLoadIncrement)
                    TextField("Cap RPE", text: $progressionCapRpe)
                }
            }

            HStack {
                Button("Cancel") {
                    dismiss()
                }
                .keyboardShortcut(.escape)

                Spacer()

                Button("Save") {
                    saveChanges()
                }
                .keyboardShortcut(.return)
                .buttonStyle(.borderedProminent)
                .disabled(planName.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)
            }
        }
        .padding()
        .frame(width: 520, height: 420)
        .onAppear {
            loadCurrentValues()
        }
    }

    private func loadCurrentValues() {
        planName = plan.name
        author = plan.author ?? ""
        unit = plan.unit
        if let progression = plan.getProgressionSettings() {
            progressionMode = progression.mode
            progressionRepsFirst = progression.repsFirst
            progressionLoadIncrement = progression.loadIncrementKg.map { String($0) } ?? ""
            progressionCapRpe = progression.capRpe.map { String($0) } ?? ""
        } else {
            progressionMode = ""
            progressionRepsFirst = false
            progressionLoadIncrement = ""
            progressionCapRpe = ""
        }
    }

    private func saveChanges() {
        let trimmedMode = progressionMode.trimmingCharacters(in: .whitespacesAndNewlines)
        let loadIncrement = Double(progressionLoadIncrement.trimmingCharacters(in: .whitespacesAndNewlines))
        let capRpe = Double(progressionCapRpe.trimmingCharacters(in: .whitespacesAndNewlines))

        let progression = trimmedMode.isEmpty ? nil : ProgressionSettings(
            mode: trimmedMode,
            repsFirst: progressionRepsFirst,
            loadIncrementKg: loadIncrement,
            capRpe: capRpe
        )

        appState.pushUndo(plan.planJSON, label: "Edit Plan")
        plan.updatePlanMetadata(
            name: planName,
            author: author,
            unit: unit,
            progression: progression
        )
        dismiss()
    }
}
