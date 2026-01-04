import SwiftUI

struct SingleGroupEditorView: View {
    @ObservedObject var plan: PlanDocument
    let groupName: String
    let exercises: [String]
    let onSave: ([String], [String: [String: [String: JSONValue]]]) -> Void
    let onDelete: () -> Void

    @Environment(\.dismiss) private var dismiss
    @State private var editableExercises: [String]
    @State private var newExercise = ""
    @State private var newExerciseName = ""
    @State private var groupVariants: [String: [String: [String: JSONValue]]] = [:]

    private let roles: [(id: String, label: String)] = [
        ("strength", "Strength"),
        ("volume", "Volume"),
        ("endurance", "Endurance")
    ]

    init(plan: PlanDocument, groupName: String, exercises: [String], onSave: @escaping ([String], [String: [String: [String: JSONValue]]]) -> Void, onDelete: @escaping () -> Void) {
        self.plan = plan
        self.groupName = groupName
        self.exercises = exercises
        self.onSave = onSave
        self.onDelete = onDelete
        _editableExercises = State(initialValue: exercises)
        _groupVariants = State(initialValue: plan.getGroupVariants()[groupName] ?? [:])
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

            VStack(alignment: .leading, spacing: 12) {
                Text("Group Rep Ranges")
                    .font(.headline)

                VStack(alignment: .leading, spacing: 8) {
                    Text("Add Exercise")
                        .font(.caption)
                        .foregroundColor(.secondary)
                    ExercisePicker(
                        plan: plan,
                        selectedExerciseCode: $newExercise,
                        selectedExerciseName: $newExerciseName
                    )
                    .onChange(of: newExercise) { code in
                        if !code.isEmpty && !newExerciseName.isEmpty {
                            addExercise(code)
                            newExercise = ""
                            newExerciseName = ""
                        }
                    }
                }

                ScrollView {
                    Grid(alignment: .leading, horizontalSpacing: 12, verticalSpacing: 10) {
                        GridRow {
                            Text("Exercise")
                                .font(.caption)
                            ForEach(roles, id: \.id) { role in
                                Text(role.label)
                                    .font(.caption)
                            }
                        }
                        .foregroundColor(.secondary)

                        ForEach(editableExercises.indices, id: \.self) { index in
                            let exerciseCode = editableExercises[index]
                            GridRow {
                                exerciseCell(exerciseCode: exerciseCode, index: index)
                                ForEach(roles, id: \.id) { role in
                                    roleCell(roleId: role.id, exerciseCode: exerciseCode)
                                }
                            }
                        }
                    }
                }
                .frame(height: 420)
            }

            Spacer()

            HStack {
                Button("Cancel") {
                    dismiss()
                }
                .keyboardShortcut(.escape)

                Spacer()

                Button("Save") {
                    onSave(editableExercises, cleanedVariants())
                    dismiss()
                }
                .keyboardShortcut(.return)
                .buttonStyle(.borderedProminent)
            }
        }
        .padding()
        .frame(width: 760, height: 820)
    }

    private func addManualExercise() {
        guard !newExercise.trimmingCharacters(in: .whitespaces).isEmpty else { return }
        addExercise(newExercise.trimmingCharacters(in: .whitespaces))
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

    private func addExercise(_ code: String) {
        guard !editableExercises.contains(code) else { return }
        editableExercises.append(code)
        let defaults = plan.getRoleRepsDefaults(for: code)
        for role in roles.map(\.id) {
            if let range = defaults[role] {
                setOverride(role: role, exercise: code, range: range)
            }
        }
    }

    private func exerciseCell(exerciseCode: String, index: Int) -> some View {
        HStack(alignment: .top, spacing: 8) {
            VStack(alignment: .leading, spacing: 2) {
                if let displayName = plan.dictionary[exerciseCode] {
                    Text(displayName)
                        .font(.body)
                    Text(exerciseCode)
                        .font(.caption)
                        .foregroundColor(.secondary)
                } else {
                    Text(exerciseCode)
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

                Button(action: { removeExercise(exerciseCode) }) {
                    Image(systemName: "trash")
                        .foregroundColor(.red)
                }
                .buttonStyle(.borderless)
            }
        }
        .padding(.vertical, 6)
    }

    private func roleCell(roleId: String, exerciseCode: String) -> some View {
        let defaultRange = plan.getRoleRepsDefaults(for: exerciseCode)[roleId]
        let overrideRange = getOverride(role: roleId, exercise: exerciseCode)
        let displayRange = overrideRange ?? defaultRange
        let isDefault = overrideRange == nil && defaultRange != nil

        return VStack(alignment: .leading, spacing: 4) {
            HStack(spacing: 6) {
                TextField("Min", value: bindingMin(role: roleId, exercise: exerciseCode, fallback: displayRange?.min ?? 0), format: .number)
                    .textFieldStyle(.roundedBorder)
                    .frame(width: 52)
                TextField("Max", value: bindingMax(role: roleId, exercise: exerciseCode, fallback: displayRange?.max ?? 0), format: .number)
                    .textFieldStyle(.roundedBorder)
                    .frame(width: 52)
                if overrideRange != nil {
                    Button("↺") {
                        clearOverride(role: roleId, exercise: exerciseCode)
                    }
                    .buttonStyle(.borderless)
                    .help("Use default")
                }
            }
            if isDefault {
                Text("Default")
                    .font(.caption2)
                    .foregroundColor(.secondary)
            }
        }
        .padding(.vertical, 2)
    }

    private func removeExercise(_ code: String) {
        editableExercises.removeAll { $0 == code }
        for role in roles.map(\.id) {
            clearOverride(role: role, exercise: code)
        }
    }

    private func getOverride(role: String, exercise: String) -> RoleRepsRange? {
        guard let overrides = groupVariants[role]?[exercise],
              case .object(let repsObj)? = overrides["reps"],
              let min = repsObj["min"]?.intValue,
              let max = repsObj["max"]?.intValue else {
            return nil
        }
        return RoleRepsRange(min: min, max: max)
    }

    private func setOverride(role: String, exercise: String, range: RoleRepsRange) {
        if groupVariants[role] == nil {
            groupVariants[role] = [:]
        }
        if groupVariants[role]?[exercise] == nil {
            groupVariants[role]?[exercise] = [:]
        }
        groupVariants[role]?[exercise]?["reps"] = .object([
            "min": .number(Double(range.min)),
            "max": .number(Double(range.max))
        ])
    }

    private func clearOverride(role: String, exercise: String) {
        groupVariants[role]?[exercise]?.removeValue(forKey: "reps")
        if groupVariants[role]?[exercise]?.isEmpty == true {
            groupVariants[role]?.removeValue(forKey: exercise)
        }
        if groupVariants[role]?.isEmpty == true {
            groupVariants.removeValue(forKey: role)
        }
    }

    private func bindingMin(role: String, exercise: String, fallback: Int) -> Binding<Int> {
        Binding(
            get: {
                getOverride(role: role, exercise: exercise)?.min ?? fallback
            },
            set: { newValue in
                var maxValue = getOverride(role: role, exercise: exercise)?.max ?? fallback
                if maxValue <= 0 {
                    maxValue = newValue
                }
                if newValue <= 0 {
                    clearOverride(role: role, exercise: exercise)
                } else {
                    setOverride(role: role, exercise: exercise, range: RoleRepsRange(min: newValue, max: maxValue))
                }
            }
        )
    }

    private func bindingMax(role: String, exercise: String, fallback: Int) -> Binding<Int> {
        Binding(
            get: {
                getOverride(role: role, exercise: exercise)?.max ?? fallback
            },
            set: { newValue in
                var minValue = getOverride(role: role, exercise: exercise)?.min ?? fallback
                if minValue <= 0 {
                    minValue = newValue
                }
                if newValue <= 0 {
                    clearOverride(role: role, exercise: exercise)
                } else {
                    setOverride(role: role, exercise: exercise, range: RoleRepsRange(min: minValue, max: newValue))
                }
            }
        )
    }

    private func cleanedVariants() -> [String: [String: [String: JSONValue]]] {
        let exerciseSet = Set(editableExercises)
        var cleaned: [String: [String: [String: JSONValue]]] = [:]
        for (role, exercises) in groupVariants {
            let filtered = exercises.filter { exerciseSet.contains($0.key) }
            cleaned[role] = filtered
        }
        return cleaned
    }
}
