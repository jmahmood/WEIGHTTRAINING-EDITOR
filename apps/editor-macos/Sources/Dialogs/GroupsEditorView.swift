import SwiftUI

struct GroupsEditorView: View {
    @ObservedObject var plan: PlanDocument
    let onSave: () -> Void

    @Environment(\.dismiss) private var dismiss
    @State private var groups: [String: [String]] = [:]
    @State private var selectedGroup: String?
    @State private var showingAddGroup = false
    @State private var newGroupName = ""

    var body: some View {
        VStack(spacing: 16) {
            Text("Exercise Groups")
                .font(.title2)
                .fontWeight(.bold)

            HSplitView {
                // Groups list
                VStack(alignment: .leading) {
                    HStack {
                        Text("Groups")
                            .font(.headline)
                        Spacer()
                        Button(action: { showingAddGroup = true }) {
                            Image(systemName: "plus.circle")
                        }
                        .buttonStyle(.borderless)
                    }

                    List(groups.keys.sorted(), id: \.self, selection: $selectedGroup) { key in
                        VStack(alignment: .leading, spacing: 4) {
                            Text(key)
                                .font(.body)
                            Text("\(groups[key]?.count ?? 0) exercises")
                                .font(.caption)
                                .foregroundColor(.secondary)
                        }
                        .tag(key)
                    }
                }
                .frame(minWidth: 200)

                // Group details
                if let groupName = selectedGroup, let exercises = groups[groupName] {
                    GroupDetailView(
                        groupName: groupName,
                        exercises: exercises,
                        onUpdate: { newExercises in
                            groups[groupName] = newExercises
                        },
                        onDelete: {
                            groups.removeValue(forKey: groupName)
                            selectedGroup = nil
                        }
                    )
                } else {
                    Text("Select a group to view details")
                        .foregroundColor(.secondary)
                        .frame(maxWidth: .infinity, maxHeight: .infinity)
                }
            }

            Divider()

            HStack {
                Button("Cancel") {
                    dismiss()
                }
                .keyboardShortcut(.escape)

                Spacer()

                Button("Save All") {
                    saveGroups()
                }
                .keyboardShortcut(.return)
                .buttonStyle(.borderedProminent)
            }
        }
        .padding()
        .frame(width: 600, height: 400)
        .onAppear {
            groups = plan.groups
        }
        .sheet(isPresented: $showingAddGroup) {
            AddGroupView(onAdd: { name in
                groups[name] = []
                selectedGroup = name
            })
        }
    }

    private func saveGroups() {
        do {
            // Get current plan JSON
            var updatedPlan = plan.planJSON

            // Remove old groups first by getting the difference
            let oldGroups = plan.groups
            for oldGroupName in oldGroups.keys {
                if !groups.keys.contains(oldGroupName) {
                    updatedPlan = try RustBridge.removeGroup(name: oldGroupName, from: updatedPlan)
                }
            }

            // Add or update all groups
            for (name, exercises) in groups {
                updatedPlan = try RustBridge.addGroup(name: name, exercises: exercises, to: updatedPlan)
            }

            // Update the plan
            plan.updatePlan(updatedPlan)

            dismiss()
            onSave()
        } catch {
            print("Error saving groups: \(error)")
            // TODO: Show error alert to user
        }
    }
}

struct GroupDetailView: View {
    let groupName: String
    let exercises: [String]
    let onUpdate: ([String]) -> Void
    let onDelete: () -> Void

    @State private var editableExercises: [String]
    @State private var newExercise = ""

    init(groupName: String, exercises: [String], onUpdate: @escaping ([String]) -> Void, onDelete: @escaping () -> Void) {
        self.groupName = groupName
        self.exercises = exercises
        self.onUpdate = onUpdate
        self.onDelete = onDelete
        _editableExercises = State(initialValue: exercises)
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack {
                Text(groupName)
                    .font(.headline)

                Spacer()

                Button("Delete Group", role: .destructive) {
                    onDelete()
                }
                .buttonStyle(.borderless)
            }

            Text("Exercises:")
                .font(.subheadline)
                .foregroundColor(.secondary)

            List {
                ForEach(editableExercises, id: \.self) { exercise in
                    Text(exercise)
                }
                .onDelete { offsets in
                    editableExercises.remove(atOffsets: offsets)
                    onUpdate(editableExercises)
                }
            }

            HStack {
                TextField("Add exercise code", text: $newExercise)
                    .textFieldStyle(.roundedBorder)
                    .onSubmit {
                        addExercise()
                    }

                Button("Add") {
                    addExercise()
                }
                .buttonStyle(.bordered)
                .disabled(newExercise.isEmpty)
            }
        }
        .padding()
    }

    private func addExercise() {
        guard !newExercise.isEmpty else { return }
        editableExercises.append(newExercise)
        onUpdate(editableExercises)
        newExercise = ""
    }
}

struct AddGroupView: View {
    let onAdd: (String) -> Void

    @Environment(\.dismiss) private var dismiss
    @State private var groupName = ""

    var body: some View {
        VStack(spacing: 20) {
            Text("Add New Group")
                .font(.title2)

            TextField("Group Name", text: $groupName)
                .textFieldStyle(.roundedBorder)

            HStack {
                Button("Cancel") {
                    dismiss()
                }

                Button("Add") {
                    onAdd(groupName)
                    dismiss()
                }
                .buttonStyle(.borderedProminent)
                .disabled(groupName.isEmpty)
            }
        }
        .padding()
        .frame(width: 300, height: 150)
    }
}
