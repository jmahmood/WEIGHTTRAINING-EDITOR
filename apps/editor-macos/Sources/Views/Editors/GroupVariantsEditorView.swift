import SwiftUI

struct GroupVariantsEditorView: View {
    @ObservedObject var plan: PlanDocument
    @Environment(\.dismiss) private var dismiss

    @State private var searchText = ""
    @State private var selectedExercise: String?
    @State private var draftDefaults: [String: [String: RoleRepsRange]] = [:]
    @State private var hasChanges = false

    private let roles: [(id: String, label: String)] = [
        ("strength", "Strength"),
        ("volume", "Volume"),
        ("endurance", "Endurance")
    ]

    init(plan: PlanDocument) {
        self.plan = plan
        let meta = plan.getExerciseMeta()
        var defaults: [String: [String: RoleRepsRange]] = [:]
        for (code, entry) in meta {
            defaults[code] = entry.roleReps
        }
        _draftDefaults = State(initialValue: defaults)
    }

    var body: some View {
        VStack(spacing: 0) {
            HStack {
                Text("Default Rep Ranges")
                    .font(.headline)
                Spacer()
                Button("Close") { dismiss() }
            }
            .padding()

            Divider()

            HSplitView {
                exerciseList
                    .frame(minWidth: 260)
                defaultsEditor
                    .frame(minWidth: 460)
            }
            .frame(maxWidth: .infinity, maxHeight: .infinity)

            Divider()

            HStack {
                Spacer()
                Button("Cancel") {
                    dismiss()
                }
                Button("Save") {
                    saveDefaults()
                    hasChanges = false
                    dismiss()
                }
                .buttonStyle(.borderedProminent)
                .disabled(!hasChanges)
            }
            .padding()
        }
        .frame(width: 900, height: 520)
        .onAppear {
            if selectedExercise == nil {
                selectedExercise = filteredExercises.first
            }
        }
    }

    private var exerciseList: some View {
        VStack(alignment: .leading, spacing: 8) {
            TextField("Search exercises...", text: $searchText)
                .textFieldStyle(.roundedBorder)

            List(filteredExercises, id: \.self, selection: $selectedExercise) { code in
                VStack(alignment: .leading, spacing: 2) {
                    Text(plan.dictionary[code] ?? code)
                    Text(code)
                        .font(.caption2)
                        .foregroundColor(.secondary)
                }
            }
        }
        .padding()
    }

    private var defaultsEditor: some View {
        VStack(alignment: .leading, spacing: 12) {
            if let exerciseCode = selectedExercise {
                Text(plan.dictionary[exerciseCode] ?? exerciseCode)
                    .font(.headline)
                Text(exerciseCode)
                    .font(.caption)
                    .foregroundColor(.secondary)

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
                            TextField("", value: bindingMin(exercise: exerciseCode, role: role.id), format: .number)
                                .textFieldStyle(.roundedBorder)
                                .frame(width: 64)
                            TextField("", value: bindingMax(exercise: exerciseCode, role: role.id), format: .number)
                                .textFieldStyle(.roundedBorder)
                                .frame(width: 64)
                        }
                    }
                }
            } else {
                Text("Select an exercise to edit defaults.")
                    .font(.caption)
                    .foregroundColor(.secondary)
            }
            Spacer()
        }
        .padding()
    }

    private var filteredExercises: [String] {
        let all = plan.dictionary.keys.sorted()
        guard !searchText.isEmpty else { return all }
        return all.filter { code in
            code.localizedCaseInsensitiveContains(searchText) ||
            (plan.dictionary[code]?.localizedCaseInsensitiveContains(searchText) ?? false)
        }
    }

    private func bindingMin(exercise: String, role: String) -> Binding<Int> {
        Binding(
            get: { draftDefaults[exercise]?[role]?.min ?? 0 },
            set: { newValue in
                var entry = draftDefaults[exercise] ?? [:]
                var range = entry[role] ?? RoleRepsRange(min: 0, max: 0)
                range.min = newValue
                entry[role] = range
                draftDefaults[exercise] = entry
                hasChanges = true
            }
        )
    }

    private func bindingMax(exercise: String, role: String) -> Binding<Int> {
        Binding(
            get: { draftDefaults[exercise]?[role]?.max ?? 0 },
            set: { newValue in
                var entry = draftDefaults[exercise] ?? [:]
                var range = entry[role] ?? RoleRepsRange(min: 0, max: 0)
                range.max = newValue
                entry[role] = range
                draftDefaults[exercise] = entry
                hasChanges = true
            }
        )
    }

    private func saveDefaults() {
        for (exercise, roleReps) in draftDefaults {
            let cleaned = roleReps.filter { $0.value.min > 0 && $0.value.max > 0 }
            if cleaned == plan.getRoleRepsDefaults(for: exercise) {
                continue
            }
            plan.updateExerciseRoleDefaults(for: exercise, roleReps: cleaned)
        }
    }
}
