import SwiftUI

struct ExercisePicker: View {
    @ObservedObject var plan: PlanDocument
    @Binding var selectedExerciseCode: String
    @Binding var selectedExerciseName: String

    @State private var searchText = ""
    @State private var isSearching = false
    @FocusState private var isSearchFieldFocused: Bool

    var filteredExercises: [(String, String)] {
        let dict = plan.dictionary
        let exercises = dict.map { ($0.key, $0.value) }
            .sorted { $0.1 < $1.1 }

        if searchText.isEmpty {
            return []
        }

        return exercises.filter { code, name in
            name.localizedCaseInsensitiveContains(searchText) ||
            code.localizedCaseInsensitiveContains(searchText)
        }.prefix(10).map { $0 }
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            // Show selected exercise if one is selected and not searching
            if !selectedExerciseName.isEmpty && !isSearching {
                HStack {
                    VStack(alignment: .leading, spacing: 2) {
                        Text(selectedExerciseName)
                            .font(.body)
                        Text(selectedExerciseCode)
                            .font(.caption)
                            .foregroundColor(.secondary)
                    }

                    Spacer()

                    Button(action: {
                        searchText = ""
                        isSearching = true
                        isSearchFieldFocused = true
                    }) {
                        Text("Change")
                            .font(.caption)
                    }
                    .buttonStyle(.borderless)
                }
                .padding(8)
                .background(Color(NSColor.controlBackgroundColor))
                .cornerRadius(6)
            } else {
                // Search field
                HStack {
                    Image(systemName: "magnifyingglass")
                        .foregroundColor(.secondary)

                    TextField("Search exercises (e.g., bench press)...", text: $searchText)
                        .textFieldStyle(.plain)
                        .focused($isSearchFieldFocused)
                        .onChange(of: searchText) { _ in
                            isSearching = true
                        }

                    if !searchText.isEmpty {
                        Button(action: {
                            searchText = ""
                            if selectedExerciseName.isEmpty {
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

                // Results list
                if isSearching && !searchText.isEmpty {
                    if filteredExercises.isEmpty {
                        Text("No exercises found")
                            .font(.caption)
                            .foregroundColor(.secondary)
                            .padding(8)
                    } else {
                        VStack(alignment: .leading, spacing: 0) {
                            ForEach(filteredExercises, id: \.0) { code, name in
                                Button(action: {
                                    selectedExerciseCode = code
                                    selectedExerciseName = name
                                    searchText = ""
                                    isSearching = false
                                    isSearchFieldFocused = false
                                }) {
                                    VStack(alignment: .leading, spacing: 2) {
                                        Text(name)
                                            .font(.body)
                                            .foregroundColor(.primary)
                                        Text(code)
                                            .font(.caption)
                                            .foregroundColor(.secondary)
                                    }
                                    .frame(maxWidth: .infinity, alignment: .leading)
                                    .padding(.vertical, 6)
                                    .padding(.horizontal, 8)
                                }
                                .buttonStyle(.plain)
                                .background(Color(NSColor.controlBackgroundColor).opacity(0.5))

                                if code != filteredExercises.last?.0 {
                                    Divider()
                                }
                            }
                        }
                        .background(Color(NSColor.controlBackgroundColor))
                        .cornerRadius(6)
                        .overlay(
                            RoundedRectangle(cornerRadius: 6)
                                .stroke(Color.secondary.opacity(0.2), lineWidth: 1)
                        )
                    }
                }
            }
        }
        .onAppear {
            if selectedExerciseName.isEmpty {
                isSearching = true
                isSearchFieldFocused = true
            }
        }
    }
}

struct ExercisePickerDialog: View {
    @ObservedObject var plan: PlanDocument
    @Binding var searchText: String
    let onSelect: (String, String) -> Void

    @Environment(\.dismiss) private var dismiss

    var filteredExercises: [(String, String)] {
        let dict = plan.dictionary
        let exercises = dict.map { ($0.key, $0.value) }
            .sorted { $0.1 < $1.1 }

        if searchText.isEmpty {
            return exercises
        }

        return exercises.filter { code, name in
            name.localizedCaseInsensitiveContains(searchText) ||
            code.localizedCaseInsensitiveContains(searchText)
        }
    }

    var body: some View {
        VStack(spacing: 0) {
            // Header
            HStack {
                Text("Select Exercise")
                    .font(.title2)
                    .fontWeight(.bold)

                Spacer()

                Button("Cancel") {
                    dismiss()
                }
                .keyboardShortcut(.escape)
            }
            .padding()

            // Search
            HStack {
                Image(systemName: "magnifyingglass")
                    .foregroundColor(.secondary)

                TextField("Search exercises...", text: $searchText)
                    .textFieldStyle(.plain)

                if !searchText.isEmpty {
                    Button(action: { searchText = "" }) {
                        Image(systemName: "xmark.circle.fill")
                            .foregroundColor(.secondary)
                    }
                    .buttonStyle(.borderless)
                }
            }
            .padding(8)
            .background(Color(NSColor.controlBackgroundColor))
            .cornerRadius(8)
            .padding(.horizontal)
            .padding(.bottom, 8)

            Divider()

            // Exercise list
            if filteredExercises.isEmpty {
                VStack(spacing: 12) {
                    Image(systemName: "magnifyingglass")
                        .font(.system(size: 48))
                        .foregroundColor(.secondary)
                    Text("No exercises found")
                        .font(.headline)
                        .foregroundColor(.secondary)
                    if !searchText.isEmpty {
                        Text("Try a different search term")
                            .font(.caption)
                            .foregroundColor(.secondary)
                    }
                }
                .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else {
                List(filteredExercises, id: \.0) { code, name in
                    Button(action: {
                        onSelect(code, name)
                    }) {
                        VStack(alignment: .leading, spacing: 4) {
                            Text(name)
                                .font(.body)
                                .foregroundColor(.primary)
                            Text(code)
                                .font(.caption)
                                .foregroundColor(.secondary)
                        }
                        .padding(.vertical, 4)
                    }
                    .buttonStyle(.plain)
                }
            }

            // Footer
            HStack {
                Text("\(filteredExercises.count) exercise(s)")
                    .font(.caption)
                    .foregroundColor(.secondary)

                Spacer()
            }
            .padding()
        }
        .frame(width: 500, height: 600)
    }
}
