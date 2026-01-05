import SwiftUI

struct LoadAxisRow: View {
    let axisName: String
    @Binding var axis: LoadAxis
    let onRename: (String) -> Void
    let onDelete: () -> Void

    @State private var isExpanded = true
    @State private var draftName: String = ""
    @FocusState private var isNameFocused: Bool

    var body: some View {
        DisclosureGroup(isExpanded: $isExpanded) {
            VStack(alignment: .leading, spacing: 8) {
                Picker("Type", selection: Binding(
                    get: { axis.kind },
                    set: { axis = LoadAxis(kind: $0, values: axis.values) }
                )) {
                    Text("Categorical (labels)").tag(LoadAxisKind.categorical)
                    Text("Ordinal (ordered levels)").tag(LoadAxisKind.ordinal)
                }
                Text(axis.kind == .categorical
                     ? "Categorical = unordered labels (e.g., red/green/blue)."
                     : "Ordinal = ordered levels (e.g., 1/2/3 or light/medium/heavy).")
                    .font(.caption)
                    .foregroundColor(.secondary)

                Text("Values")
                    .font(.caption)

                ForEach(Array(axis.values.enumerated()), id: \.offset) { index, value in
                    HStack {
                        TextField("Value", text: Binding(
                            get: {
                                guard index < axis.values.count else { return "" }
                                return axis.values[index]
                            },
                            set: { newValue in
                                guard index < axis.values.count else { return }
                                axis.values[index] = newValue
                            }
                        ))
                        Button("Remove") {
                            guard index < axis.values.count else { return }
                            axis.values.remove(at: index)
                        }
                        .buttonStyle(.borderless)
                    }
                }

                Button("Add Value") {
                    axis.values.append("value\(axis.values.count + 1)")
                }
                .buttonStyle(.borderless)
            }
            .padding(.vertical, 4)
        } label: {
            HStack {
                TextField("Resistance type", text: $draftName)
                    .textFieldStyle(.roundedBorder)
                    .focused($isNameFocused)
                    .onSubmit { commitRename() }
                    .onChange(of: isNameFocused) { focused in
                        if !focused {
                            commitRename()
                        }
                    }
                    .onAppear {
                        if draftName.isEmpty {
                            draftName = axisName
                        }
                    }
                Spacer()
                Button("Delete") { onDelete() }
                    .buttonStyle(.borderless)
            }
        }
    }

    private func commitRename() {
        let trimmed = draftName.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmed.isEmpty, trimmed != axisName else {
            draftName = axisName
            return
        }
        onRename(trimmed)
    }
}
