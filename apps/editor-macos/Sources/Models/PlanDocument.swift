import SwiftUI
import UniformTypeIdentifiers
import Foundation

/// JSON-based plan document - stores plan as raw JSON string
/// This avoids the complexity of modeling the entire Rust structure in Swift
enum PlanMutationError: Error {
    case invalidStructure
    case serializationFailed
}

struct ProgressionSettings: Equatable {
    var mode: String
    var repsFirst: Bool
    var loadIncrementKg: Double?
    var capRpe: Double?

    var isEmpty: Bool {
        mode.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty
    }

    func toDictionary() -> [String: Any]? {
        let trimmed = mode.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmed.isEmpty else { return nil }
        var output: [String: Any] = [
            "mode": trimmed,
            "reps_first": repsFirst
        ]
        if let loadIncrementKg = loadIncrementKg {
            output["load_increment_kg"] = loadIncrementKg
        }
        if let capRpe = capRpe {
            output["cap_rpe"] = capRpe
        }
        return output
    }
}

class PlanDocument: ObservableObject {
    private enum Defaults {
        static let restSeconds = 90
        static let rpe: Double = 8.5
    }
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
        let raw = parsed?["name"] as? String ?? "Untitled Plan"
        if raw.lowercased().hasSuffix(".json") {
            return String(raw.dropLast(5))
        }
        return raw
    }

    var author: String? {
        parsed?["author"] as? String
    }

    var unit: String {
        parsed?["unit"] as? String ?? "kg"
    }

    func getProgressionSettings() -> ProgressionSettings? {
        guard let progression = parsed?["progression"] as? [String: Any] else {
            return nil
        }

        let mode = progression["mode"] as? String ?? ""
        let repsFirst = progression["reps_first"] as? Bool ?? false
        let loadIncrementKg = (progression["load_increment_kg"] as? NSNumber)?.doubleValue
        let capRpe = (progression["cap_rpe"] as? NSNumber)?.doubleValue

        return ProgressionSettings(
            mode: mode,
            repsFirst: repsFirst,
            loadIncrementKg: loadIncrementKg,
            capRpe: capRpe
        )
    }

    var dictionary: [String: String] {
        parsed?["dictionary"] as? [String: String] ?? [:]
    }

    var groups: [String: [String]] {
        parsed?["groups"] as? [String: [String]] ?? [:]
    }

    func getGroupVariants() -> [String: [String: [String: [String: JSONValue]]]] {
        guard let variantsAny = parsed?["group_variants"] as? [String: Any] else {
            return [:]
        }

        var result: [String: [String: [String: [String: JSONValue]]]] = [:]

        for (groupId, rolesAny) in variantsAny {
            guard let rolesDict = rolesAny as? [String: Any] else { continue }
            var roles: [String: [String: [String: JSONValue]]] = [:]

            for (roleId, exercisesAny) in rolesDict {
                guard let exercisesDict = exercisesAny as? [String: Any] else { continue }
                var exercises: [String: [String: JSONValue]] = [:]

                for (exerciseCode, overridesAny) in exercisesDict {
                    guard let overridesDict = overridesAny as? [String: Any] else { continue }
                    var overrides: [String: JSONValue] = [:]

                    for (key, valueAny) in overridesDict {
                        if let jsonValue = JSONValue(any: valueAny) {
                            overrides[key] = jsonValue
                        }
                    }

                    exercises[exerciseCode] = overrides
                }

                roles[roleId] = exercises
            }

            result[groupId] = roles
        }

        return result
    }

    func getRolesForGroup(_ groupId: String) -> [String] {
        guard groups[groupId] != nil else { return [] }
        return ["strength", "volume", "endurance"]
    }

    func firstGroupContaining(exerciseCode: String) -> String? {
        let groupNames = groups.keys.sorted()
        for name in groupNames {
            if let exercises = groups[name], exercises.contains(exerciseCode) {
                return name
            }
        }
        return nil
    }

    func getExerciseMeta() -> [String: ExerciseMeta] {
        guard let metaAny = parsed?["exercise_meta"] as? [String: Any] else {
            return [:]
        }

        var result: [String: ExerciseMeta] = [:]

        for (exerciseCode, metaValue) in metaAny {
            guard let metaDict = metaValue as? [String: Any] else { continue }
            var loadAxes: [String: LoadAxis] = [:]
            var roleReps: [String: RoleRepsRange] = [:]

            if let loadAxesAny = metaDict["load_axes"] as? [String: Any] {
                for (axisName, axisValue) in loadAxesAny {
                    guard let axisDict = axisValue as? [String: Any] else { continue }
                    guard let kindRaw = axisDict["kind"] as? String,
                          let kind = LoadAxisKind(rawValue: kindRaw) else { continue }

                    let valuesAny = axisDict["values"] as? [Any] ?? []
                    let values = valuesAny.compactMap { value -> String? in
                        if let string = value as? String {
                            return string
                        }
                        if let number = value as? NSNumber {
                            return number.stringValue
                        }
                        return nil
                    }

                    loadAxes[axisName] = LoadAxis(kind: kind, values: values)
                }
            }

            if let roleRepsAny = metaDict["role_reps"] as? [String: Any] {
                for (roleName, repsValue) in roleRepsAny {
                    guard let repsDict = repsValue as? [String: Any],
                          let min = repsDict["min"] as? Int,
                          let max = repsDict["max"] as? Int else { continue }
                    roleReps[roleName] = RoleRepsRange(min: min, max: max)
                }
            }

            result[exerciseCode] = ExerciseMeta(loadAxes: loadAxes, roleReps: roleReps)
        }

        return result
    }

    func getLoadAxesForExercise(_ exerciseCode: String) -> [String: LoadAxis] {
        getExerciseMeta()[exerciseCode]?.loadAxes ?? [:]
    }

    func getRoleRepsDefaults(for exerciseCode: String) -> [String: RoleRepsRange] {
        getExerciseMeta()[exerciseCode]?.roleReps ?? [:]
    }

    func updateExerciseRoleDefaults(for exerciseCode: String, roleReps: [String: RoleRepsRange]) {
        let existingMeta = getExerciseMeta()
        let oldDefaults = existingMeta[exerciseCode]?.roleReps ?? [:]

        var updatedMeta = existingMeta
        var entry = updatedMeta[exerciseCode] ?? ExerciseMeta(loadAxes: [:], roleReps: [:])
        entry.roleReps = roleReps
        updatedMeta[exerciseCode] = entry
        updateExerciseMeta(updatedMeta)

        var variants = getGroupVariants()
        let roles = ["strength", "volume", "endurance"]

        for (groupId, _) in variants {
            for role in roles {
                let oldDefault = oldDefaults[role]
                let newDefault = roleReps[role]

                if oldDefault == nil && newDefault == nil {
                    continue
                }
                if oldDefault?.min == newDefault?.min && oldDefault?.max == newDefault?.max {
                    continue
                }

                let override = variants[groupId]?[role]?[exerciseCode]
                var overrideMin: Int?
                var overrideMax: Int?

                if let override = override, case .object(let repsObj)? = override["reps"] {
                    overrideMin = repsObj["min"]?.intValue
                    overrideMax = repsObj["max"]?.intValue
                }

                let matchesOldDefault = {
                    guard let oldDefault = oldDefault,
                          let overrideMin = overrideMin,
                          let overrideMax = overrideMax else {
                        return false
                    }
                    return overrideMin == oldDefault.min && overrideMax == oldDefault.max
                }()

                let isNullish = (overrideMin ?? 0) <= 0 || (overrideMax ?? 0) <= 0

                let shouldPropagate = override == nil || matchesOldDefault || isNullish

                if shouldPropagate {
                    if let newDefault = newDefault {
                        if variants[groupId] == nil { variants[groupId] = [:] }
                        if variants[groupId]?[role] == nil { variants[groupId]?[role] = [:] }
                        if variants[groupId]?[role]?[exerciseCode] == nil { variants[groupId]?[role]?[exerciseCode] = [:] }
                        variants[groupId]?[role]?[exerciseCode]?["reps"] = .object([
                            "min": .number(Double(newDefault.min)),
                            "max": .number(Double(newDefault.max))
                        ])
                    } else {
                        variants[groupId]?[role]?[exerciseCode]?["reps"] = nil
                        if variants[groupId]?[role]?[exerciseCode]?.isEmpty == true {
                            variants[groupId]?[role]?.removeValue(forKey: exerciseCode)
                        }
                        if variants[groupId]?[role]?.isEmpty == true {
                            variants[groupId]?.removeValue(forKey: role)
                        }
                        if variants[groupId]?.isEmpty == true {
                            variants.removeValue(forKey: groupId)
                        }
                    }
                }
            }
        }

        updateGroupVariants(variants)
    }

    func ensureGroupRoleExists(groupId: String, roleId: String) {
        guard !groupId.isEmpty, !roleId.isEmpty else { return }
        do {
            try mutatePlanDictionary { root in
                var variants = root["group_variants"] as? [String: Any] ?? [:]
                var group = variants[groupId] as? [String: Any] ?? [:]
                if group[roleId] == nil {
                    group[roleId] = [:]
                    variants[groupId] = group
                    root["group_variants"] = variants
                }
            }
        } catch {
            print("Failed to ensure group role: \(error)")
        }
    }

    func updateGroupVariants(_ variants: [String: [String: [String: [String: JSONValue]]]]) {
        do {
            try mutatePlanDictionary { root in
                guard !variants.isEmpty else {
                    root.removeValue(forKey: "group_variants")
                    return
                }

                var output: [String: Any] = [:]
                for (groupId, roles) in variants {
                    var rolesOut: [String: Any] = [:]
                    for (roleId, exercises) in roles {
                        var exercisesOut: [String: Any] = [:]
                        for (exerciseCode, overrides) in exercises {
                            var overridesOut: [String: Any] = [:]
                            for (key, value) in overrides {
                                overridesOut[key] = value.toAny()
                            }
                            exercisesOut[exerciseCode] = overridesOut
                        }
                        rolesOut[roleId] = exercisesOut
                    }
                    output[groupId] = rolesOut
                }

                root["group_variants"] = output
            }
        } catch {
            print("Failed to update group_variants: \(error)")
        }
    }

    func updateExerciseMeta(_ meta: [String: ExerciseMeta]) {
        do {
            try mutatePlanDictionary { root in
                guard !meta.isEmpty else {
                    root.removeValue(forKey: "exercise_meta")
                    return
                }

                var output: [String: Any] = [:]
                for (exerciseCode, metaValue) in meta {
                    guard !metaValue.loadAxes.isEmpty || !metaValue.roleReps.isEmpty else { continue }
                    var axesOut: [String: Any] = [:]
                    for (axisName, axis) in metaValue.loadAxes {
                        axesOut[axisName] = axis.toDictionary()
                    }
                    var exerciseOut: [String: Any] = [:]
                    if !axesOut.isEmpty {
                        exerciseOut["load_axes"] = axesOut
                    }
                    if !metaValue.roleReps.isEmpty {
                        var roleOut: [String: Any] = [:]
                        for (roleName, reps) in metaValue.roleReps {
                            roleOut[roleName] = ["min": reps.min, "max": reps.max]
                        }
                        exerciseOut["role_reps"] = roleOut
                    }
                    if !exerciseOut.isEmpty {
                        output[exerciseCode] = exerciseOut
                    }
                }

                if output.isEmpty {
                    root.removeValue(forKey: "exercise_meta")
                } else {
                    root["exercise_meta"] = output
                }
            }
        } catch {
            print("Failed to update exercise_meta: \(error)")
        }
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

    func updateSegmentFields(dayIndex: Int, segmentIndex: Int, updates: [String: Any]) throws {
        try mutatePlanDictionary { root in
            guard var schedule = root["schedule"] as? [[String: Any]],
                  schedule.indices.contains(dayIndex),
                  var segments = schedule[dayIndex]["segments"] as? [[String: Any]],
                  segments.indices.contains(segmentIndex) else {
                throw PlanMutationError.invalidStructure
            }

            var segment = segments[segmentIndex]
            for (key, value) in updates {
                segment[key] = value
            }
            segments[segmentIndex] = segment
            schedule[dayIndex]["segments"] = segments
            root["schedule"] = schedule
        }
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

    /// Duplicate a segment by copying its JSON within the same day
    func duplicateSegment(at segmentIndex: Int, inDayAt dayIndex: Int) throws {
        try mutatePlanDictionary { root in
            guard var schedule = root["schedule"] as? [[String: Any]], dayIndex < schedule.count else {
                throw PlanMutationError.invalidStructure
            }

            var day = schedule[dayIndex]
            guard var segments = day["segments"] as? [[String: Any]], segmentIndex < segments.count else {
                throw PlanMutationError.invalidStructure
            }

            let segment = segments[segmentIndex]
            let insertIndex = min(segmentIndex + 1, segments.count)
            segments.insert(segment, at: insertIndex)

            day["segments"] = segments
            schedule[dayIndex] = day
            root["schedule"] = schedule
        }
    }

    /// Move a segment within a day
    func moveSegment(inDayAt dayIndex: Int, from fromIndex: Int, to toIndex: Int) throws {
        guard fromIndex != toIndex else { return }
        try mutatePlanDictionary { root in
            guard var schedule = root["schedule"] as? [[String: Any]], dayIndex < schedule.count else {
                throw PlanMutationError.invalidStructure
            }

            var day = schedule[dayIndex]
            guard var segments = day["segments"] as? [[String: Any]],
                  fromIndex < segments.count,
                  toIndex >= 0, toIndex <= segments.count else {
                throw PlanMutationError.invalidStructure
            }

            let segment = segments.remove(at: fromIndex)
            let destination = min(max(toIndex, 0), segments.count)
            segments.insert(segment, at: destination)

            day["segments"] = segments
            schedule[dayIndex] = day
            root["schedule"] = schedule
        }
    }

    /// Convenience accessor for a display segment
    func segmentDisplay(dayIndex: Int, segmentIndex: Int) -> SegmentDisplay? {
        guard days.indices.contains(dayIndex) else { return nil }
        let segments = days[dayIndex].segments()
        guard segments.indices.contains(segmentIndex) else { return nil }
        return segments[segmentIndex]
    }

    /// Add an exercise entry to the dictionary
    func addExercise(code: String, name: String) throws {
        let updatedJSON = try RustBridge.addExercise(code: code, name: name, to: planJSON)
        updatePlan(updatedJSON)
    }

    func updatePlanMetadata(name: String, author: String?, unit: String, progression: ProgressionSettings?) {
        do {
            try mutatePlanDictionary { root in
                let trimmedName = name.trimmingCharacters(in: .whitespacesAndNewlines)
                if !trimmedName.isEmpty {
                    root["name"] = trimmedName
                }

                let trimmedAuthor = author?.trimmingCharacters(in: .whitespacesAndNewlines) ?? ""
                if trimmedAuthor.isEmpty {
                    root.removeValue(forKey: "author")
                } else {
                    root["author"] = trimmedAuthor
                }

                root["unit"] = unit

                if let progressionDict = progression?.toDictionary() {
                    root["progression"] = progressionDict
                } else {
                    root.removeValue(forKey: "progression")
                }
            }
        } catch {
            print("Failed to update plan metadata: \(error)")
        }
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

    private func mutatePlanDictionary(_ mutate: (inout [String: Any]) throws -> Void) throws {
        guard var root = try JSONSerialization.jsonObject(with: Data(planJSON.utf8)) as? [String: Any] else {
            throw PlanMutationError.serializationFailed
        }

        try mutate(&root)

        let updatedData = try JSONSerialization.data(withJSONObject: root, options: [.sortedKeys])
        guard let updatedJSON = String(data: updatedData, encoding: .utf8) else {
            throw PlanMutationError.serializationFailed
        }

        updatePlan(updatedJSON)
    }

    private static func normalize(json: String) -> String {
        guard var root = try? JSONSerialization.jsonObject(with: Data(json.utf8)) as? [String: Any] else {
            return json
        }

        var changed = false

        if var schedule = root["schedule"] as? [[String: Any]] {
            for dayIndex in schedule.indices {
                if var segments = schedule[dayIndex]["segments"] as? [[String: Any]] {
                    for segIndex in segments.indices {
                        var segment = segments[segIndex]
                        guard let type = segment["type"] as? String else { continue }

                        func ensureRestAndRPE() {
                            if segment["rest_sec"] == nil {
                                segment["rest_sec"] = Defaults.restSeconds
                                changed = true
                            }
                            if segment["rpe"] == nil {
                                segment["rpe"] = Defaults.rpe
                                changed = true
                            }
                        }

                        switch type {
                        case "straight", "rpe", "percentage":
                            ensureRestAndRPE()
                        case "scheme":
                            if var entries = segment["sets"] as? [[String: Any]] {
                                for idx in entries.indices {
                                    var entry = entries[idx]
                                    if entry["rest_sec"] == nil {
                                        entry["rest_sec"] = Defaults.restSeconds
                                        changed = true
                                    }
                                    if entry["rpe"] == nil {
                                        entry["rpe"] = Defaults.rpe
                                        changed = true
                                    }
                                    entries[idx] = entry
                                }
                                segment["sets"] = entries
                            }
                        case "superset", "circuit":
                            if var items = segment["items"] as? [[String: Any]] {
                                for idx in items.indices {
                                    var item = items[idx]
                                    if item["rest_sec"] == nil {
                                        item["rest_sec"] = 0
                                        changed = true
                                    }
                                    if item["rpe"] == nil {
                                        item["rpe"] = Defaults.rpe
                                        changed = true
                                    }
                                    items[idx] = item
                                }
                                segment["items"] = items
                            }
                        default:
                            break
                        }

                        segments[segIndex] = segment
                    }
                    schedule[dayIndex]["segments"] = segments
                }
            }
            root["schedule"] = schedule
        }

        guard changed,
              let data = try? JSONSerialization.data(withJSONObject: root, options: [.sortedKeys]),
              let string = String(data: data, encoding: .utf8) else {
            return json
        }

        return string
    }

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

struct InspectorItem: Identifiable {
    let id = UUID()
    let title: String
    let value: String
}

struct GroupExerciseDetail: Identifiable {
    let id = UUID()
    let name: String
    let code: String?
    let details: String
    let notes: String?
}

struct SchemeSetDetail: Identifiable {
    let id = UUID()
    let title: String
    let summary: String
    let rest: String?
    let notes: String?
}

extension SegmentDisplay {
    enum SegmentKind {
        case straight
        case rpe
        case percentage
        case amrap
        case time
        case superset
        case circuit
        case scheme
        case comment
        case choose
        case generic
    }

    var kind: SegmentKind {
        switch type {
        case "straight":
            return .straight
        case "rpe":
            return .rpe
        case "percentage":
            return .percentage
        case "amrap":
            return .amrap
        case "time":
            return .time
        case "superset", "group.superset":
            return .superset
        case "circuit", "group.circuit":
            return .circuit
        case "scheme":
            return .scheme
        case "comment":
            return .comment
        case "choose":
            return .choose
        default:
            return .generic
        }
    }

    var humanReadableType: String {
        type.replacingOccurrences(of: ".", with: " ").capitalized
    }

    func primaryTitle(with plan: PlanDocument) -> String {
        if let base = segmentDict["base"] as? [String: Any] {
            if let label = base["label"] as? String, !label.isEmpty {
                return label
            }
            if let ex = base["ex"] as? String {
                if let friendly = plan.dictionary[ex] {
                    return friendly
                }
                return ex
            }
        }
        if let label = segmentDict["label"] as? String, !label.isEmpty {
            return label
        }
        if let text = segmentDict["text"] as? String, !text.isEmpty {
            return text
        }
        return displayText
    }

    var setsDescription: String {
        if let sets = segmentDict["sets"] as? Int {
            if let repsRange = segmentDict["reps"] as? [String: Any] {
                if let min = repsRange["min"] as? Int, let max = repsRange["max"] as? Int {
                    if min == max {
                        return "\(sets) × \(min)"
                    }
                    return "\(sets) × \(min)–\(max)"
                }
            } else if let reps = segmentDict["reps"] as? Int {
                return "\(sets) × \(reps)"
            }
            return "\(sets) sets"
        }
        if let rounds = segmentDict["rounds"] as? Int, rounds > 0 {
            return "\(rounds) rounds"
        }
        return "—"
    }

    var restDescription: String {
        if let rest = segmentDict["rest_sec"] as? Int {
            return "\(rest) s"
        }
        if let rest = segmentDict["rest_between_rounds_sec"] as? Int {
            return "\(rest) s between rounds"
        }
        return "—"
    }

    var notesDescription: String {
        if let note = segmentDict["note"] as? String, !note.isEmpty {
            return note
        }
        if type == "comment", let text = segmentDict["text"] as? String, !text.isEmpty {
            return text
        }
        if let tempo = segmentDict["tempo"] as? String, !tempo.isEmpty {
            return "Tempo \(tempo)"
        }
        return "—"
    }

    var exerciseCode: String? {
        baseValue("ex") ?? segmentDict["ex"] as? String
    }

    var altGroupCode: String? {
        baseValue("alt_group") ?? segmentDict["alt_group"] as? String
    }

    var commentText: String? {
        segmentDict["text"] as? String
    }

    func basicInspectorItems(plan: PlanDocument) -> [InspectorItem] {
        var items: [InspectorItem] = []
        if let code = exerciseCode {
            let name = plan.dictionary[code] ?? code
            items.append(.init(title: "Exercise", value: name))
            items.append(.init(title: "Code", value: code))
        } else {
            items.append(.init(title: "Exercise", value: primaryTitle(with: plan)))
        }

        if let alt = altGroupCode {
            items.append(.init(title: "Alt Group", value: alt))
        }

        items.append(.init(title: "Sets × Reps", value: setsDescription))

        if let rest = restSeconds {
            items.append(.init(title: "Rest", value: SegmentDisplay.formatSeconds(rest)))
        }

        if let between = intValue("rest_between_rounds_sec") {
            items.append(.init(title: "Between Rounds", value: SegmentDisplay.formatSeconds(between)))
        }

        if let rpe = rpeValue {
            items.append(.init(title: "RPE", value: SegmentDisplay.formatDecimal(rpe)))
        }

        if let rir = doubleValue("rir") {
            items.append(.init(title: "RIR", value: SegmentDisplay.formatDecimal(rir)))
        }

        if let tempo = tempoValue {
            items.append(.init(title: "Tempo", value: tempo))
        }

        if let notes = inspectorNote {
            items.append(.init(title: "Notes", value: notes))
        }

        return items
    }

    func groupMetadataItems() -> [InspectorItem] {
        var items: [InspectorItem] = []
        if let rounds = intValue("rounds") {
            items.append(.init(title: "Rounds", value: "\(rounds)"))
        }
        if let rest = restSeconds {
            items.append(.init(title: "Rest", value: SegmentDisplay.formatSeconds(rest)))
        }
        if let between = intValue("rest_between_rounds_sec") {
            items.append(.init(title: "Between Rounds", value: SegmentDisplay.formatSeconds(between)))
        }
        if let pairing = stringValue("pairing") {
            items.append(.init(title: "Pairing", value: pairing))
        }
        return items
    }

    func groupExercises(plan: PlanDocument) -> [GroupExerciseDetail] {
        let possibleKeys = ["exercises", "items", "children", "segments"]
        var stack: [[String: Any]] = []
        for key in possibleKeys {
            if let entries = segmentDict[key] as? [[String: Any]] {
                stack.append(contentsOf: entries)
            }
        }

        guard !stack.isEmpty else {
            return []
        }

        return stack.enumerated().map { index, entry in
            let code = entry["ex"] as? String
            let name = code.flatMap { plan.dictionary[$0] } ??
                (entry["label"] as? String) ??
                "Item \(index + 1)"

            var details: [String] = []
            if let sets = entry["sets"] as? Int {
                details.append("\(sets) sets")
            }

            if let repsDetail = SegmentDisplay.describeReps(from: entry["reps"]) {
                details.append(repsDetail)
            } else if let reps = entry["reps"] as? Int {
                details.append("\(reps) reps")
            } else if let range = entry["reps_min"] as? Int, let max = entry["reps_max"] as? Int {
                details.append("\(range)-\(max) reps")
            }

            if let rest = entry["rest_sec"] as? Int {
                details.append("Rest \(SegmentDisplay.formatSeconds(rest))")
            }

            if let time = entry["time_sec"] as? Int {
                details.append("\(SegmentDisplay.formatSeconds(time)) duration")
            }

            if let rpe = entry["rpe"] as? Double {
                details.append("RPE \(SegmentDisplay.formatDecimal(rpe))")
            }

            let notes = entry["note"] as? String
            return GroupExerciseDetail(name: name,
                                       code: code,
                                       details: details.joined(separator: " • "),
                                       notes: notes)
        }
    }

    func schemeSetDetails() -> [SchemeSetDetail] {
        guard let sets = segmentDict["sets"] as? [[String: Any]] else {
            return []
        }

        return sets.enumerated().map { index, entry in
            let title = entry["label"] as? String ?? "Set \(index + 1)"
            var parts: [String] = []
            if let count = entry["sets"] as? Int {
                parts.append("\(count) sets")
            }
            if let repsDetail = SegmentDisplay.describeReps(from: entry["reps"]) {
                parts.append(repsDetail)
            } else if let reps = entry["reps"] as? Int {
                parts.append("\(reps) reps")
            }
            if let rpe = entry["rpe"] as? Double {
                parts.append("RPE \(SegmentDisplay.formatDecimal(rpe))")
            }
            if let tempo = entry["tempo"] as? String {
                parts.append("Tempo \(tempo)")
            }
            let rest = (entry["rest_sec"] as? Int).map { SegmentDisplay.formatSeconds($0) }
            let notes = entry["note"] as? String
            return SchemeSetDetail(title: title, summary: parts.joined(separator: " • "), rest: rest, notes: notes)
        }
    }

    func choiceOptions(plan: PlanDocument) -> [String] {
        if let options = segmentDict["options"] as? [String] {
            return options
        }
        if let from = segmentDict["from"] as? [[String: Any]] {
            return from.enumerated().map { index, entry in
                if let code = entry["ex"] as? String {
                    return plan.dictionary[code] ?? code
                }
                if let label = entry["label"] as? String {
                    return label
                }
                return "Option \(index + 1)"
            }
        }
        if let array = segmentDict["from"] as? [String] {
            return array
        }
        return []
    }

    var restSeconds: Int? {
        intValue("rest_sec")
    }

    var tempoValue: String? {
        stringValue("tempo")
    }

    var rpeValue: Double? {
        doubleValue("rpe")
    }

    var inspectorNote: String? {
        if let note = segmentDict["note"] as? String, !note.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty {
            return note
        }
        if kind == .comment, let text = segmentDict["text"] as? String, !text.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty {
            return text
        }
        return nil
    }

    func stringValue(_ key: String) -> String? {
        baseValue(key) ?? segmentDict[key] as? String
    }

    func intValue(_ key: String) -> Int? {
        if let value: Int = baseValue(key) {
            return value
        }
        if let number = segmentDict[key] as? NSNumber {
            return number.intValue
        }
        return segmentDict[key] as? Int
    }

    func doubleValue(_ key: String) -> Double? {
        if let value: Double = baseValue(key) {
            return value
        }
        if let number = segmentDict[key] as? NSNumber {
            return number.doubleValue
        }
        return segmentDict[key] as? Double
    }

    private func baseDictionary() -> [String: Any]? {
        segmentDict["base"] as? [String: Any]
    }

    private func baseValue<T>(_ key: String) -> T? {
        baseDictionary()?[key] as? T
    }

    private static func formatSeconds(_ seconds: Int) -> String {
        if seconds >= 60 {
            let minutes = seconds / 60
            let remainder = seconds % 60
            if remainder == 0 {
                return "\(minutes)m"
            } else {
                return "\(minutes)m \(remainder)s"
            }
        }
        return "\(seconds)s"
    }

    static func formatDecimal(_ value: Double) -> String {
        if value == floor(value) {
            return String(format: "%.0f", value)
        }
        return String(format: "%.1f", value)
    }

    static func describeReps(from value: Any?) -> String? {
        guard let dict = value as? [String: Any] else {
            return nil
        }
        if let min = dict["min"] as? Int, let max = dict["max"] as? Int {
            if min == max {
                return "\(min) reps"
            }
            return "\(min)–\(max) reps"
        }
        if let reps = dict["value"] as? Int {
            return "\(reps) reps"
        }
        return nil
    }

    static func prettyValue(_ value: Any) -> String {
        if let string = value as? String {
            return string
        }
        if let number = value as? NSNumber {
            return number.stringValue
        }
        if let bool = value as? Bool {
            return bool ? "True" : "False"
        }
        if let array = value as? [Any] {
            return array.map { prettyValue($0) }.joined(separator: ", ")
        }
        if let dict = value as? [String: Any] {
            let pairs = dict.map { "\($0.key): \(prettyValue($0.value))" }
            return "{ \(pairs.joined(separator: ", ")) }"
        }
        return "\(value)"
    }
}

private extension String {
    var nonEmptyValue: String? {
        isEmpty ? nil : self
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
