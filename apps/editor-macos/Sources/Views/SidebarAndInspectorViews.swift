import SwiftUI

struct ExerciseSidebarView: View {
    @ObservedObject var plan: PlanDocument
    @EnvironmentObject var appState: AppState

    @State private var selectedTab = 0
    @State private var exerciseSearchText = ""
    @State private var groupSearchText = ""
    @State private var showingAddExercise = false
    @State private var showingAddGroup = false
    @State private var selectedGroupForEdit: String?

    var body: some View {
        VStack(spacing: 8) {
            Picker("", selection: $selectedTab) {
                Text("Exercises").tag(0)
                Text("Groups").tag(1)
            }
            .pickerStyle(.segmented)
            .padding(.horizontal)
            .padding(.top, 12)

            if selectedTab == 0 {
                exerciseLibrary
            } else {
                groupsLibrary
            }
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .background(Color(NSColor.windowBackgroundColor))
        .sheet(isPresented: $showingAddExercise) {
            AddExerciseSheet(plan: plan)
        }
        .sheet(item: Binding(
            get: { selectedGroupForEdit.map { GroupIdentifier(name: $0) } },
            set: { selectedGroupForEdit = $0?.name }
        )) { groupIdentifier in
            SingleGroupEditorView(
                plan: plan,
                groupName: groupIdentifier.name,
                exercises: plan.groups[groupIdentifier.name] ?? [],
                onSave: { updated in
                    appState.pushUndo(plan.planJSON)
                    updateGroup(groupIdentifier.name, exercises: updated)
                },
                onDelete: {
                    appState.pushUndo(plan.planJSON)
                    deleteGroup(groupIdentifier.name)
                }
            )
        }
    }

    private var exerciseLibrary: some View {
        VStack(alignment: .leading, spacing: 8) {
            SearchField(text: $exerciseSearchText, placeholder: "Search exercises...")
                .padding(.horizontal)

            HStack {
                Button {
                    showingAddExercise = true
                } label: {
                    Label("Add Exercise", systemImage: "plus.circle")
                }
                .buttonStyle(.borderedProminent)

                Spacer()
            }
            .padding(.horizontal)

            List(filteredExercises, id: \.key) { key, value in
                VStack(alignment: .leading, spacing: 2) {
                    Text(value)
                        .font(.body)
                    Text(key)
                        .font(.caption2)
                        .foregroundColor(.secondary)
                }
                .padding(.vertical, 4)
                .help("Code: \(key)")
            }
            .listStyle(.inset)
        }
    }

    private var groupsLibrary: some View {
        VStack(alignment: .leading, spacing: 8) {
            SearchField(text: $groupSearchText, placeholder: "Search groups, exercises, or codes")
                .padding(.horizontal)
                .padding(.top, 4)

            HStack {
                Button {
                    showingAddGroup = true
                } label: {
                    Label("Add Group", systemImage: "plus.circle")
                }
                .buttonStyle(.bordered)
                .sheet(isPresented: $showingAddGroup) {
                    AddNewGroupDialog(plan: plan) { name in
                        appState.pushUndo(plan.planJSON)
                        addNewGroup(name)
                        showingAddGroup = false
                    }
                }

                Spacer()
            }
            .padding(.horizontal)

            let groups = filteredGroups
            if groups.isEmpty {
                GroupSearchEmptyState(searchText: groupSearchText)
                    .padding()
            } else {
                List(groups.sorted(by: { $0.key < $1.key }), id: \.key) { name, exercises in
                    DisclosureGroup {
                        ForEach(exercises, id: \.self) { code in
                            VStack(alignment: .leading, spacing: 2) {
                                Text(plan.dictionary[code] ?? code)
                                    .font(.body)
                                Text(code)
                                    .font(.caption2)
                                    .foregroundColor(.secondary)
                            }
                            .padding(.vertical, 2)
                        }
                    } label: {
                        HStack {
                            VStack(alignment: .leading, spacing: 2) {
                                Text(name)
                                    .font(.body)
                                Text("\(exercises.count) exercises")
                                    .font(.caption)
                                    .foregroundColor(.secondary)
                            }
                            Spacer()
                            Button {
                                selectedGroupForEdit = name
                            } label: {
                                Image(systemName: "pencil")
                            }
                            .buttonStyle(.borderless)
                        }
                    }
                    .contextMenu {
                        Button("Edit Group") {
                            selectedGroupForEdit = name
                        }
                        Button("Delete Group", role: .destructive) {
                            appState.pushUndo(plan.planJSON)
                            deleteGroup(name)
                        }
                    }
                }
                .listStyle(.inset)
            }
        }
    }

    private var filteredExercises: [(key: String, value: String)] {
        let dictionary = plan.dictionary
        if exerciseSearchText.isEmpty {
            return dictionary.sorted(by: { $0.key < $1.key })
        }

        return dictionary.filter { key, value in
            key.localizedCaseInsensitiveContains(exerciseSearchText) ||
            value.localizedCaseInsensitiveContains(exerciseSearchText)
        }
        .sorted { $0.key < $1.key }
    }

    private var filteredGroups: [String: [String]] {
        if groupSearchText.isEmpty {
            return plan.groups
        }

        return plan.groups.filter { name, exercises in
            if name.localizedCaseInsensitiveContains(groupSearchText) {
                return true
            }
            return exercises.contains(where: { $0.localizedCaseInsensitiveContains(groupSearchText) }) ||
                   exercises.contains(where: { plan.dictionary[$0]?.localizedCaseInsensitiveContains(groupSearchText) == true })
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

struct InspectorView: View {
    @ObservedObject var plan: PlanDocument
    @EnvironmentObject var appState: AppState

    private var selection: SegmentDisplay? {
        guard let identifier = appState.selectedSegmentIds.first else { return nil }
        let components = identifier.split(separator: "_")
        guard components.count == 2,
              let day = Int(components[0]),
              let index = Int(components[1]),
              plan.days.indices.contains(day) else { return nil }
        let segments = plan.days[day].segments()
        guard segments.indices.contains(index) else { return nil }
        return segments[index]
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 16) {
            if let segment = selection {
                Text("Inspector")
                    .font(.headline)

                VStack(alignment: .leading, spacing: 8) {
                    Text(segment.primaryTitle(with: plan))
                        .font(.title3)
                        .fontWeight(.semibold)
                    Text(segment.humanReadableType)
                        .font(.caption)
                        .foregroundColor(.secondary)
                }

                Divider()

                SegmentDetailView(segment: segment, plan: plan)

                Spacer()
            } else {
                InspectorEmptyState()
            }
        }
        .padding()
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
        .background(Color(NSColor.windowBackgroundColor))
    }

}

struct InspectorMetricRow: View {
    let title: String
    let value: String

    var body: some View {
        VStack(alignment: .leading, spacing: 2) {
            Text(title.uppercased())
                .font(.caption2)
                .foregroundColor(.secondary)
            Text(value)
                .font(.body)
                .foregroundColor(value == "—" ? .secondary : .primary)
        }
    }
}

struct InspectorEmptyState: View {
    var body: some View {
        VStack(spacing: 12) {
            Image(systemName: "cursorarrow.rays")
                .font(.system(size: 40))
                .foregroundColor(.secondary)
            Text("No Selection")
                .font(.headline)
            Text("Select a segment on the canvas to inspect or edit its details.")
                .font(.caption)
                .multilineTextAlignment(.center)
                .foregroundColor(.secondary)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .center)
        .padding(.horizontal)
    }
}

struct GroupSearchEmptyState: View {
    let searchText: String

    var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            Text(searchText.isEmpty ? "No groups defined yet." : "No groups match “\(searchText)”")
                .font(.body)
            Text(searchText.isEmpty ? "Use the Add Group button to create one." : "Try searching by another group name or exercise.")
                .font(.caption)
                .foregroundColor(.secondary)
        }
        .frame(maxWidth: .infinity, alignment: .leading)
    }
}

struct SegmentDetailView: View {
    let segment: SegmentDisplay
    @ObservedObject var plan: PlanDocument

    var body: some View {
        switch segment.kind {
        case .straight, .rpe, .percentage, .amrap, .time:
            InspectorKeyValueList(items: segment.basicInspectorItems(plan: plan))
        case .superset, .circuit:
            GroupInspectorDetail(metadata: segment.groupMetadataItems(),
                                 exercises: segment.groupExercises(plan: plan))
        case .scheme:
            SchemeInspectorDetail(exerciseName: segment.primaryTitle(with: plan),
                                  code: segment.exerciseCode ?? "",
                                  sets: segment.schemeSetDetails())
        case .comment:
            CommentInspectorDetail(text: segment.commentText ?? "No comment provided.")
        case .choose:
            ChooseInspectorDetail(pick: segment.intValue("pick"),
                                  options: segment.choiceOptions(plan: plan))
        default:
            GenericInspectorDetail(dictionary: segment.segmentDict)
        }
    }
}

struct InspectorKeyValueList: View {
    let items: [InspectorItem]

    var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            ForEach(items) { item in
                InspectorMetricRow(title: item.title, value: item.value)
            }
        }
    }
}

struct GroupInspectorDetail: View {
    let metadata: [InspectorItem]
    let exercises: [GroupExerciseDetail]

    var body: some View {
        VStack(alignment: .leading, spacing: 16) {
            if !metadata.isEmpty {
                InspectorKeyValueList(items: metadata)
            }

            if !exercises.isEmpty {
                VStack(alignment: .leading, spacing: 12) {
                    Text("Exercises")
                        .font(.headline)
                    ForEach(exercises) { exercise in
                        VStack(alignment: .leading, spacing: 4) {
                            Text(exercise.name)
                                .font(.subheadline)
                                .fontWeight(.semibold)
                            if let code = exercise.code {
                                Text(code)
                                    .font(.caption2)
                                    .foregroundColor(.secondary)
                            }
                            if !exercise.details.isEmpty {
                                Text(exercise.details)
                                    .font(.caption)
                            }
                            if let notes = exercise.notes {
                                Text(notes)
                                    .font(.caption2)
                                    .foregroundColor(.secondary)
                            }
                        }
                        .padding(8)
                        .background(Color(NSColor.controlBackgroundColor))
                        .cornerRadius(6)
                    }
                }
            } else {
                Text("No exercises listed for this group.")
                    .font(.caption)
                    .foregroundColor(.secondary)
            }
        }
    }
}

struct SchemeInspectorDetail: View {
    let exerciseName: String
    let code: String
    let sets: [SchemeSetDetail]

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            InspectorMetricRow(title: "Exercise", value: exerciseName)
            if !code.isEmpty {
                InspectorMetricRow(title: "Code", value: code)
            }

            if sets.isEmpty {
                Text("No scheme sets defined.")
                    .font(.caption)
                    .foregroundColor(.secondary)
            } else {
                ForEach(sets) { set in
                    VStack(alignment: .leading, spacing: 4) {
                        Text(set.title)
                            .font(.subheadline)
                            .fontWeight(.semibold)
                        Text(set.summary)
                            .font(.caption)
                        if let rest = set.rest {
                            Text("Rest: \(rest)")
                                .font(.caption2)
                                .foregroundColor(.secondary)
                        }
                        if let notes = set.notes {
                            Text(notes)
                                .font(.caption2)
                                .foregroundColor(.secondary)
                        }
                    }
                    .padding(8)
                    .background(Color(NSColor.controlBackgroundColor))
                    .cornerRadius(6)
                }
            }
        }
    }
}

struct CommentInspectorDetail: View {
    let text: String

    var body: some View {
        Text(text)
            .font(.body)
            .padding()
            .frame(maxWidth: .infinity, alignment: .leading)
            .background(Color(NSColor.controlBackgroundColor))
            .cornerRadius(6)
    }
}

struct ChooseInspectorDetail: View {
    let pick: Int?
    let options: [String]

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            if let pick = pick {
                InspectorMetricRow(title: "Pick", value: "\(pick)")
            }

            if options.isEmpty {
                Text("No options provided.")
                    .font(.caption)
                    .foregroundColor(.secondary)
            } else {
                Text("Options")
                    .font(.headline)
                ForEach(options, id: \.self) { option in
                    Text("• \(option)")
                        .font(.caption)
                }
            }
        }
    }
}

struct GenericInspectorDetail: View {
    let dictionary: [String: Any]

    var body: some View {
        let rows = dictionary
            .map { InspectorItem(title: $0.key.capitalized.replacingOccurrences(of: "_", with: " "), value: SegmentDisplay.prettyValue($0.value)) }
            .sorted { $0.title < $1.title }
        InspectorKeyValueList(items: rows)
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
    @EnvironmentObject var appState: AppState

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
            appState.pushUndo(plan.planJSON)
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
        dismiss()
    }
}
