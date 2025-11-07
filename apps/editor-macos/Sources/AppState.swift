import SwiftUI
import Combine

/// Global application state
class AppState: ObservableObject {
    // UI State
    @Published var selectedDayIndex: Int?
    @Published var selectedSegmentIds: Set<String> = []
    @Published var focusedDayIndex: Int?

    // Dialog State
    @Published var showValidation = false
    @Published var showSegmentEditor = false
    @Published var showDayEditor = false
    @Published var showPlanEditor = false
    @Published var showGroupsEditor = false
    @Published var showExerciseSearch = false

    // Editing State
    @Published var editingSegmentJSON: String?
    @Published var editingSegmentIndex: Int?
    @Published var editingDayIndex: Int?

    // Validation
    @Published var validationErrors: [ValidationErrorInfo] = []

    // Preferences
    @Published var autosaveEnabled = true
    @Published var autosaveInterval: TimeInterval = 5.0

    // Undo/Redo (stores JSON snapshots)
    private var undoStack: [String] = []
    private var redoStack: [String] = []

    init() {
        // Initialize preferences from UserDefaults
        loadPreferences()
    }

    // MARK: - Selection Management

    func selectDay(_ index: Int) {
        selectedDayIndex = index
        selectedSegmentIndices.removeAll()
    }

    func selectSegment(_ index: Int, multiSelect: Bool = false, rangeSelect: Bool = false) {
        if multiSelect {
            if selectedSegmentIndices.contains(index) {
                selectedSegmentIndices.remove(index)
            } else {
                selectedSegmentIndices.insert(index)
            }
        } else if rangeSelect, let last = selectedSegmentIndices.max() {
            let range = min(last, index)...max(last, index)
            selectedSegmentIndices.formUnion(range)
        } else {
            selectedSegmentIndices = [index]
        }
    }

    func clearSelection() {
        selectedSegmentIndices.removeAll()
    }

    // MARK: - Editing Actions

    /// Edit a segment using its JSON representation
    func editSegmentJSON(_ json: String, at index: Int, in dayIndex: Int) {
        editingSegmentJSON = json
        editingSegmentIndex = index
        editingDayIndex = dayIndex
        showSegmentEditor = true
    }

    func addSegment(to dayIndex: Int) {
        editingSegmentJSON = nil
        editingSegmentIndex = nil
        editingDayIndex = dayIndex
        showSegmentEditor = true
    }

    // MARK: - Undo/Redo

    func pushUndo(_ planJSON: String) {
        undoStack.append(planJSON)
        redoStack.removeAll()
    }

    func undo() -> String? {
        guard let planJSON = undoStack.popLast() else { return nil }
        redoStack.append(planJSON)
        return planJSON
    }

    func redo() -> String? {
        guard let planJSON = redoStack.popLast() else { return nil }
        undoStack.append(planJSON)
        return planJSON
    }

    var canUndo: Bool {
        !undoStack.isEmpty
    }

    var canRedo: Bool {
        !redoStack.isEmpty
    }

    // MARK: - Preferences

    private func loadPreferences() {
        let defaults = UserDefaults.standard
        autosaveEnabled = defaults.bool(forKey: "autosaveEnabled")
        autosaveInterval = defaults.double(forKey: "autosaveInterval")

        if autosaveInterval == 0 {
            autosaveInterval = 5.0
        }
    }

    func savePreferences() {
        let defaults = UserDefaults.standard
        defaults.set(autosaveEnabled, forKey: "autosaveEnabled")
        defaults.set(autosaveInterval, forKey: "autosaveInterval")
    }
}
