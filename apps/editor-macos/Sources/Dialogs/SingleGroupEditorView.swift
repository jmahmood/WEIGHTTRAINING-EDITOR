import SwiftUI

struct SingleGroupEditorView: View {
    @ObservedObject var plan: PlanDocument
    let groupName: String
    let exercises: [String]
    let onSave: ([String]) -> Void
    let onDelete: () -> Void

    @Environment(\.dismiss) private var dismiss
    @State private var editableExercises: [String]
    @State private var newExercise = ""
    @State private var newExerciseName = ""

    init(plan: PlanDocument, groupName: String, exercises: [String], onSave: @escaping ([String]) -> Void, onDelete: @escaping () -> Void) {
        self.plan = plan
        self.groupName = groupName
        self.exercises = exercises
        self.onSave = onSave
        self.onDelete = onDelete
        _editableExercises = State(initialValue: exercises)
    }

    var body: some View {
        VStack(spacing: 20) {
            HStack {
                Text("Edit Group: \(groupName)")
                    .font(.title2)
                    .fontWeight(.bold)

                Spacer()

                Button("Delete Group", role: .destructive) {
                    onDelete()
                    dismiss()
                }
                .buttonStyle(.borderless)
            }

            Text("Exercises in this group:")
                .font(.subheadline)
                .foregroundColor(.secondary)
                .frame(maxWidth: .infinity, alignment: .leading)

            ScrollView {
                VStack(spacing: 4) {
                    ForEach(editableExercises.indices, id: \.self) { index in
                        HStack {
                            VStack(alignment: .leading, spacing: 2) {
                                if let displayName = plan.dictionary[editableExercises[index]] {
                                    Text(displayName)
                                        .font(.body)
                                    Text(editableExercises[index])
                                        .font(.caption)
                                        .foregroundColor(.secondary)
                                } else {
                                    Text(editableExercises[index])
                                        .font(.body)
                                }
                            }

                            Spacer()

                            HStack(spacing: 4) {
                                Button(action: { moveUp(index) }) {
                                    Image(systemName: "arrow.up")
                                }
                                .buttonStyle(.borderless)
                                .disabled(index == 0)

                                Button(action: { moveDown(index) }) {
                                    Image(systemName: "arrow.down")
                                }
                                .buttonStyle(.borderless)
                                .disabled(index == editableExercises.count - 1)

                                Button(action: { editableExercises.remove(at: index) }) {
                                    Image(systemName: "trash")
                                        .foregroundColor(.red)
                                }
                                .buttonStyle(.borderless)
                            }
                        }
                        .padding(.vertical, 4)
                        .padding(.horizontal, 8)
                        .background(Color(NSColor.controlBackgroundColor))
                        .cornerRadius(4)
                    }
                }
                .padding(.horizontal, 4)
            }
            .frame(height: 200)

            VStack(alignment: .leading, spacing: 8) {
                Text("Add Exercise:")
                    .font(.subheadline)
                    .foregroundColor(.secondary)

                ExercisePicker(
                    plan: plan,
                    selectedExerciseCode: $newExercise,
                    selectedExerciseName: $newExerciseName
                )
                .onChange(of: newExercise) { code in
                    if !code.isEmpty && !newExerciseName.isEmpty {
                        // Exercise was selected
                        editableExercises.append(code)
                        newExercise = ""
                        newExerciseName = ""
                    }
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
                    onSave(editableExercises)
                    dismiss()
                }
                .keyboardShortcut(.return)
                .buttonStyle(.borderedProminent)
            }
        }
        .padding()
        .frame(width: 600, height: 700)
    }

    private func addManualExercise() {
        guard !newExercise.trimmingCharacters(in: .whitespaces).isEmpty else { return }
        editableExercises.append(newExercise.trimmingCharacters(in: .whitespaces))
        newExercise = ""
    }

    private func moveUp(_ index: Int) {
        guard index > 0 else { return }
        editableExercises.swapAt(index, index - 1)
    }

    private func moveDown(_ index: Int) {
        guard index < editableExercises.count - 1 else { return }
        editableExercises.swapAt(index, index + 1)
    }
}
