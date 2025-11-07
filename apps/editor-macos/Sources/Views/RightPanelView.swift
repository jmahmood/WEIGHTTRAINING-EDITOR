import SwiftUI

struct RightPanelView: View {
    @ObservedObject var plan: PlanDocument
    @State private var selectedTab = 0

    var body: some View {
        VStack(spacing: 0) {
            // Tab selector
            Picker("", selection: $selectedTab) {
                Text("Exercises").tag(0)
                Text("Groups").tag(1)
            }
            .pickerStyle(.segmented)
            .padding()

            // Tab content
            if selectedTab == 0 {
                DictionaryTabView(plan: plan)
            } else {
                GroupsTabView(plan: plan)
            }
        }
        .background(Color(NSColor.controlBackgroundColor))
    }
}

struct DictionaryTabView: View {
    @ObservedObject var plan: PlanDocument

    @State private var searchText = ""

    var body: some View {
        VStack {
            SearchField(text: $searchText, placeholder: "Search exercises...")
                .padding()

            let dictionary = plan.dictionary
            let filtered = searchText.isEmpty ? dictionary : dictionary.filter { key, value in
                key.localizedCaseInsensitiveContains(searchText) ||
                value.localizedCaseInsensitiveContains(searchText)
            }

            if filtered.isEmpty {
                Text(searchText.isEmpty ? "No exercises in dictionary" : "No matches")
                    .foregroundColor(.secondary)
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else {
                List(filtered.sorted(by: { $0.key < $1.key }), id: \.key) { key, value in
                    VStack(alignment: .leading, spacing: 4) {
                        Text(value)
                            .font(.body)
                        Text(key)
                            .font(.caption)
                            .foregroundColor(.secondary)
                    }
                    .padding(.vertical, 4)
                }
            }
        }
    }
}

struct GroupsTabView: View {
    @ObservedObject var plan: PlanDocument
    @EnvironmentObject var appState: AppState
    @State private var selectedGroupForEdit: String?
    @State private var showingAddGroup = false
    @State private var searchText = ""

    var filteredGroups: [String: [String]] {
        let groups = plan.groups

        if searchText.isEmpty {
            return groups
        }

        return groups.filter { groupName, exercises in
            // Search by group name
            if groupName.localizedCaseInsensitiveContains(searchText) {
                return true
            }

            // Search by exercise codes
            if exercises.contains(where: { $0.localizedCaseInsensitiveContains(searchText) }) {
                return true
            }

            // Search by exercise display names
            for exerciseCode in exercises {
                if let displayName = plan.dictionary[exerciseCode],
                   displayName.localizedCaseInsensitiveContains(searchText) {
                    return true
                }
            }

            return false
        }
    }

    var body: some View {
        VStack {
            HStack {
                Text("Substitution Groups")
                    .font(.headline)

                Spacer()

                Button(action: {
                    showingAddGroup = true
                }) {
                    Image(systemName: "plus.circle")
                }
                .help("Add new group")
            }
            .padding()

            SearchField(text: $searchText, placeholder: "Search groups...")
                .padding(.horizontal)
                .padding(.bottom, 8)

            let groups = filteredGroups

            if groups.isEmpty {
                if searchText.isEmpty {
                    Text("No groups defined")
                        .foregroundColor(.secondary)
                        .frame(maxWidth: .infinity, maxHeight: .infinity)
                } else {
                    Text("No groups found")
                        .foregroundColor(.secondary)
                        .frame(maxWidth: .infinity, maxHeight: .infinity)
                }
            } else {
                List(groups.sorted(by: { $0.key < $1.key }), id: \.key) { key, value in
                    DisclosureGroup {
                        ForEach(value, id: \.self) { exercise in
                            HStack {
                                Text("•")
                                    .foregroundColor(.secondary)
                                VStack(alignment: .leading) {
                                    if let displayName = plan.dictionary[exercise] {
                                        Text(displayName)
                                            .font(.caption)
                                        Text(exercise)
                                            .font(.caption2)
                                            .foregroundColor(.secondary)
                                    } else {
                                        Text(exercise)
                                            .font(.caption)
                                    }
                                }
                            }
                        }
                    } label: {
                        HStack {
                            VStack(alignment: .leading, spacing: 4) {
                                Text(key)
                                    .font(.body)
                                    .fontWeight(.semibold)
                                Text("\(value.count) exercises")
                                    .font(.caption)
                                    .foregroundColor(.secondary)
                            }

                            Spacer()

                            Button(action: {
                                selectedGroupForEdit = key
                            }) {
                                Image(systemName: "pencil.circle")
                                    .foregroundColor(.blue)
                            }
                            .buttonStyle(.borderless)
                        }
                    }
                    .padding(.vertical, 4)
                    .contextMenu {
                        Button("Edit Group") {
                            selectedGroupForEdit = key
                        }
                        Button("Delete Group", role: .destructive) {
                            deleteGroup(key)
                        }
                    }
                }
            }
        }
        .sheet(item: Binding(
            get: { selectedGroupForEdit.map { GroupIdentifier(name: $0) } },
            set: { selectedGroupForEdit = $0?.name }
        )) { groupId in
            SingleGroupEditorView(
                plan: plan,
                groupName: groupId.name,
                exercises: plan.groups[groupId.name] ?? [],
                onSave: { newExercises in
                    updateGroup(groupId.name, exercises: newExercises)
                },
                onDelete: {
                    deleteGroup(groupId.name)
                    selectedGroupForEdit = nil
                }
            )
        }
        .sheet(isPresented: $showingAddGroup) {
            AddNewGroupDialog(plan: plan, onAdd: { groupName in
                addNewGroup(groupName)
                showingAddGroup = false
            })
        }
    }

    private func addNewGroup(_ name: String) {
        do {
            let updatedJSON = try RustBridge.addGroup(name: name, exercises: [], to: plan.planJSON)
            plan.updatePlan(updatedJSON)
        } catch {
            print("Error adding group: \(error)")
        }
    }

    private func updateGroup(_ name: String, exercises: [String]) {
        do {
            let updatedJSON = try RustBridge.addGroup(name: name, exercises: exercises, to: plan.planJSON)
            plan.updatePlan(updatedJSON)
        } catch {
            print("Error updating group: \(error)")
        }
    }

    private func deleteGroup(_ name: String) {
        do {
            let updatedJSON = try RustBridge.removeGroup(name: name, from: plan.planJSON)
            plan.updatePlan(updatedJSON)
        } catch {
            print("Error deleting group: \(error)")
        }
    }
}

struct GroupIdentifier: Identifiable {
    let name: String
    var id: String { name }
}

struct AddNewGroupDialog: View {
    @ObservedObject var plan: PlanDocument
    let onAdd: (String) -> Void

    @Environment(\.dismiss) private var dismiss
    @State private var groupName = ""
    @State private var showError = false
    @State private var errorMessage = ""

    var body: some View {
        VStack(spacing: 20) {
            Text("Add New Group")
                .font(.title2)
                .fontWeight(.bold)

            VStack(alignment: .leading, spacing: 8) {
                Text("Group Name:")
                    .font(.headline)

                TextField("Enter group name", text: $groupName)
                    .textFieldStyle(.roundedBorder)

                if showError {
                    Text(errorMessage)
                        .font(.caption)
                        .foregroundColor(.red)
                }
            }

            HStack {
                Button("Cancel") {
                    dismiss()
                }
                .keyboardShortcut(.escape)

                Spacer()

                Button("Add") {
                    addGroup()
                }
                .keyboardShortcut(.return)
                .buttonStyle(.borderedProminent)
                .disabled(groupName.isEmpty)
            }
        }
        .padding()
        .frame(width: 400, height: 200)
    }

    private func addGroup() {
        let trimmedName = groupName.trimmingCharacters(in: .whitespaces)

        guard !trimmedName.isEmpty else {
            showError = true
            errorMessage = "Group name cannot be empty"
            return
        }

        if plan.groups.keys.contains(trimmedName) {
            showError = true
            errorMessage = "A group with this name already exists"
            return
        }

        onAdd(trimmedName)
    }
}

struct SearchField: View {
    @Binding var text: String
    let placeholder: String

    var body: some View {
        HStack {
            Image(systemName: "magnifyingglass")
                .foregroundColor(.secondary)

            TextField(placeholder, text: $text)
                .textFieldStyle(.plain)

            if !text.isEmpty {
                Button(action: { text = "" }) {
                    Image(systemName: "xmark.circle.fill")
                        .foregroundColor(.secondary)
                }
                .buttonStyle(.borderless)
            }
        }
        .padding(8)
        .background(Color(NSColor.controlBackgroundColor))
        .cornerRadius(8)
    }
}
