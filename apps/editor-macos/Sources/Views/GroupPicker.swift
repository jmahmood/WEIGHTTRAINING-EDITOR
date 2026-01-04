import SwiftUI

/// A searchable group picker that filters by group name, exercise name, or exercise code
/// This component is used consistently everywhere for selecting alternative groups
struct GroupPicker: View {
    @ObservedObject var plan: PlanDocument
    @Binding var selectedGroup: String?
    var onChange: (() -> Void)? = nil

    @State private var searchText = ""
    @State private var isSearching = false
    @EnvironmentObject private var appState: AppState

    var filteredGroups: [String] {
        let allGroups = Array(plan.groups.keys).sorted()

        if searchText.isEmpty {
            return allGroups
        }

        return allGroups.filter { groupName in
            // Search by group name
            if groupName.localizedCaseInsensitiveContains(searchText) {
                return true
            }

            // Search by exercises in the group
            if let exercises = plan.groups[groupName] {
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
            }

            return false
        }
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            // Show selected group if one is selected and not searching
            if let selected = selectedGroup, !isSearching {
                HStack {
                    VStack(alignment: .leading, spacing: 2) {
                        Text(selected)
                            .font(.body)
                        if let exercises = plan.groups[selected] {
                            Text("\(exercises.count) exercises")
                                .font(.caption)
                                .foregroundColor(.secondary)
                        }
                    }

                    Spacer()

                    Button(action: {
                        selectedGroup = nil
                        onChange?()
                    }) {
                        Text("Clear")
                            .font(.caption)
                    }
                    .buttonStyle(.borderless)

                    Button(action: {
                        searchText = ""
                        isSearching = true
                    }) {
                        Text("Change")
                            .font(.caption)
                    }
                    .buttonStyle(.borderless)

                    Button(action: {
                        appState.focusedGroupName = selected
                    }) {
                        Text("Edit")
                            .font(.caption)
                    }
                    .buttonStyle(.borderless)
                }
                .padding(8)
                .background(Color(NSColor.controlBackgroundColor))
                .cornerRadius(6)
            } else {
                // Search field
                VStack(spacing: 0) {
                    HStack {
                        Image(systemName: "magnifyingglass")
                            .foregroundColor(.secondary)

                        TextField("Search groups...", text: $searchText)
                            .textFieldStyle(.plain)
                            .onChange(of: searchText) { _ in
                                isSearching = true
                            }
                            .onSubmit {
                                // Select first result on Enter
                                if let firstGroup = filteredGroups.first {
                                    selectedGroup = firstGroup
                                    searchText = ""
                                    isSearching = false
                                    onChange?()
                                }
                            }

                        if !searchText.isEmpty {
                            Button(action: {
                                searchText = ""
                                if selectedGroup == nil {
                                    isSearching = true
                                } else {
                                    isSearching = false
                                }
                            }) {
                                Image(systemName: "xmark.circle.fill")
                                    .foregroundColor(.secondary)
                            }
                            .buttonStyle(.borderless)
                        }
                    }
                    .padding(8)
                    .background(Color(NSColor.controlBackgroundColor))
                    .cornerRadius(6)

                    // Hidden button to handle Escape key
                    Button("") {
                        searchText = ""
                        if selectedGroup != nil {
                            isSearching = false
                        }
                    }
                    .keyboardShortcut(.escape, modifiers: [])
                    .hidden()
                }

                // Results list
                if isSearching {
                    ScrollView {
                        VStack(alignment: .leading, spacing: 0) {
                            // None option
                            Button(action: {
                                selectedGroup = nil
                                isSearching = false
                                onChange?()
                            }) {
                                HStack {
                                    Text("None")
                                        .font(.body)
                                        .foregroundColor(.primary)
                                    Spacer()
                                    if selectedGroup == nil {
                                        Image(systemName: "checkmark")
                                            .foregroundColor(.blue)
                                    }
                                }
                                .frame(maxWidth: .infinity, alignment: .leading)
                                .padding(.vertical, 6)
                                .padding(.horizontal, 8)
                            }
                            .buttonStyle(.plain)
                            .background(Color(NSColor.controlBackgroundColor).opacity(0.5))

                            Divider()

                            if filteredGroups.isEmpty {
                                Text("No groups found")
                                    .font(.caption)
                                    .foregroundColor(.secondary)
                                    .padding(8)
                            } else {
                                ForEach(filteredGroups, id: \.self) { groupName in
                                    Button(action: {
                                        selectedGroup = groupName
                                        searchText = ""
                                        isSearching = false
                                        onChange?()
                                    }) {
                                        HStack {
                                            VStack(alignment: .leading, spacing: 2) {
                                                Text(groupName)
                                                    .font(.body)
                                                    .foregroundColor(.primary)
                                                if let exercises = plan.groups[groupName] {
                                                    Text("\(exercises.count) exercises")
                                                        .font(.caption)
                                                        .foregroundColor(.secondary)
                                                }
                                            }
                                            Spacer()
                                            if selectedGroup == groupName {
                                                Image(systemName: "checkmark")
                                                    .foregroundColor(.blue)
                                            }
                                        }
                                        .frame(maxWidth: .infinity, alignment: .leading)
                                        .padding(.vertical, 6)
                                        .padding(.horizontal, 8)
                                    }
                                    .buttonStyle(.plain)
                                    .background(Color(NSColor.controlBackgroundColor).opacity(0.5))

                                    if groupName != filteredGroups.last {
                                        Divider()
                                    }
                                }
                            }
                        }
                    }
                    .frame(maxHeight: 200)
                    .background(Color(NSColor.controlBackgroundColor))
                    .cornerRadius(6)
                    .overlay(
                        RoundedRectangle(cornerRadius: 6)
                            .stroke(Color.secondary.opacity(0.2), lineWidth: 1)
                    )
                }
            }
        }
        .onAppear {
            if selectedGroup == nil {
                isSearching = true
            }
        }
    }
}
