import Foundation
import CFFIBridge

/// Swift wrapper for Rust FFI calls
/// Works with JSON strings - no complex Swift models needed
class RustBridge {
    /// Wrapper for FFI result
    private struct FFIResultWrapper {
        let success: Bool
        let data: String?
        let error: String?

        init(from ffiResult: FFIResult) {
            self.success = ffiResult.success
            self.data = ffiResult.data != nil ? String(cString: ffiResult.data!) : nil
            self.error = ffiResult.error != nil ? String(cString: ffiResult.error!) : nil

            // Clean up FFI memory
            ffi_free_result(ffiResult)
        }
    }

    // MARK: - Plan Operations

    /// Create a new plan and return as JSON string
    static func createNewPlanJSON() throws -> String {
        let result = FFIResultWrapper(from: ffi_plan_new())

        guard result.success, let data = result.data else {
            throw RustBridgeError.ffiError(result.error ?? "Unknown error creating plan")
        }

        return data
    }

    /// Open a plan from file and return as JSON string
    static func openPlanJSON(at path: String) throws -> String {
        let cPath = path.cString(using: .utf8)!
        let result = FFIResultWrapper(from: ffi_plan_open(cPath))

        guard result.success, let data = result.data else {
            throw RustBridgeError.ffiError(result.error ?? "Unknown error opening plan")
        }

        return data
    }

    /// Save a plan JSON to file
    static func savePlan(_ planJSON: String, to path: String) throws {
        let cJson = planJSON.cString(using: .utf8)!
        let cPath = path.cString(using: .utf8)!

        let result = FFIResultWrapper(from: ffi_plan_save(cJson, cPath))

        guard result.success else {
            throw RustBridgeError.ffiError(result.error ?? "Unknown error saving plan")
        }
    }

    /// Validate a plan and return validation result
    static func validatePlan(_ planJSON: String) throws -> ValidationResult {
        let cJson = planJSON.cString(using: .utf8)!

        let result = FFIResultWrapper(from: ffi_plan_validate(cJson))

        guard result.success, let data = result.data else {
            throw RustBridgeError.ffiError(result.error ?? "Unknown error validating plan")
        }

        let decoder = JSONDecoder()
        decoder.keyDecodingStrategy = .convertFromSnakeCase
        return try decoder.decode(ValidationResult.self, from: Data(data.utf8))
    }

    // MARK: - Segment Operations

    /// Add a segment to a day, returns updated plan JSON
    static func addSegment(_ segmentJSON: String, toDayAt dayIndex: Int, in planJSON: String) throws -> String {
        let cPlan = planJSON.cString(using: .utf8)!
        let cSegment = segmentJSON.cString(using: .utf8)!

        let result = FFIResultWrapper(from: ffi_segment_add(cPlan, UInt(dayIndex), cSegment))

        guard result.success, let data = result.data else {
            throw RustBridgeError.ffiError(result.error ?? "Unknown error adding segment")
        }

        return data
    }

    /// Remove a segment from a day, returns updated plan JSON
    static func removeSegment(at segmentIndex: Int, fromDayAt dayIndex: Int, in planJSON: String) throws -> String {
        let cPlan = planJSON.cString(using: .utf8)!

        let result = FFIResultWrapper(from: ffi_segment_remove(cPlan, UInt(dayIndex), UInt(segmentIndex)))

        guard result.success, let data = result.data else {
            throw RustBridgeError.ffiError(result.error ?? "Unknown error removing segment")
        }

        return data
    }

    /// Update a segment, returns updated plan JSON
    static func updateSegment(_ segmentJSON: String, at segmentIndex: Int, inDayAt dayIndex: Int, in planJSON: String) throws -> String {
        let cPlan = planJSON.cString(using: .utf8)!
        let cSegment = segmentJSON.cString(using: .utf8)!

        let result = FFIResultWrapper(from: ffi_segment_update(cPlan, UInt(dayIndex), UInt(segmentIndex), cSegment))

        guard result.success, let data = result.data else {
            throw RustBridgeError.ffiError(result.error ?? "Unknown error updating segment")
        }

        return data
    }

    // MARK: - Day Operations

    /// Add a day to the plan, returns updated plan JSON
    static func addDay(_ dayJSON: String, to planJSON: String) throws -> String {
        let cPlan = planJSON.cString(using: .utf8)!
        let cDay = dayJSON.cString(using: .utf8)!

        let result = FFIResultWrapper(from: ffi_day_add(cPlan, cDay))

        guard result.success, let data = result.data else {
            throw RustBridgeError.ffiError(result.error ?? "Unknown error adding day")
        }

        return data
    }

    /// Remove a day from the plan, returns updated plan JSON
    static func removeDay(at index: Int, from planJSON: String) throws -> String {
        let cPlan = planJSON.cString(using: .utf8)!

        let result = FFIResultWrapper(from: ffi_day_remove(cPlan, UInt(index)))

        guard result.success, let data = result.data else {
            throw RustBridgeError.ffiError(result.error ?? "Unknown error removing day")
        }

        return data
    }

    // MARK: - Exercise Group Operations

    /// Get all groups as JSON (HashMap<String, Vec<String>>)
    static func getGroups(from planJSON: String) throws -> [String: [String]] {
        let cPlan = planJSON.cString(using: .utf8)!

        let result = FFIResultWrapper(from: ffi_groups_get(cPlan))

        guard result.success, let data = result.data else {
            throw RustBridgeError.ffiError(result.error ?? "Unknown error getting groups")
        }

        let decoder = JSONDecoder()
        return try decoder.decode([String: [String]].self, from: Data(data.utf8))
    }

    /// Add or update a group, returns updated plan JSON
    static func addGroup(name: String, exercises: [String], to planJSON: String) throws -> String {
        let cPlan = planJSON.cString(using: .utf8)!
        let cName = name.cString(using: .utf8)!

        let encoder = JSONEncoder()
        let exercisesData = try encoder.encode(exercises)
        let exercisesJSON = String(data: exercisesData, encoding: .utf8)!
        let cExercises = exercisesJSON.cString(using: .utf8)!

        let result = FFIResultWrapper(from: ffi_group_add(cPlan, cName, cExercises))

        guard result.success, let data = result.data else {
            throw RustBridgeError.ffiError(result.error ?? "Unknown error adding group")
        }

        return data
    }

    /// Remove a group, returns updated plan JSON
    static func removeGroup(name: String, from planJSON: String) throws -> String {
        let cPlan = planJSON.cString(using: .utf8)!
        let cName = name.cString(using: .utf8)!

        let result = FFIResultWrapper(from: ffi_group_remove(cPlan, cName))

        guard result.success, let data = result.data else {
            throw RustBridgeError.ffiError(result.error ?? "Unknown error removing group")
        }

        return data
    }

    /// Add or update a dictionary entry, returns updated plan JSON
    static func addExercise(code: String, name: String, to planJSON: String) throws -> String {
        let cPlan = planJSON.cString(using: .utf8)!
        let cCode = code.cString(using: .utf8)!
        let cName = name.cString(using: .utf8)!

        let result = FFIResultWrapper(from: ffi_dictionary_add_entry(cPlan, cCode, cName))

        guard result.success, let data = result.data else {
            throw RustBridgeError.ffiError(result.error ?? "Unknown error adding exercise")
        }

        return data
    }

    // MARK: - Platform Paths

    static func getAppSupportDir() throws -> String {
        let result = FFIResultWrapper(from: ffi_get_app_support_dir())

        guard result.success, let path = result.data else {
            throw RustBridgeError.ffiError(result.error ?? "Unknown error getting app support dir")
        }

        return path
    }

    static func getCacheDir() throws -> String {
        let result = FFIResultWrapper(from: ffi_get_cache_dir())

        guard result.success, let path = result.data else {
            throw RustBridgeError.ffiError(result.error ?? "Unknown error getting cache dir")
        }

        return path
    }

    static func getDraftsDir() throws -> String {
        let result = FFIResultWrapper(from: ffi_get_drafts_dir())

        guard result.success, let path = result.data else {
            throw RustBridgeError.ffiError(result.error ?? "Unknown error getting drafts dir")
        }

        return path
    }
}

// MARK: - Error Type

enum RustBridgeError: Error, LocalizedError {
    case ffiError(String)
    case encodingError
    case decodingError

    var errorDescription: String? {
        switch self {
        case .ffiError(let message):
            return message
        case .encodingError:
            return "Failed to encode data"
        case .decodingError:
            return "Failed to decode data"
        }
    }
}
