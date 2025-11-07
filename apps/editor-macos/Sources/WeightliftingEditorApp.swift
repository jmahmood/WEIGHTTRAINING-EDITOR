import SwiftUI

@main
struct WeightliftingEditorApp: App {
    @StateObject private var appState = AppState()

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
