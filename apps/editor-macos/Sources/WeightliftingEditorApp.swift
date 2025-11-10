import SwiftUI

@main
struct WeightliftingEditorApp: App {
    @StateObject private var appStateStorage = AppState()
    private var appState: AppState { appStateStorage }
    @FocusedValue(\.segmentActions) private var segmentActions

    var body: some Scene {
        DocumentGroup(newDocument: WeightliftingDocument()) { file in
            MainWindowView(document: file.$document)
                .environmentObject(appState)
        }
        .commands {
            CommandGroup(after: .newItem) {
                Button("Validate Plan...") {
                    appState.showValidation = true
                }
                .keyboardShortcut("v", modifiers: [.command])
            }

            CommandGroup(replacing: .undoRedo) {
                Button(appState.undoActionName.map { "Undo \($0)" } ?? "Undo") {
                    if let plan = appState.activePlan {
                        appState.performUndo(on: plan)
                    }
                }
                .keyboardShortcut("z", modifiers: .command)
                .disabled(!appState.canUndo)

                Button(appState.redoActionName.map { "Redo \($0)" } ?? "Redo") {
                    if let plan = appState.activePlan {
                        appState.performRedo(on: plan)
                    }
                }
                .keyboardShortcut("Z", modifiers: [.command, .shift])
                .disabled(!appState.canRedo)
            }

            CommandMenu("Segments") {
                if let plan = appState.activePlan {
                    Button("Edit Segment") {
                        segmentActions?.edit()
                    }
                    .keyboardShortcut(.return)
                    .disabled(segmentActions == nil)

                    Button("Preview Segment") {
                        segmentActions?.preview()
                    }
                    .keyboardShortcut(.space)
                    .disabled(segmentActions == nil)

                    Button("Duplicate Segment") {
                        appState.duplicateSelectedSegment(in: plan)
                    }
                    .keyboardShortcut("d", modifiers: .command)
                    .disabled(!appState.canDuplicateSelection())

                    Button("Move Segment Up") {
                        appState.moveSelectedSegment(up: true, in: plan)
                    }
                    .keyboardShortcut(.upArrow, modifiers: .command)
                    .disabled(!appState.canMoveSelection(up: true, in: plan))

                    Button("Move Segment Down") {
                        appState.moveSelectedSegment(up: false, in: plan)
                    }
                    .keyboardShortcut(.downArrow, modifiers: .command)
                    .disabled(!appState.canMoveSelection(up: false, in: plan))

                    Button("Delete Segment") {
                        appState.deleteSelectedSegments(in: plan)
                    }
                    .keyboardShortcut(.delete, modifiers: [])
                    .disabled(!appState.hasSelection())
                } else {
                    Button("Edit Segment") {}.disabled(true)
                    Button("Preview Segment") {}.disabled(true)
                    Button("Duplicate Segment") {}.disabled(true)
                    Button("Move Segment Up") {}.disabled(true)
                    Button("Move Segment Down") {}.disabled(true)
                    Button("Delete Segment") {}.disabled(true)
                }
            }

            CommandGroup(replacing: .help) {
                Button("Weightlifting Editor Help") {
                    // Open help
                }
                Button("View Error Log...") {
                    NSWorkspace.shared.selectFile(ErrorLogger.shared.getLogFilePath(), inFileViewerRootedAtPath: "")
                }
            }
        }

        // Settings window
        Settings {
            PreferencesView()
                .environmentObject(appState)
        }
    }
}
