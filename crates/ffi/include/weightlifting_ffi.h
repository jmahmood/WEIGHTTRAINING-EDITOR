#ifndef WEIGHTLIFTING_FFI_H
#define WEIGHTLIFTING_FFI_H

#pragma once

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

/**
 * Opaque handle to a Plan object
 */
typedef struct PlanHandle PlanHandle;

/**
 * Represents a result returned from FFI calls
 */
typedef struct FFIResult {
  bool success;
  char *data;
  char *error;
} FFIResult;

/**
 * Frees a C string allocated by Rust
 */
void ffi_free_string(char *ptr);

/**
 * Frees an FFIResult
 */
void ffi_free_result(struct FFIResult result);

/**
 * Frees a PlanHandle
 */
void ffi_plan_free(struct PlanHandle *handle);

/**
 * Creates a new empty plan
 * Returns a JSON string with the plan data
 */
struct FFIResult ffi_plan_new(void);

/**
 * Opens a plan from a file path
 * Returns a JSON string with the plan data
 */
struct FFIResult ffi_plan_open(const char *path);

/**
 * Saves a plan to a file path
 * plan_json: JSON string representation of the plan
 * path: File path to save to
 */
struct FFIResult ffi_plan_save(const char *plan_json, const char *path);

/**
 * Validates a plan
 * Returns a JSON array of validation errors (empty if valid)
 */
struct FFIResult ffi_plan_validate(const char *plan_json);

/**
 * Adds a segment to a plan
 * plan_json: JSON string representation of the plan
 * day_index: Index of the day to add the segment to
 * segment_json: JSON string representation of the segment
 * Returns updated plan as JSON
 */
struct FFIResult ffi_segment_add(const char *plan_json,
                                 uintptr_t day_index,
                                 const char *segment_json);

/**
 * Removes a segment from a plan
 */
struct FFIResult ffi_segment_remove(const char *plan_json,
                                    uintptr_t day_index,
                                    uintptr_t segment_index);

/**
 * Updates a segment in a plan
 */
struct FFIResult ffi_segment_update(const char *plan_json,
                                    uintptr_t day_index,
                                    uintptr_t segment_index,
                                    const char *segment_json);

/**
 * Adds a day to a plan
 */
struct FFIResult ffi_day_add(const char *plan_json, const char *day_json);

/**
 * Removes a day from a plan
 */
struct FFIResult ffi_day_remove(const char *plan_json, uintptr_t day_index);

/**
 * Gets all exercise groups from a plan (returns HashMap<String, Vec<String>>)
 */
struct FFIResult ffi_groups_get(const char *plan_json);

/**
 * Adds or updates an exercise group in a plan
 * group_name: Name of the group
 * exercises_json: JSON array of exercise codes ["ex1", "ex2"]
 */
struct FFIResult ffi_group_add(const char *plan_json,
                               const char *group_name,
                               const char *exercises_json);

/**
 * Removes an exercise group from a plan
 */
struct FFIResult ffi_group_remove(const char *plan_json, const char *group_name);

/**
 * Adds or updates an exercise dictionary entry in the plan
 */
struct FFIResult ffi_dictionary_add_entry(const char *plan_json,
                                          const char *exercise_code,
                                          const char *exercise_name);

/**
 * Gets the application support directory path for the platform
 */
struct FFIResult ffi_get_app_support_dir(void);

/**
 * Gets the cache directory path for the platform
 */
struct FFIResult ffi_get_cache_dir(void);

/**
 * Gets the drafts directory path for the platform
 */
struct FFIResult ffi_get_drafts_dir(void);

#endif /* WEIGHTLIFTING_FFI_H */
