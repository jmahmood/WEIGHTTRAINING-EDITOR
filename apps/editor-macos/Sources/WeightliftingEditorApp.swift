import SwiftUI

@main
struct WeightliftingEditorApp: App {
    @StateObject private var appStateStorage = AppState()
    private var appState: AppState { appStateStorage }

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
                Button("Undo") {
                    if let plan = appState.activePlan {
                        appState.performUndo(on: plan)
                    }
                }
                .keyboardShortcut("z", modifiers: .command)
                .disabled(!appState.canUndo)

                Button("Redo") {
                    if let plan = appState.activePlan {
                        appState.performRedo(on: plan)
                    }
                }
                .keyboardShortcut("Z", modifiers: [.command, .shift])
                .disabled(!appState.canRedo)
            }

            CommandGroup(replacing: .help) {
                Button("Weightlifting Editor Help") {
                    // Open help
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
