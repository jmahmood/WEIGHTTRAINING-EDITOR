import SwiftUI
import UniformTypeIdentifiers
import Foundation

/// JSON-based plan document - stores plan as raw JSON string
/// This avoids the complexity of modeling the entire Rust structure in Swift
class PlanDocument: ObservableObject {
    @Published var planJSON: String
    @Published var isDirty = false

    private var parsedCache: [String: Any]?

    init(json: String = "") {
        if json.isEmpty {
            // Create new plan using FFI
            do {
                self.planJSON = try RustBridge.createNewPlanJSON()
            } catch {
                print("Failed to create new plan: \(error)")
                self.planJSON = PlanDocument.fallbackEmptyPlan()
            }
        } else {
            self.planJSON = json
        }
    }

    // MARK: - Parsed Properties

    /// Lazily parsed JSON
    private var parsed: [String: Any]? {
        if parsedCache == nil {
            parsedCache = try? JSONSerialization.jsonObject(
                with: Data(planJSON.utf8)
            ) as? [String: Any]
        }
        return parsedCache
    }

    /// Clear cache when JSON changes
    func invalidateCache() {
        parsedCache = nil
    }

    // MARK: - Plan Properties

    var name: String {
        parsed?["name"] as? String ?? "Untitled Plan"
    }

    var author: String? {
        parsed?["author"] as? String
    }

    var unit: String {
        parsed?["unit"] as? String ?? "kg"
    }

    var dictionary: [String: String] {
        parsed?["dictionary"] as? [String: String] ?? [:]
    }

    var groups: [String: [String]] {
        parsed?["groups"] as? [String: [String]] ?? [:]
    }

    /// Get display models for days
    var days: [DayDisplay] {
        guard let schedule = parsed?["schedule"] as? [[String: Any]] else {
            return []
        }

        return schedule.enumerated().compactMap { index, dayDict in
            DayDisplay(index: index, dict: dayDict, parent: self)
        }
    }

    // MARK: - Mutations

    /// Update the entire plan JSON
    func updatePlan(_ newJSON: String) {
        planJSON = newJSON
        invalidateCache()
        isDirty = true
        objectWillChange.send()
    }

    /// Add a segment to a day
    func addSegment(_ segmentJSON: String, toDayAt dayIndex: Int) throws {
        let updatedJSON = try RustBridge.addSegment(segmentJSON, toDayAt: dayIndex, in: planJSON)
        updatePlan(updatedJSON)
    }

    /// Remove a segment
    func removeSegment(at segmentIndex: Int, fromDayAt dayIndex: Int) throws {
        let updatedJSON = try RustBridge.removeSegment(at: segmentIndex, fromDayAt: dayIndex, in: planJSON)
        updatePlan(updatedJSON)
    }

    /// Update a segment
    func updateSegment(_ segmentJSON: String, at segmentIndex: Int, inDayAt dayIndex: Int) throws {
        let updatedJSON = try RustBridge.updateSegment(
            segmentJSON,
            at: segmentIndex,
            inDayAt: dayIndex,
            in: planJSON
        )
        updatePlan(updatedJSON)
    }

    /// Add a day
    func addDay(_ dayJSON: String) throws {
        let updatedJSON = try RustBridge.addDay(dayJSON, to: planJSON)
        updatePlan(updatedJSON)
    }

    /// Remove a day
    func removeDay(at index: Int) throws {
        let updatedJSON = try RustBridge.removeDay(at: index, from: planJSON)
        updatePlan(updatedJSON)
    }

    /// Add an exercise entry to the dictionary
    func addExercise(code: String, name: String) throws {
        let updatedJSON = try RustBridge.addExercise(code: code, name: name, to: planJSON)
        updatePlan(updatedJSON)
    }

    /// Validate the plan
    func validate() throws -> ValidationResult {
        try RustBridge.validatePlan(planJSON)
    }

    // MARK: - File I/O

    func load(from url: URL) throws {
        let json = try String(contentsOf: url, encoding: .utf8)
        updatePlan(json)
        isDirty = false
    }

    func save(to url: URL) throws {
        try planJSON.write(to: url, atomically: true, encoding: .utf8)
        isDirty = false
    }

    // MARK: - Helpers

    private static func fallbackEmptyPlan() -> String {
        """
        {
          "name": "New Plan",
          "unit": "kg",
          "dictionary": {},
          "groups": {},
          "schedule": []
        }
        """
    }
}

// MARK: - Display Models

/// Lightweight model for displaying a day
struct DayDisplay: Identifiable {
    let id: Int  // Index in schedule
    let dayNumber: UInt32
    let label: String
    let goal: String?
    let timeCapMin: UInt32?
    let segmentCount: Int

    weak var parent: PlanDocument?
    private let dayDict: [String: Any]

    init?(index: Int, dict: [String: Any], parent: PlanDocument) {
        guard let label = dict["label"] as? String else {
            return nil
        }

        self.id = index
        self.dayNumber = dict["day"] as? UInt32 ?? UInt32(index + 1)
        self.label = label
        self.goal = dict["goal"] as? String
        self.timeCapMin = dict["time_cap_min"] as? UInt32
        self.dayDict = dict
        self.parent = parent

        if let segments = dict["segments"] as? [[String: Any]] {
            self.segmentCount = segments.count
        } else {
            self.segmentCount = 0
        }
    }

    /// Get segments for this day
    func segments() -> [SegmentDisplay] {
        guard let segmentsArray = dayDict["segments"] as? [[String: Any]] else {
            return []
        }

        return segmentsArray.enumerated().compactMap { index, segDict in
            SegmentDisplay(index: index, dayIndex: id, dict: segDict, parent: parent)
        }
    }
}

/// Lightweight model for displaying a segment
struct SegmentDisplay: Identifiable {
    let id: String  // Computed from day + segment index
    let index: Int
    let dayIndex: Int
    let type: String
    let displayText: String
    let icon: String
    let color: String

    weak var parent: PlanDocument?
    let segmentDict: [String: Any]

    init?(index: Int, dayIndex: Int, dict: [String: Any], parent: PlanDocument?) {
        guard let type = dict["type"] as? String else {
            return nil
        }

        self.id = "\(dayIndex)_\(index)"
        self.index = index
        self.dayIndex = dayIndex
        self.type = type
        self.segmentDict = dict
        self.parent = parent

        // Helper to format reps/reps_range
        func formatReps(_ dict: [String: Any]) -> String {
            // Check if reps is an object with min/max
            if let repsObj = dict["reps"] as? [String: Any],
               let min = repsObj["min"] as? Int,
               let max = repsObj["max"] as? Int {
                return "\(min)-\(max)"
            }

            // Check for simple reps as Int
            if let reps = dict["reps"] as? Int {
                return "\(reps)"
            }

            // Handle NSNumber (in case JSON parsing returns this)
            if let reps = dict["reps"] as? NSNumber {
                return "\(reps.intValue)"
            }

            return "?"
        }

        // Helper to get exercise display name
        func getExerciseName(_ code: String) -> String {
            if let dictionary = parent?.dictionary,
               let name = dictionary[code] {
                return name
            }
            return code
        }

        // Generate display text based on type
        switch type {
        case "straight":
            let exCode = dict["ex"] as? String ?? "Unknown"
            let ex = getExerciseName(exCode)
            let sets = dict["sets"] as? Int ?? 0
            let reps = formatReps(dict)
            self.displayText = "\(ex) • \(sets) × \(reps)"
            self.icon = "figure.strengthtraining.traditional"
            self.color = "blue"

        case "rpe":
            let exCode = dict["ex"] as? String ?? "Unknown"
            let ex = getExerciseName(exCode)
            let sets = dict["sets"] as? Int ?? 0
            let reps = formatReps(dict)
            let rpe = dict["rpe"] as? Double ?? 0
            self.displayText = "\(ex) • \(sets) × \(reps) @ RPE \(rpe)"
            self.icon = "gauge"
            self.color = "orange"

        case "percentage":
            let exCode = dict["ex"] as? String ?? "Unknown"
            let ex = getExerciseName(exCode)
            let sets = dict["sets"] as? Int ?? 0
            let reps = formatReps(dict)
            let pct = dict["percentage"] as? Double ?? 0
            self.displayText = "\(ex) • \(sets) × \(reps) @ \(Int(pct))%"
            self.icon = "percent"
            self.color = "purple"

        case "amrap":
            let exCode = dict["ex"] as? String ?? "Unknown"
            let ex = getExerciseName(exCode)
            self.displayText = "\(ex) • AMRAP"
            self.icon = "flame"
            self.color = "red"

        case "superset":
            if let label = dict["label"] as? String {
                let rounds = dict["rounds"] as? Int ?? 0
                self.displayText = "\(label) • \(rounds) rounds"
            } else if let exercises = dict["exercises"] as? [[String: Any]], !exercises.isEmpty {
                let exNames = exercises.compactMap { ex in
                    (ex["ex"] as? String).map { getExerciseName($0) }
                }.joined(separator: " + ")
                let rounds = dict["rounds"] as? Int ?? 0
                self.displayText = "\(exNames) • \(rounds) rounds"
            } else {
                let rounds = dict["rounds"] as? Int ?? 0
                self.displayText = "Superset • \(rounds) rounds"
            }
            self.icon = "arrow.triangle.2.circlepath"
            self.color = "green"

        case "circuit":
            if let label = dict["label"] as? String {
                let rounds = dict["rounds"] as? Int ?? 0
                self.displayText = "\(label) • \(rounds) rounds"
            } else if let exercises = dict["exercises"] as? [[String: Any]], !exercises.isEmpty {
                let exNames = exercises.compactMap { ex in
                    (ex["ex"] as? String).map { getExerciseName($0) }
                }.joined(separator: " + ")
                let rounds = dict["rounds"] as? Int ?? 0
                self.displayText = "\(exNames) • \(rounds) rounds"
            } else {
                let rounds = dict["rounds"] as? Int ?? 0
                self.displayText = "Circuit • \(rounds) rounds"
            }
            self.icon = "circle.grid.cross"
            self.color = "teal"

        case "scheme":
            let exCode = dict["ex"] as? String ?? "Unknown"
            let ex = getExerciseName(exCode)

            // Count total sets in the scheme
            var totalSets = 0
            if let setsArray = dict["sets"] as? [[String: Any]] {
                for setInfo in setsArray {
                    let numSets = setInfo["sets"] as? Int ?? 1
                    totalSets += numSets
                }
            }

            // Show set count
            if totalSets > 0 {
                self.displayText = "\(ex) • \(totalSets) sets"
            } else {
                self.displayText = "\(ex) • scheme"
            }
            self.icon = "list.number"
            self.color = "indigo"

        case "complex":
            let label = dict["label"] as? String
            let sets = dict["sets"] as? Int ?? 0
            self.displayText = label ?? "Complex • \(sets) sets"
            self.icon = "arrow.right.circle"
            self.color = "pink"

        case "comment":
            let text = dict["text"] as? String ?? ""
            self.displayText = text
            self.icon = "text.bubble"
            self.color = "gray"

        default:
            self.displayText = type.capitalized
            self.icon = "questionmark.circle"
            self.color = "gray"
        }
    }

    /// Get the full JSON for editing
    func toJSON() -> String? {
        guard let data = try? JSONSerialization.data(withJSONObject: segmentDict, options: .prettyPrinted),
              let json = String(data: data, encoding: .utf8) else {
            return nil
        }
        return json
    }
}

// MARK: - Validation Result

struct ValidationResult: Codable {
    let errors: [ValidationErrorInfo]
    let warnings: [ValidationErrorInfo]

    var isValid: Bool {
        errors.isEmpty
    }
}

struct ValidationErrorInfo: Codable, Identifiable {
    let error: String
    let path: String
    let context: String?
    let message: String?

    var id: String { "\(path):\(error)" }

    var displayMessage: String {
        message ?? error
    }
}
