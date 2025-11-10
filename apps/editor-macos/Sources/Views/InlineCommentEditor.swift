import SwiftUI

/// Inline editable view for comment segments
struct InlineCommentEditor: View {
    @ObservedObject var plan: PlanDocument
    let segment: SegmentDisplay
    @EnvironmentObject var appState: AppState

    @State private var isEditing = false
    @State private var commentText: String = ""

    @FocusState private var isTextFocused: Bool

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack {
                Text("Comment")
                    .font(.caption)
                    .foregroundColor(.secondary)
                Spacer()
                Button(isEditing ? "Save" : "Edit") {
                    if isEditing {
                        saveChanges()
                    } else {
                        startEditing()
                    }
                }
                .buttonStyle(.borderedProminent)

                if isEditing {
                    Button("Cancel") {
                        cancelEditing()
                    }
                    .keyboardShortcut(.escape, modifiers: [])
                }
            }

            if isEditing {
                TextEditor(text: $commentText)
                    .frame(minHeight: 100, maxHeight: 200)
                    .focused($isTextFocused)
                    .border(Color.gray.opacity(0.3))
            } else {
                Text(commentText.isEmpty ? "No comment" : commentText)
                    .font(.body)
                    .foregroundColor(commentText.isEmpty ? .secondary : .primary)
                    .padding()
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .background(Color(NSColor.controlBackgroundColor))
                    .cornerRadius(6)
            }
        }
        .onAppear {
            loadCurrentValue()
        }
    }

    private func startEditing() {
        loadCurrentValue()
        isEditing = true
        isTextFocused = true
    }

    private func cancelEditing() {
        isEditing = false
        isTextFocused = false
        loadCurrentValue() // Reset to original value
    }

    private func saveChanges() {
        var updatedDict = segment.segmentDict
        updatedDict["text"] = commentText

        do {
            let jsonData = try JSONSerialization.data(withJSONObject: updatedDict)
            if let jsonString = String(data: jsonData, encoding: .utf8) {
                appState.pushUndo(plan.planJSON, label: "Edit Comment")
                try plan.updateSegment(jsonString, at: segment.index, inDayAt: segment.dayIndex)
                isEditing = false
                isTextFocused = false
            }
        } catch {
            print("Failed to save comment edit: \(error)")
        }
    }

    private func loadCurrentValue() {
        commentText = segment.commentText ?? ""
    }
}
