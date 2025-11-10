use crate::state::AppState;
use std::error::Error;
use std::fmt;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
#[allow(dead_code)]
pub enum GroupError {
    NoCurrentPlan,
    GroupAlreadyExists(String),
    GroupNotFound(String),
    ExerciseNotInDictionary(String),
    EmptyGroupName,
    EmptyExercisesList,
}

impl fmt::Display for GroupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GroupError::NoCurrentPlan => write!(f, "No plan is currently loaded"),
            GroupError::GroupAlreadyExists(name) => write!(f, "Group '{}' already exists", name),
            GroupError::GroupNotFound(name) => write!(f, "Group '{}' not found", name),
            GroupError::ExerciseNotInDictionary(code) => {
                write!(f, "Exercise '{}' not found in dictionary", code)
            }
            GroupError::EmptyGroupName => write!(f, "Group name cannot be empty"),
            GroupError::EmptyExercisesList => write!(f, "Group must contain at least one exercise"),
        }
    }
}

impl Error for GroupError {}

/// Creates a new exercise group
pub fn create_group(
    state: Arc<Mutex<AppState>>,
    name: String,
    exercises: Vec<String>,
) -> Result<(), GroupError> {
    let mut state_lock = state.lock().unwrap();

    // Validate input
    if name.trim().is_empty() {
        return Err(GroupError::EmptyGroupName);
    }

    // Check if plan exists and validate
    {
        let plan = state_lock
            .current_plan
            .as_ref()
            .ok_or(GroupError::NoCurrentPlan)?;

        // Check if group already exists (for new groups)
        if plan.groups.contains_key(&name) {
            return Err(GroupError::GroupAlreadyExists(name));
        }

        // Validate all exercises exist in dictionary
        for exercise in &exercises {
            if !plan.dictionary.contains_key(exercise) {
                return Err(GroupError::ExerciseNotInDictionary(exercise.clone()));
            }
        }
    }

    // Save to undo history before making changes
    state_lock.save_to_undo_history();

    // Add the group
    let plan = state_lock.current_plan.as_mut().unwrap(); // Safe after validation above
    plan.groups.insert(name, exercises);

    // Mark as modified
    state_lock.mark_modified();

    Ok(())
}

/// Updates an existing exercise group
pub fn update_group(
    state: Arc<Mutex<AppState>>,
    name: String,
    exercises: Vec<String>,
) -> Result<(), GroupError> {
    let mut state_lock = state.lock().unwrap();

    // Validate input
    if name.trim().is_empty() {
        return Err(GroupError::EmptyGroupName);
    }

    // Check if plan exists and validate
    {
        let plan = state_lock
            .current_plan
            .as_ref()
            .ok_or(GroupError::NoCurrentPlan)?;

        // Check if group exists
        if !plan.groups.contains_key(&name) {
            return Err(GroupError::GroupNotFound(name));
        }

        // Validate all exercises exist in dictionary
        for exercise in &exercises {
            if !plan.dictionary.contains_key(exercise) {
                return Err(GroupError::ExerciseNotInDictionary(exercise.clone()));
            }
        }
    }

    // Save to undo history before making changes
    state_lock.save_to_undo_history();

    // Update the group
    let plan = state_lock.current_plan.as_mut().unwrap(); // Safe after validation above
    plan.groups.insert(name, exercises);

    // Mark as modified
    state_lock.mark_modified();

    Ok(())
}

/// Deletes an exercise group
pub fn delete_group(state: Arc<Mutex<AppState>>, name: String) -> Result<(), GroupError> {
    let mut state_lock = state.lock().unwrap();

    // Validate input
    if name.trim().is_empty() {
        return Err(GroupError::EmptyGroupName);
    }

    // Check if plan exists and validate
    {
        let plan = state_lock
            .current_plan
            .as_ref()
            .ok_or(GroupError::NoCurrentPlan)?;

        // Check if group exists
        if !plan.groups.contains_key(&name) {
            return Err(GroupError::GroupNotFound(name));
        }
    }

    // Save to undo history before making changes
    state_lock.save_to_undo_history();

    // Remove the group
    let plan = state_lock.current_plan.as_mut().unwrap(); // Safe after validation above
    plan.groups.remove(&name);

    // Mark as modified
    state_lock.mark_modified();

    Ok(())
}

/// Validates that all exercises in a list exist in the dictionary
pub fn validate_group_exercises(
    state: Arc<Mutex<AppState>>,
    exercises: &[String],
) -> Result<(), GroupError> {
    let state_lock = state.lock().unwrap();

    // Check if plan exists
    let plan = state_lock
        .current_plan
        .as_ref()
        .ok_or(GroupError::NoCurrentPlan)?;

    // Validate all exercises exist in dictionary
    for exercise in exercises {
        if !plan.dictionary.contains_key(exercise) {
            return Err(GroupError::ExerciseNotInDictionary(exercise.clone()));
        }
    }

    Ok(())
}

/// Gets all available groups for the current plan
#[allow(dead_code)]
pub fn get_all_groups(state: Arc<Mutex<AppState>>) -> Vec<(String, Vec<String>)> {
    let state_lock = state.lock().unwrap();

    if let Some(plan) = &state_lock.current_plan {
        plan.groups
            .iter()
            .map(|(name, exercises)| (name.clone(), exercises.clone()))
            .collect()
    } else {
        Vec::new()
    }
}

/// Checks if a group name exists
pub fn group_exists(state: Arc<Mutex<AppState>>, name: &str) -> bool {
    let state_lock = state.lock().unwrap();

    if let Some(plan) = &state_lock.current_plan {
        plan.groups.contains_key(name)
    } else {
        false
    }
}

/// Gets exercises for a specific group
#[allow(dead_code)]
pub fn get_group_exercises(state: Arc<Mutex<AppState>>, name: &str) -> Option<Vec<String>> {
    let state_lock = state.lock().unwrap();

    if let Some(plan) = &state_lock.current_plan {
        plan.groups.get(name).cloned()
    } else {
        None
    }
}

/// Adds an exercise to an existing group
#[allow(dead_code)]
pub fn add_exercise_to_group(
    state: Arc<Mutex<AppState>>,
    group_name: String,
    exercise_code: String,
) -> Result<(), GroupError> {
    let mut state_lock = state.lock().unwrap();

    // Check if plan exists and validate
    {
        let plan = state_lock
            .current_plan
            .as_ref()
            .ok_or(GroupError::NoCurrentPlan)?;

        // Check if group exists
        if !plan.groups.contains_key(&group_name) {
            return Err(GroupError::GroupNotFound(group_name));
        }

        // Validate exercise exists in dictionary
        if !plan.dictionary.contains_key(&exercise_code) {
            return Err(GroupError::ExerciseNotInDictionary(exercise_code));
        }
    }

    // Save to undo history before making changes
    state_lock.save_to_undo_history();

    // Add exercise to group if not already present
    let plan = state_lock.current_plan.as_mut().unwrap(); // Safe after validation above
    if let Some(exercises) = plan.groups.get_mut(&group_name) {
        if !exercises.contains(&exercise_code) {
            exercises.push(exercise_code);
        }
    }

    // Mark as modified
    state_lock.mark_modified();

    Ok(())
}

/// Removes an exercise from a group
#[allow(dead_code)]
pub fn remove_exercise_from_group(
    state: Arc<Mutex<AppState>>,
    group_name: String,
    exercise_code: String,
) -> Result<(), GroupError> {
    let mut state_lock = state.lock().unwrap();

    // Check if plan exists and validate
    {
        let plan = state_lock
            .current_plan
            .as_ref()
            .ok_or(GroupError::NoCurrentPlan)?;

        // Check if group exists
        if !plan.groups.contains_key(&group_name) {
            return Err(GroupError::GroupNotFound(group_name));
        }
    }

    // Save to undo history before making changes
    state_lock.save_to_undo_history();

    // Remove exercise from group
    let plan = state_lock.current_plan.as_mut().unwrap(); // Safe after validation above
    if let Some(exercises) = plan.groups.get_mut(&group_name) {
        exercises.retain(|e| e != &exercise_code);

        // Don't allow empty groups
        if exercises.is_empty() {
            return Err(GroupError::EmptyExercisesList);
        }
    }

    // Mark as modified
    state_lock.mark_modified();

    Ok(())
}
