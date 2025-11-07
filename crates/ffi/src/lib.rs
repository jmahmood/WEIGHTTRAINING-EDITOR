use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::path::PathBuf;
use weightlifting_core::models::{Plan, Segment, Day};
use weightlifting_validate::PlanValidator;

/// Represents a result returned from FFI calls
#[repr(C)]
pub struct FFIResult {
    pub success: bool,
    pub data: *mut c_char,
    pub error: *mut c_char,
}

/// Opaque handle to a Plan object
pub struct PlanHandle {
    plan: Plan,
}

// ============================================================================
// Memory Management
// ============================================================================

/// Frees a C string allocated by Rust
#[no_mangle]
pub extern "C" fn ffi_free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        unsafe {
            let _ = CString::from_raw(ptr);
        }
    }
}

/// Frees an FFIResult
#[no_mangle]
pub extern "C" fn ffi_free_result(result: FFIResult) {
    ffi_free_string(result.data);
    ffi_free_string(result.error);
}

/// Frees a PlanHandle
#[no_mangle]
pub extern "C" fn ffi_plan_free(handle: *mut PlanHandle) {
    if !handle.is_null() {
        unsafe {
            let _ = Box::from_raw(handle);
        }
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

fn create_success_result(data: String) -> FFIResult {
    FFIResult {
        success: true,
        data: CString::new(data).unwrap().into_raw(),
        error: std::ptr::null_mut(),
    }
}

fn create_error_result(error: String) -> FFIResult {
    FFIResult {
        success: false,
        data: std::ptr::null_mut(),
        error: CString::new(error).unwrap().into_raw(),
    }
}

fn c_str_to_string(ptr: *const c_char) -> Result<String, String> {
    if ptr.is_null() {
        return Err("Null pointer passed".to_string());
    }
    unsafe {
        CStr::from_ptr(ptr)
            .to_str()
            .map(|s| s.to_string())
            .map_err(|e| format!("Invalid UTF-8: {}", e))
    }
}

// ============================================================================
// Plan Operations
// ============================================================================

/// Creates a new empty plan
/// Returns a JSON string with the plan data
#[no_mangle]
pub extern "C" fn ffi_plan_new() -> FFIResult {
    let plan = Plan::new("New Plan".to_string());
    match serde_json::to_string(&plan) {
        Ok(json) => create_success_result(json),
        Err(e) => create_error_result(format!("Failed to serialize plan: {}", e)),
    }
}

/// Opens a plan from a file path
/// Returns a JSON string with the plan data
#[no_mangle]
pub extern "C" fn ffi_plan_open(path: *const c_char) -> FFIResult {
    let path_str = match c_str_to_string(path) {
        Ok(s) => s,
        Err(e) => return create_error_result(e),
    };

    let path = PathBuf::from(path_str);
    match std::fs::read_to_string(&path) {
        Ok(content) => match serde_json::from_str::<Plan>(&content) {
            Ok(plan) => match serde_json::to_string(&plan) {
                Ok(json) => create_success_result(json),
                Err(e) => create_error_result(format!("Failed to serialize plan: {}", e)),
            },
            Err(e) => create_error_result(format!("Failed to parse plan: {}", e)),
        },
        Err(e) => create_error_result(format!("Failed to read file: {}", e)),
    }
}

/// Saves a plan to a file path
/// plan_json: JSON string representation of the plan
/// path: File path to save to
#[no_mangle]
pub extern "C" fn ffi_plan_save(plan_json: *const c_char, path: *const c_char) -> FFIResult {
    let plan_str = match c_str_to_string(plan_json) {
        Ok(s) => s,
        Err(e) => return create_error_result(e),
    };

    let path_str = match c_str_to_string(path) {
        Ok(s) => s,
        Err(e) => return create_error_result(e),
    };

    let plan: Plan = match serde_json::from_str(&plan_str) {
        Ok(p) => p,
        Err(e) => return create_error_result(format!("Failed to parse plan JSON: {}", e)),
    };

    let path = PathBuf::from(path_str);
    match std::fs::write(&path, serde_json::to_string_pretty(&plan).unwrap()) {
        Ok(_) => create_success_result("Plan saved successfully".to_string()),
        Err(e) => create_error_result(format!("Failed to write file: {}", e)),
    }
}

/// Validates a plan
/// Returns a JSON array of validation errors (empty if valid)
#[no_mangle]
pub extern "C" fn ffi_plan_validate(plan_json: *const c_char) -> FFIResult {
    let plan_str = match c_str_to_string(plan_json) {
        Ok(s) => s,
        Err(e) => return create_error_result(e),
    };

    let plan: Plan = match serde_json::from_str(&plan_str) {
        Ok(p) => p,
        Err(e) => return create_error_result(format!("Failed to parse plan JSON: {}", e)),
    };

    let validator = match PlanValidator::new() {
        Ok(v) => v,
        Err(e) => return create_error_result(format!("Failed to create validator: {}", e)),
    };

    let result = validator.validate(&plan);
    match serde_json::to_string(&result) {
        Ok(json) => create_success_result(json),
        Err(e) => create_error_result(format!("Failed to serialize errors: {}", e)),
    }
}

// ============================================================================
// Segment Operations
// ============================================================================

/// Adds a segment to a plan
/// plan_json: JSON string representation of the plan
/// day_index: Index of the day to add the segment to
/// segment_json: JSON string representation of the segment
/// Returns updated plan as JSON
#[no_mangle]
pub extern "C" fn ffi_segment_add(
    plan_json: *const c_char,
    day_index: usize,
    segment_json: *const c_char,
) -> FFIResult {
    let plan_str = match c_str_to_string(plan_json) {
        Ok(s) => s,
        Err(e) => return create_error_result(e),
    };

    let segment_str = match c_str_to_string(segment_json) {
        Ok(s) => s,
        Err(e) => return create_error_result(e),
    };

    let mut plan: Plan = match serde_json::from_str(&plan_str) {
        Ok(p) => p,
        Err(e) => return create_error_result(format!("Failed to parse plan JSON: {}", e)),
    };

    let segment: Segment = match serde_json::from_str(&segment_str) {
        Ok(s) => s,
        Err(e) => return create_error_result(format!("Failed to parse segment JSON: {}", e)),
    };

    if day_index >= plan.schedule.len() {
        return create_error_result(format!("Day index {} out of bounds", day_index));
    }

    plan.schedule[day_index].segments.push(segment);

    match serde_json::to_string(&plan) {
        Ok(json) => create_success_result(json),
        Err(e) => create_error_result(format!("Failed to serialize plan: {}", e)),
    }
}

/// Removes a segment from a plan
#[no_mangle]
pub extern "C" fn ffi_segment_remove(
    plan_json: *const c_char,
    day_index: usize,
    segment_index: usize,
) -> FFIResult {
    let plan_str = match c_str_to_string(plan_json) {
        Ok(s) => s,
        Err(e) => return create_error_result(e),
    };

    let mut plan: Plan = match serde_json::from_str(&plan_str) {
        Ok(p) => p,
        Err(e) => return create_error_result(format!("Failed to parse plan JSON: {}", e)),
    };

    if day_index >= plan.schedule.len() {
        return create_error_result(format!("Day index {} out of bounds", day_index));
    }

    if segment_index >= plan.schedule[day_index].segments.len() {
        return create_error_result(format!("Segment index {} out of bounds", segment_index));
    }

    plan.schedule[day_index].segments.remove(segment_index);

    match serde_json::to_string(&plan) {
        Ok(json) => create_success_result(json),
        Err(e) => create_error_result(format!("Failed to serialize plan: {}", e)),
    }
}

/// Updates a segment in a plan
#[no_mangle]
pub extern "C" fn ffi_segment_update(
    plan_json: *const c_char,
    day_index: usize,
    segment_index: usize,
    segment_json: *const c_char,
) -> FFIResult {
    let plan_str = match c_str_to_string(plan_json) {
        Ok(s) => s,
        Err(e) => return create_error_result(e),
    };

    let segment_str = match c_str_to_string(segment_json) {
        Ok(s) => s,
        Err(e) => return create_error_result(e),
    };

    let mut plan: Plan = match serde_json::from_str(&plan_str) {
        Ok(p) => p,
        Err(e) => return create_error_result(format!("Failed to parse plan JSON: {}", e)),
    };

    let segment: Segment = match serde_json::from_str(&segment_str) {
        Ok(s) => s,
        Err(e) => return create_error_result(format!("Failed to parse segment JSON: {}", e)),
    };

    if day_index >= plan.schedule.len() {
        return create_error_result(format!("Day index {} out of bounds", day_index));
    }

    if segment_index >= plan.schedule[day_index].segments.len() {
        return create_error_result(format!("Segment index {} out of bounds", segment_index));
    }

    plan.schedule[day_index].segments[segment_index] = segment;

    match serde_json::to_string(&plan) {
        Ok(json) => create_success_result(json),
        Err(e) => create_error_result(format!("Failed to serialize plan: {}", e)),
    }
}

// ============================================================================
// Day Operations
// ============================================================================

/// Adds a day to a plan
#[no_mangle]
pub extern "C" fn ffi_day_add(plan_json: *const c_char, day_json: *const c_char) -> FFIResult {
    let plan_str = match c_str_to_string(plan_json) {
        Ok(s) => s,
        Err(e) => return create_error_result(e),
    };

    let day_str = match c_str_to_string(day_json) {
        Ok(s) => s,
        Err(e) => return create_error_result(e),
    };

    let mut plan: Plan = match serde_json::from_str(&plan_str) {
        Ok(p) => p,
        Err(e) => return create_error_result(format!("Failed to parse plan JSON: {}", e)),
    };

    let day: Day = match serde_json::from_str(&day_str) {
        Ok(d) => d,
        Err(e) => return create_error_result(format!("Failed to parse day JSON: {}", e)),
    };

    plan.schedule.push(day);

    match serde_json::to_string(&plan) {
        Ok(json) => create_success_result(json),
        Err(e) => create_error_result(format!("Failed to serialize plan: {}", e)),
    }
}

/// Removes a day from a plan
#[no_mangle]
pub extern "C" fn ffi_day_remove(plan_json: *const c_char, day_index: usize) -> FFIResult {
    let plan_str = match c_str_to_string(plan_json) {
        Ok(s) => s,
        Err(e) => return create_error_result(e),
    };

    let mut plan: Plan = match serde_json::from_str(&plan_str) {
        Ok(p) => p,
        Err(e) => return create_error_result(format!("Failed to parse plan JSON: {}", e)),
    };

    if day_index >= plan.schedule.len() {
        return create_error_result(format!("Day index {} out of bounds", day_index));
    }

    plan.schedule.remove(day_index);

    match serde_json::to_string(&plan) {
        Ok(json) => create_success_result(json),
        Err(e) => create_error_result(format!("Failed to serialize plan: {}", e)),
    }
}

// ============================================================================
// Exercise Group Operations
// ============================================================================

/// Gets all exercise groups from a plan (returns HashMap<String, Vec<String>>)
#[no_mangle]
pub extern "C" fn ffi_groups_get(plan_json: *const c_char) -> FFIResult {
    let plan_str = match c_str_to_string(plan_json) {
        Ok(s) => s,
        Err(e) => return create_error_result(e),
    };

    let plan: Plan = match serde_json::from_str(&plan_str) {
        Ok(p) => p,
        Err(e) => return create_error_result(format!("Failed to parse plan JSON: {}", e)),
    };

    match serde_json::to_string(&plan.groups) {
        Ok(json) => create_success_result(json),
        Err(e) => create_error_result(format!("Failed to serialize groups: {}", e)),
    }
}

/// Adds or updates an exercise group in a plan
/// group_name: Name of the group
/// exercises_json: JSON array of exercise codes ["ex1", "ex2"]
#[no_mangle]
pub extern "C" fn ffi_group_add(
    plan_json: *const c_char,
    group_name: *const c_char,
    exercises_json: *const c_char,
) -> FFIResult {
    let plan_str = match c_str_to_string(plan_json) {
        Ok(s) => s,
        Err(e) => return create_error_result(e),
    };

    let name = match c_str_to_string(group_name) {
        Ok(s) => s,
        Err(e) => return create_error_result(e),
    };

    let exercises_str = match c_str_to_string(exercises_json) {
        Ok(s) => s,
        Err(e) => return create_error_result(e),
    };

    let mut plan: Plan = match serde_json::from_str(&plan_str) {
        Ok(p) => p,
        Err(e) => return create_error_result(format!("Failed to parse plan JSON: {}", e)),
    };

    let exercises: Vec<String> = match serde_json::from_str(&exercises_str) {
        Ok(e) => e,
        Err(e) => return create_error_result(format!("Failed to parse exercises JSON: {}", e)),
    };

    plan.groups.insert(name, exercises);

    match serde_json::to_string(&plan) {
        Ok(json) => create_success_result(json),
        Err(e) => create_error_result(format!("Failed to serialize plan: {}", e)),
    }
}

/// Removes an exercise group from a plan
#[no_mangle]
pub extern "C" fn ffi_group_remove(plan_json: *const c_char, group_name: *const c_char) -> FFIResult {
    let plan_str = match c_str_to_string(plan_json) {
        Ok(s) => s,
        Err(e) => return create_error_result(e),
    };

    let name = match c_str_to_string(group_name) {
        Ok(s) => s,
        Err(e) => return create_error_result(e),
    };

    let mut plan: Plan = match serde_json::from_str(&plan_str) {
        Ok(p) => p,
        Err(e) => return create_error_result(format!("Failed to parse plan JSON: {}", e)),
    };

    plan.groups.remove(&name);

    match serde_json::to_string(&plan) {
        Ok(json) => create_success_result(json),
        Err(e) => create_error_result(format!("Failed to serialize plan: {}", e)),
    }
}

// ============================================================================
// Platform Paths
// ============================================================================

/// Gets the application support directory path for the platform
#[no_mangle]
pub extern "C" fn ffi_get_app_support_dir() -> FFIResult {
    match weightlifting_core::paths::get_app_support_dir() {
        Ok(path) => create_success_result(path.to_string_lossy().to_string()),
        Err(e) => create_error_result(format!("Failed to get app support dir: {}", e)),
    }
}

/// Gets the cache directory path for the platform
#[no_mangle]
pub extern "C" fn ffi_get_cache_dir() -> FFIResult {
    match weightlifting_core::paths::get_cache_dir() {
        Ok(path) => create_success_result(path.to_string_lossy().to_string()),
        Err(e) => create_error_result(format!("Failed to get cache dir: {}", e)),
    }
}

/// Gets the drafts directory path for the platform
#[no_mangle]
pub extern "C" fn ffi_get_drafts_dir() -> FFIResult {
    match weightlifting_core::paths::get_drafts_dir() {
        Ok(path) => create_success_result(path.to_string_lossy().to_string()),
        Err(e) => create_error_result(format!("Failed to get drafts dir: {}", e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plan_new() {
        let result = ffi_plan_new();
        assert!(result.success);
        assert!(!result.data.is_null());
        ffi_free_result(result);
    }

    #[test]
    fn test_plan_validate() {
        let result = ffi_plan_new();
        let plan_json = result.data;
        let validate_result = ffi_plan_validate(plan_json);
        assert!(validate_result.success);
        ffi_free_result(result);
        ffi_free_result(validate_result);
    }
}
