import SwiftUI
import Combine

/// Global application state
class AppState: ObservableObject {
    // UI State
    @Published var selectedDayIndex: Int?
    @Published var selectedSegmentIds: Set<String> = []
    @Published var focusedDayIndex: Int?
    @Published var recentlyAddedSegmentID: String?
    weak var activePlan: PlanDocument?

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
    private var lastSelectedCoordinate: (day: Int, index: Int)?

    init() {
        // Initialize preferences from UserDefaults
        loadPreferences()
    }

    // MARK: - Selection Management

    func selectDay(_ index: Int) {
        selectedDayIndex = index
        clearSelection()
    }

    func selectSegment(_ segment: SegmentDisplay, in plan: PlanDocument, multiSelect: Bool = false, rangeSelect: Bool = false) {
        let identifier = segment.id
        if multiSelect {
            if selectedSegmentIds.contains(identifier) {
                selectedSegmentIds.remove(identifier)
            } else {
                selectedSegmentIds.insert(identifier)
            }
            lastSelectedCoordinate = (segment.dayIndex, segment.index)
        } else if rangeSelect, let last = lastSelectedCoordinate, last.day == segment.dayIndex {
            let segments = plan.days[segment.dayIndex].segments()
            let lower = min(last.index, segment.index)
            let upper = max(last.index, segment.index)
            guard lower >= 0, upper < segments.count else {
                selectedSegmentIds = [identifier]
                lastSelectedCoordinate = (segment.dayIndex, segment.index)
                return
            }
            let rangeIDs = segments[lower...upper].map(\.id)
            selectedSegmentIds.formUnion(rangeIDs)
            lastSelectedCoordinate = (segment.dayIndex, segment.index)
        } else {
            selectedSegmentIds = [identifier]
            lastSelectedCoordinate = (segment.dayIndex, segment.index)
        }
    }

    func clearSelection() {
        selectedSegmentIds.removeAll()
        lastSelectedCoordinate = nil
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

    func performUndo(on plan: PlanDocument) {
        guard let previous = undoStack.popLast() else { return }
        redoStack.append(plan.planJSON)
        plan.updatePlan(previous)
    }

    func performRedo(on plan: PlanDocument) {
        guard let redoJSON = redoStack.popLast() else { return }
        undoStack.append(plan.planJSON)
        plan.updatePlan(redoJSON)
    }

    var canUndo: Bool {
        !undoStack.isEmpty
    }

    var canRedo: Bool {
        !redoStack.isEmpty
    }

    func markRecentlyAddedSegment(dayIndex: Int, segmentIndex: Int) {
        let identifier = "\(dayIndex)_\(segmentIndex)"
        recentlyAddedSegmentID = identifier
        DispatchQueue.main.asyncAfter(deadline: .now() + 1.5) { [weak self] in
            if self?.recentlyAddedSegmentID == identifier {
                self?.recentlyAddedSegmentID = nil
            }
        }
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
