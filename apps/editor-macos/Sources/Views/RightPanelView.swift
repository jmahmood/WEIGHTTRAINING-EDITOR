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
    @State private var showingAddExercise = false

    var body: some View {
        VStack {
            SearchField(text: $searchText, placeholder: "Search exercises...")
                .padding()

            HStack {
                Spacer()
                Button {
                    showingAddExercise = true
                } label: {
                    Label("Add Exercise", systemImage: "plus.circle")
                }
                .buttonStyle(.borderedProminent)
            }
            .padding(.horizontal)
            .padding(.bottom, 8)

            let dictionary = plan.dictionary
            let filtered = searchText.isEmpty ? dictionary : dictionary.filter { key, value in
                key.localizedCaseInsensitiveContains(searchText) ||
                value.localizedCaseInsensitiveContains(searchText)
            }

            Divider()

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
        .sheet(isPresented: $showingAddExercise) {
            AddExerciseSheet(plan: plan)
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

struct NewExerciseForm: View {
    @ObservedObject var plan: PlanDocument

    @State private var selectedPatternIndex = 0
    @State private var selectedImplementIndex = 0
    @State private var customPattern = ""
    @State private var customImplement = ""
    @State private var variant = ""
    @State private var exerciseName = ""
    @State private var errorMessage: String?
    @State private var successMessage: String?

    private let patternOptions = [
        "SQ","BP","DL","OHP","ROW","PULLUP","DIP","HINGE","LUNGE","CALF","CORE","CARRY","CURL","EXT","RAISE","Custom…"
    ]

    private let implementOptions = [
        "BB","DB","KB","BW","CBL","MACH","SM","SWISS","TB","SSB","Custom…"
    ]

    private var isPatternCustom: Bool {
        selectedPatternIndex == patternOptions.count - 1
    }

    private var isImplementCustom: Bool {
        selectedImplementIndex == implementOptions.count - 1
    }

    private var patternValue: String? {
        let raw = isPatternCustom ? customPattern : patternOptions[selectedPatternIndex]
        let trimmed = raw.trimmingCharacters(in: .whitespacesAndNewlines)
        if trimmed.isEmpty { return nil }
        return trimmed.uppercased()
    }

    private var implementValue: String? {
        let raw = isImplementCustom ? customImplement : implementOptions[selectedImplementIndex]
        let trimmed = raw.trimmingCharacters(in: .whitespacesAndNewlines)
        if trimmed.isEmpty { return nil }
        return trimmed.uppercased()
    }

    private var variantValue: String {
        variant.trimmingCharacters(in: .whitespacesAndNewlines).uppercased()
    }

    private var trimmedName: String {
        exerciseName.trimmingCharacters(in: .whitespacesAndNewlines)
    }

    private var isFormReady: Bool {
        patternValue != nil &&
        implementValue != nil &&
        !variantValue.isEmpty &&
        !trimmedName.isEmpty
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            HStack(spacing: 12) {
                VStack(alignment: .leading) {
                    Text("Pattern")
                        .font(.caption)
                        .foregroundColor(.secondary)
                    Picker("", selection: $selectedPatternIndex) {
                        ForEach(patternOptions.indices, id: \.self) { idx in
                            Text(patternOptions[idx]).tag(idx)
                        }
                    }
                    .pickerStyle(.menu)
                }

                if isPatternCustom {
                    TextField("Custom pattern", text: $customPattern)
                        .textFieldStyle(.roundedBorder)
                        .frame(width: 120)
                }

                Text(".")
                    .font(.headline)

                VStack(alignment: .leading) {
                    Text("Implement")
                        .font(.caption)
                        .foregroundColor(.secondary)
                    Picker("", selection: $selectedImplementIndex) {
                        ForEach(implementOptions.indices, id: \.self) { idx in
                            Text(implementOptions[idx]).tag(idx)
                        }
                    }
                    .pickerStyle(.menu)
                }

                if isImplementCustom {
                    TextField("Custom implement", text: $customImplement)
                        .textFieldStyle(.roundedBorder)
                        .frame(width: 120)
                }

                Text(".")
                    .font(.headline)

                TextField("Variant", text: $variant)
                    .textFieldStyle(.roundedBorder)
                    .frame(width: 120)
            }

            TextField("Exercise Name (e.g., Bench Press)", text: $exerciseName)
                .textFieldStyle(.roundedBorder)

            HStack {
                Button(action: addExercise) {
                    Label("Add Exercise", systemImage: "plus.circle")
                }
                .buttonStyle(.borderedProminent)
                .disabled(!isFormReady)

                Spacer()

                Text("Code looks like PATTERN.IMPLEMENT.VARIANT")
                    .font(.caption)
                    .foregroundColor(.secondary)
            }

            if let error = errorMessage {
                Text(error)
                    .font(.caption)
                    .foregroundColor(.red)
            } else if let success = successMessage {
                Text(success)
                    .font(.caption)
                    .foregroundColor(.green)
            }
        }
        .padding()
        .background(Color(NSColor.controlBackgroundColor))
        .cornerRadius(8)
        .onChange(of: selectedPatternIndex) { _ in clearFeedback() }
        .onChange(of: selectedImplementIndex) { _ in clearFeedback() }
        .onChange(of: customPattern) { _ in clearFeedback() }
        .onChange(of: customImplement) { _ in clearFeedback() }
        .onChange(of: variant) { _ in clearFeedback() }
        .onChange(of: exerciseName) { _ in clearFeedback() }
    }

    private func addExercise() {
        guard let pattern = patternValue else {
            errorMessage = "Select or enter a pattern"
            successMessage = nil
            return
        }

        guard let implement = implementValue else {
            errorMessage = "Select or enter an implement"
            successMessage = nil
            return
        }

        let variantText = variantValue
        guard !variantText.isEmpty else {
            errorMessage = "Provide a variant code"
            successMessage = nil
            return
        }

        let nameText = trimmedName
        guard !nameText.isEmpty else {
            errorMessage = "Provide a display name"
            successMessage = nil
            return
        }

        let code = "\(pattern).\(implement).\(variantText)"

        if plan.dictionary.keys.contains(code) {
            errorMessage = "An exercise with code \(code) already exists"
            successMessage = nil
            return
        }

        if plan.dictionary.values.contains(where: { $0.caseInsensitiveCompare(nameText) == .orderedSame }) {
            errorMessage = "An exercise with this name already exists"
            successMessage = nil
            return
        }

        do {
            try plan.addExercise(code: code, name: nameText)
            successMessage = "Added \(nameText)"
            errorMessage = nil
            variant = ""
            exerciseName = ""
            customPattern = ""
            customImplement = ""
        } catch {
            errorMessage = error.localizedDescription
            successMessage = nil
        }
    }

    private func clearFeedback() {
        errorMessage = nil
        successMessage = nil
    }
}

struct AddExerciseSheet: View {
    @ObservedObject var plan: PlanDocument
    @Environment(\.dismiss) private var dismiss

    var body: some View {
        VStack(alignment: .leading, spacing: 16) {
            HStack {
                Text("Add New Exercise")
                    .font(.title2)
                    .fontWeight(.bold)
                Spacer()
                Button("Close") {
                    dismiss()
                }
                .keyboardShortcut(.cancelAction)
            }

            Divider()

            NewExerciseForm(plan: plan)

            Spacer()
        }
        .padding()
        .frame(minWidth: 520)
    }
}
