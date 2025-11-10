import SwiftUI

struct MainWindowView: View {
    @EnvironmentObject var appState: AppState
    @Binding var document: WeightliftingDocument
    @Environment(\.undoManager) private var undoManager
    @State private var splitVisibility: NavigationSplitViewVisibility = .all

    private let sidebarWidth: CGFloat = 260

    var body: some View {
        ZStack {
            NavigationSplitView(columnVisibility: $splitVisibility) {
                ExerciseSidebarView(plan: document.planDocument)
                    .environmentObject(appState)
                    .frame(minWidth: sidebarWidth, maxWidth: sidebarWidth)
                    .navigationSplitViewColumnWidth(sidebarWidth)
            } content: {
                CanvasView(plan: document.planDocument)
                    .environmentObject(appState)
                    .frame(minWidth: 520)
                    .layoutPriority(1)
                    .navigationSplitViewColumnWidth(min: 520, ideal: 0, max: .infinity)
            } detail: {
                InspectorView(plan: document.planDocument)
                    .environmentObject(appState)
                    .frame(minWidth: 320, idealWidth: 360, maxWidth: 440)
                    .navigationSplitViewColumnWidth(min: 320, ideal: 360, max: 440)
            }

            // Hidden buttons to capture Tab navigation
            VStack {
                Button("") {
                    appState.shouldFocusInspector = true
                }
                .keyboardShortcut(.tab, modifiers: [])
                .hidden()
                .frame(width: 0, height: 0)

                Button("") {
                    appState.shouldFocusCanvas = true
                }
                .keyboardShortcut(.tab, modifiers: [.shift])
                .hidden()
                .frame(width: 0, height: 0)
            }
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
        .sheet(item: $appState.previewToken) { token in
            SegmentPreviewSheet(plan: document.planDocument, token: token)
        }
        .sheet(isPresented: $appState.showDayEditor) {
            DayEditorView(plan: document.planDocument)
                .environmentObject(appState)
        }
        .onAppear {
            appState.activePlan = document.planDocument
        }
        .onDisappear {
            if appState.activePlan === document.planDocument {
                appState.activePlan = nil
            }
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
                appState.pushUndo(document.planDocument.planJSON, label: "Edit Segment")
                try document.planDocument.updateSegment(
                    segmentJSON,
                    at: segmentIndex,
                    inDayAt: dayIndex
                )
            } else {
                // Add new segment
                appState.pushUndo(document.planDocument.planJSON, label: "Add Segment")
                try document.planDocument.addSegment(segmentJSON, toDayAt: dayIndex)
                if document.planDocument.days.indices.contains(dayIndex) {
                    let newIndex = document.planDocument.days[dayIndex].segmentCount - 1
                    // Auto-select and scroll to new segment (Raskin: immediate feedback)
                    appState.selectIdentifier("\(dayIndex)_\(newIndex)")
                    appState.markRecentlyAddedSegment(dayIndex: dayIndex, segmentIndex: newIndex)
                }
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
