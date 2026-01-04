import SwiftUI

struct ExerciseRoleDefaultsEditorView: View {
    @ObservedObject var plan: PlanDocument
    let exerciseCode: String
    @Environment(\.dismiss) private var dismiss

    @State private var roleReps: [String: RoleRepsRange] = [:]
    @State private var hasChanges = false

    private let roles: [(id: String, label: String)] = [
        ("strength", "Strength"),
        ("volume", "Volume"),
        ("endurance", "Endurance")
    ]

    init(plan: PlanDocument, exerciseCode: String) {
        self.plan = plan
        self.exerciseCode = exerciseCode
        _roleReps = State(initialValue: plan.getRoleRepsDefaults(for: exerciseCode))
    }

    var body: some View {
        VStack(spacing: 16) {
            HStack {
                Text("Exercise Defaults")
                    .font(.title2)
                    .fontWeight(.bold)
                Spacer()
                Button("Close") { dismiss() }
                    .keyboardShortcut(.cancelAction)
            }

            VStack(alignment: .leading, spacing: 4) {
                Text(plan.dictionary[exerciseCode] ?? exerciseCode)
                    .font(.headline)
                Text(exerciseCode)
                    .font(.caption)
                    .foregroundColor(.secondary)
            }

            Divider()

            Grid(alignment: .leading, horizontalSpacing: 12, verticalSpacing: 10) {
                GridRow {
                    Text("Role")
                        .font(.caption)
                    Text("Min")
                        .font(.caption)
                    Text("Max")
                        .font(.caption)
                }
                .foregroundColor(.secondary)

                ForEach(roles, id: \.id) { role in
                    GridRow {
                        Text(role.label)
                        TextField("", value: bindingMin(for: role.id), format: .number)
                            .textFieldStyle(.roundedBorder)
                            .frame(width: 60)
                        TextField("", value: bindingMax(for: role.id), format: .number)
                            .textFieldStyle(.roundedBorder)
                            .frame(width: 60)
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
                    save()
                    dismiss()
                }
                .buttonStyle(.borderedProminent)
                .disabled(!hasChanges)
            }
        }
        .padding()
        .frame(width: 420, height: 360)
    }

    private func bindingMin(for role: String) -> Binding<Int> {
        Binding(
            get: { roleReps[role]?.min ?? 0 },
            set: { newValue in
                var entry = roleReps[role] ?? RoleRepsRange(min: 0, max: 0)
                entry.min = newValue
                roleReps[role] = entry
                hasChanges = true
            }
        )
    }

    private func bindingMax(for role: String) -> Binding<Int> {
        Binding(
            get: { roleReps[role]?.max ?? 0 },
            set: { newValue in
                var entry = roleReps[role] ?? RoleRepsRange(min: 0, max: 0)
                entry.max = newValue
                roleReps[role] = entry
                hasChanges = true
            }
        )
    }

    private func save() {
        let cleaned = roleReps.filter { $0.value.min > 0 && $0.value.max > 0 }
        plan.updateExerciseRoleDefaults(for: exerciseCode, roleReps: cleaned)
    }
}
