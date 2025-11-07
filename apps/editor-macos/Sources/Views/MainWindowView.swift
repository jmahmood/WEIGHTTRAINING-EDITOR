import SwiftUI

struct MainWindowView: View {
    @EnvironmentObject var appState: AppState
    @Binding var document: WeightliftingDocument

    var body: some View {
        HSplitView {
            // Canvas (main editing area)
            CanvasView(plan: document.planDocument)
                .frame(minWidth: 400, idealWidth: 600)

            // Right panel (exercises and groups)
            RightPanelView(plan: document.planDocument)
                .frame(minWidth: 250, idealWidth: 300, maxWidth: 400)
        }
        .navigationTitle(document.planDocument.name)
        .navigationSubtitle(documentSubtitle)
        .toolbar {
            ToolbarView(
                planName: document.planDocument.name,
                onValidate: { validatePlan() }
            )
        }
        .sheet(isPresented: $appState.showSegmentEditor) {
            SegmentEditorView(
                plan: document.planDocument,
                segmentJSON: appState.editingSegmentJSON,
                dayIndex: appState.editingDayIndex ?? 0,
                onSave: { json in
                    saveSegment(json)
                }
            )
        }
        .sheet(isPresented: $appState.showValidation) {
            ValidationView(errors: appState.validationErrors)
        }
        .sheet(isPresented: $appState.showGroupsEditor) {
            GroupsEditorView(
                plan: document.planDocument,
                onSave: { /* Groups are updated directly via FFI */ }
            )
        }
    }

    // MARK: - Computed Properties

    private var documentSubtitle: String {
        // Try to get author info
        if let author = document.planDocument.author, !author.isEmpty {
            return author
        }
        return ""
    }

    // MARK: - Actions

    private func validatePlan() {
        do {
            let result = try document.planDocument.validate()
            appState.validationErrors = result.errors + result.warnings
            appState.showValidation = true
        } catch {
            print("Validation error: \(error)")
            // Show error to user
        }
    }

    private func saveSegment(_ segmentJSON: String) {
        guard let dayIndex = appState.editingDayIndex else { return }

        do {
            if let segmentIndex = appState.editingSegmentIndex {
                // Update existing segment
                try document.planDocument.updateSegment(
                    segmentJSON,
                    at: segmentIndex,
                    inDayAt: dayIndex
                )
            } else {
                // Add new segment
                try document.planDocument.addSegment(segmentJSON, toDayAt: dayIndex)
            }
            appState.showSegmentEditor = false
        } catch {
            print("Failed to save segment: \(error)")
            // Show error to user
        }
    }
}

struct ToolbarView: View {
    let planName: String
    let onValidate: () -> Void

    var body: some View {
        HStack {
            Button(action: onValidate) {
                Label("Validate", systemImage: "checkmark.circle")
            }
            .keyboardShortcut("v", modifiers: .command)

            Spacer()

            Text(planName)
                .font(.headline)
        }
    }
}

struct PreferencesView: View {
    @EnvironmentObject var appState: AppState

    var body: some View {
        Form {
            Section("Autosave") {
                Toggle("Enable Autosave", isOn: $appState.autosaveEnabled)

                HStack {
                    Text("Interval (seconds):")
                    TextField("", value: $appState.autosaveInterval, format: .number)
                        .frame(width: 60)
                }
                .disabled(!appState.autosaveEnabled)
            }
        }
        .padding()
        .frame(width: 400, height: 200)
        .onChange(of: appState.autosaveEnabled) { _ in
            appState.savePreferences()
        }
        .onChange(of: appState.autosaveInterval) { _ in
            appState.savePreferences()
        }
    }
}
