import SwiftUI

struct LoadAxesEditorView: View {
    @ObservedObject var plan: PlanDocument
    @Environment(\.dismiss) private var dismiss

    @State private var exerciseMeta: [String: ExerciseMeta] = [:]
    @State private var selectedExercise: String?
    @State private var hasChanges = false
    @State private var searchText = ""

    init(plan: PlanDocument) {
        self.plan = plan
        _exerciseMeta = State(initialValue: plan.getExerciseMeta())
    }

    var body: some View {
        VStack(spacing: 0) {
            HStack {
                Text("Alternative Resistance Types")
                    .font(.headline)
                Spacer()
                Button("Close") { dismiss() }
            }
            .padding()

            Divider()

            HSplitView {
                exerciseList
                    .frame(minWidth: 320)
                loadAxesEditor
                    .frame(minWidth: 520)
            }
            .frame(maxWidth: .infinity, maxHeight: .infinity)

            Divider()

            HStack {
                Spacer()
                Button("Cancel") { dismiss() }
                Button("Save") {
                    plan.updateExerciseMeta(exerciseMeta)
                    hasChanges = false
                    dismiss()
                }
                .buttonStyle(.borderedProminent)
                .disabled(!hasChanges)
            }
            .padding()
        }
        .frame(width: 900, height: 520)
    }

    private var exerciseList: some View {
        let exerciseCodes = Set(plan.dictionary.keys).union(exerciseMeta.keys)
        let sorted = Array(exerciseCodes).sorted()
        let filtered = sorted.filter { code in
            if searchText.isEmpty { return true }
            return code.localizedCaseInsensitiveContains(searchText) ||
                (plan.dictionary[code]?.localizedCaseInsensitiveContains(searchText) ?? false)
        }

        return VStack(alignment: .leading, spacing: 8) {
            TextField("Search exercises...", text: $searchText)
                .textFieldStyle(.roundedBorder)

            List(filtered, id: \.self, selection: $selectedExercise) { code in
                HStack {
                    VStack(alignment: .leading, spacing: 2) {
                        Text(plan.dictionary[code] ?? code)
                            .font(.body)
                        Text(code)
                            .font(.caption2)
                            .foregroundColor(.secondary)
                    }
                    Spacer()
                    if let meta = exerciseMeta[code], !meta.loadAxes.isEmpty {
                        Image(systemName: "checkmark.circle.fill")
                            .foregroundColor(.green)
                    }
                }
            }
        }
        .padding()
    }

    private var loadAxesEditor: some View {
        VStack(alignment: .leading, spacing: 12) {
            if let exercise = selectedExercise {
                VStack(alignment: .leading, spacing: 4) {
                    Text(plan.dictionary[exercise] ?? exercise)
                        .font(.headline)
                    Text(exercise)
                        .font(.caption)
                        .foregroundColor(.secondary)
                }

                let loadAxesBinding = Binding<[String: LoadAxis]>(
                    get: { exerciseMeta[exercise]?.loadAxes ?? [:] },
                    set: { newValue in
                        if exerciseMeta[exercise] == nil {
                            exerciseMeta[exercise] = ExerciseMeta(loadAxes: newValue, roleReps: [:])
                        } else {
                            exerciseMeta[exercise]?.loadAxes = newValue
                        }
                        hasChanges = true
                    }
                )

                List {
                    ForEach(Array(loadAxesBinding.wrappedValue.keys).sorted(), id: \.self) { axisName in
                        if let axisBinding = bindingForAxis(exercise: exercise, axisName: axisName) {
                            LoadAxisRow(
                                axisName: axisName,
                                axis: axisBinding,
                                onRename: { newName in
                                    renameAxis(in: exercise, from: axisName, to: newName)
                                },
                                onDelete: {
                                    var current = loadAxesBinding.wrappedValue
                                    current.removeValue(forKey: axisName)
                                    loadAxesBinding.wrappedValue = current
                                }
                            )
                        }
                    }
                }
                .frame(minHeight: 280)

                Button("Add Resistance Type") {
                    addAxis(to: exercise)
                }
                .buttonStyle(.bordered)
            } else {
                Text("Select an exercise")
                    .font(.caption)
                    .foregroundColor(.secondary)
            }
            Spacer()
        }
        .padding()
    }

    private func bindingForAxis(exercise: String, axisName: String) -> Binding<LoadAxis>? {
        guard exerciseMeta[exercise]?.loadAxes[axisName] != nil else { return nil }
        let axisBinding = Binding<LoadAxis>(
            get: { exerciseMeta[exercise]?.loadAxes[axisName] ?? LoadAxis(kind: .categorical, values: []) },
            set: { newValue in
                exerciseMeta[exercise]?.loadAxes[axisName] = newValue
                hasChanges = true
            }
        )
        return axisBinding
    }

    private func renameAxis(in exercise: String, from axisName: String, to newName: String) {
        let trimmed = newName.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmed.isEmpty, trimmed != axisName else { return }
        var loadAxes = exerciseMeta[exercise]?.loadAxes ?? [:]
        if loadAxes[trimmed] != nil {
            return
        }
        let axis = loadAxes.removeValue(forKey: axisName)
        if let axis = axis {
            loadAxes[trimmed] = axis
            exerciseMeta[exercise]?.loadAxes = loadAxes
            hasChanges = true
        }
    }

    private func addAxis(to exercise: String) {
        let base = "axis"
        var name = base
        var counter = 1
        var loadAxes = exerciseMeta[exercise]?.loadAxes ?? [:]
        while loadAxes[name] != nil {
            counter += 1
            name = "\(base)_\(counter)"
        }
        loadAxes[name] = LoadAxis(kind: .categorical, values: ["value1", "value2"])
        if exerciseMeta[exercise] == nil {
            exerciseMeta[exercise] = ExerciseMeta(loadAxes: loadAxes, roleReps: [:])
        } else {
            exerciseMeta[exercise]?.loadAxes = loadAxes
        }
        hasChanges = true
    }
}
