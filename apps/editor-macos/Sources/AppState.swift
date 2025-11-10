import SwiftUI
import Combine

/// Global application state
class AppState: ObservableObject {
    // UI State
    @Published var selectedDayIndex: Int?
    @Published var selectedSegmentIds: Set<String> = []
    @Published var focusedDayIndex: Int?
    @Published var recentlyAddedSegmentID: String?
    @Published var shouldFocusInspector = false
    weak var activePlan: PlanDocument?

    // Dialog State
    @Published var showValidation = false
    @Published var showSegmentEditor = false
    @Published var showDayEditor = false
    @Published var showPlanEditor = false
    @Published var showGroupsEditor = false
    @Published var showExerciseSearch = false

    // Inline editing state
    @Published var inlineEditingSegmentId: String?
    @Published var focusedGroupName: String?
    enum SidebarTab {
        case exercises
        case groups
    }
    @Published var sidebarTab: SidebarTab = .exercises
    @Published var previewToken: SegmentPreviewToken?

    // Editing State
    @Published var editingSegmentJSON: String?
    @Published var editingSegmentIndex: Int?
    @Published var editingDayIndex: Int?

    // Validation
    @Published var validationErrors: [ValidationErrorInfo] = []

    // Preferences
    @Published var autosaveEnabled = true
    @Published var autosaveInterval: TimeInterval = 5.0

    // Undo/Redo (stores JSON snapshots with action labels)
    private var undoStack: [(json: String, label: String)] = []
    private var redoStack: [(json: String, label: String)] = []
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
            selectIdentifier(identifier)
        }
    }

    func clearSelection() {
        selectedSegmentIds.removeAll()
        lastSelectedCoordinate = nil
    }

    func selectionCoordinates() -> [SegmentCoordinate] {
        selectedSegmentIds.compactMap { SegmentCoordinate(identifier: $0) }
    }

    func primarySelectionCoordinate() -> SegmentCoordinate? {
        selectionCoordinates().sorted().first
    }

    func selectIdentifier(_ identifier: String) {
        selectedSegmentIds = [identifier]
        if let coord = SegmentCoordinate(identifier: identifier) {
            selectedDayIndex = coord.day
            lastSelectedCoordinate = (coord.day, coord.segment)
        }
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

    func addDay() {
        showDayEditor = true
    }

    // MARK: - Undo/Redo

    func pushUndo(_ planJSON: String, label: String = "Change") {
        undoStack.append((json: planJSON, label: label))
        redoStack.removeAll()
    }

    func performUndo(on plan: PlanDocument) {
        guard let previous = undoStack.popLast() else { return }
        redoStack.append((json: plan.planJSON, label: previous.label))
        plan.updatePlan(previous.json)
    }

    func performRedo(on plan: PlanDocument) {
        guard let next = redoStack.popLast() else { return }
        undoStack.append((json: plan.planJSON, label: next.label))
        plan.updatePlan(next.json)
    }

    var canUndo: Bool {
        !undoStack.isEmpty
    }

    var canRedo: Bool {
        !redoStack.isEmpty
    }

    var undoActionName: String? {
        undoStack.last?.label
    }

    var redoActionName: String? {
        redoStack.last?.label
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

    // MARK: - Segment Operations

    func duplicateSelectedSegment(in plan: PlanDocument) {
        guard let coordinate = primarySelectionCoordinate() else { return }
        pushUndo(plan.planJSON, label: "Duplicate Segment")
        do {
            try plan.duplicateSegment(at: coordinate.segment, inDayAt: coordinate.day)
            let newIndex = coordinate.segment + 1
            selectIdentifier("\(coordinate.day)_\(newIndex)")
            markRecentlyAddedSegment(dayIndex: coordinate.day, segmentIndex: newIndex)
        } catch {
            print("Failed to duplicate segment: \(error)")
        }
    }

    func deleteSelectedSegments(in plan: PlanDocument) {
        let coordinates = selectionCoordinates().sorted(by: >)
        guard !coordinates.isEmpty else { return }
        let label = coordinates.count == 1 ? "Delete Segment" : "Delete \(coordinates.count) Segments"
        pushUndo(plan.planJSON, label: label)
        for coordinate in coordinates {
            do {
                try plan.removeSegment(at: coordinate.segment, fromDayAt: coordinate.day)
            } catch {
                print("Failed to delete segment: \(error)")
            }
        }
        clearSelection()
    }

    func moveSelectedSegment(up: Bool, in plan: PlanDocument) {
        guard let coordinate = primarySelectionCoordinate() else { return }
        let segmentsCount = plan.days.indices.contains(coordinate.day) ? plan.days[coordinate.day].segmentCount : 0
        guard segmentsCount > 0 else { return }

        let targetIndex = up ? coordinate.segment - 1 : coordinate.segment + 1
        guard targetIndex >= 0, targetIndex < segmentsCount else { return }

        let label = up ? "Move Segment Up" : "Move Segment Down"
        pushUndo(plan.planJSON, label: label)
        do {
            try plan.moveSegment(inDayAt: coordinate.day, from: coordinate.segment, to: targetIndex)
            selectIdentifier("\(coordinate.day)_\(targetIndex)")
        } catch {
            print("Failed to move segment: \(error)")
        }
    }

    func previewSelectedSegment(in plan: PlanDocument) {
        guard let coordinate = primarySelectionCoordinate() else { return }
        previewToken = SegmentPreviewToken(dayIndex: coordinate.day, segmentIndex: coordinate.segment)
    }

    func focusGroup(named name: String) {
        focusedGroupName = name
        sidebarTab = .groups
    }

    func canDuplicateSelection() -> Bool {
        selectedSegmentIds.count == 1
    }

    func hasSelection() -> Bool {
        !selectedSegmentIds.isEmpty
    }

    func canMoveSelection(up: Bool, in plan: PlanDocument) -> Bool {
        guard let coordinate = primarySelectionCoordinate(),
              plan.days.indices.contains(coordinate.day) else { return false }
        let count = plan.days[coordinate.day].segmentCount
        if up {
            return coordinate.segment > 0
        } else {
            return coordinate.segment < count - 1
        }
    }

    func canPreviewSelection() -> Bool {
        primarySelectionCoordinate() != nil
    }

    func selectAdjacentSegment(delta: Int, in plan: PlanDocument) {
        let ordered = orderedCoordinates(in: plan)
        guard !ordered.isEmpty else { return }

        if let current = primarySelectionCoordinate(),
           let idx = ordered.firstIndex(of: current) {
            let newIndex = max(0, min(ordered.count - 1, idx + delta))
            selectIdentifier(ordered[newIndex].identifier)
        } else {
            let target = delta >= 0 ? 0 : ordered.count - 1
            selectIdentifier(ordered[target].identifier)
        }
    }

    func editSelectedSegment(in plan: PlanDocument) {
        // Focus the Inspector instead of opening the old dialog
        // The Inspector now provides inline editing for all segment types
        shouldFocusInspector = true
    }

    private func orderedCoordinates(in plan: PlanDocument) -> [SegmentCoordinate] {
        plan.days.enumerated().flatMap { dayIndex, day in
            day.segments().enumerated().map { segmentIndex, _ in
                SegmentCoordinate(day: dayIndex, segment: segmentIndex)
            }
        }
    }

    struct SegmentPreviewToken: Identifiable {
        let id = UUID()
        let dayIndex: Int
        let segmentIndex: Int
    }

    struct SegmentCoordinate: Comparable {
        let day: Int
        let segment: Int

        init(day: Int, segment: Int) {
            self.day = day
            self.segment = segment
        }

        init?(identifier: String) {
            let components = identifier.split(separator: "_")
            guard components.count == 2,
                  let day = Int(components[0]),
                  let segment = Int(components[1]) else {
                return nil
            }
            self.day = day
            self.segment = segment
        }

        var identifier: String { "\(day)_\(segment)" }

        static func < (lhs: SegmentCoordinate, rhs: SegmentCoordinate) -> Bool {
            if lhs.day == rhs.day {
                return lhs.segment < rhs.segment
            }
            return lhs.day < rhs.day
        }
    }
}
