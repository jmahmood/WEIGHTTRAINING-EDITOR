/// **Death to Windows!** - Versioning system for Sprint 2
/// Draft → Diff → Promote workflow for plan management
use serde::{Deserialize, Serialize};
use crate::Plan;
use std::collections::HashMap;

/// Version states in the Draft → Diff → Promote workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VersionState {
    Draft,      // Working copy with unsaved changes
    Staged,     // Changes staged for promotion
    Promoted,   // Final stable version
}

/// Versioned plan with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionedPlan {
    pub plan: Plan,
    pub version: PlanVersion,
    pub state: VersionState,
    pub metadata: VersionMetadata,
}

/// Version information for a plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub build: Option<String>,    // Git commit hash or build identifier
}

/// Metadata about version changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionMetadata {
    pub created_at: String,       // ISO 8601 timestamp
    pub author: Option<String>,   // Author name
    pub message: Option<String>,  // Commit-style message
    pub tags: Vec<String>,        // Version tags (e.g., "stable", "experimental")
    pub parent_version: Option<PlanVersion>, // Previous version for diff calculation
}

/// Change type for diff tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChangeType {
    Added,
    Removed,
    Modified,
}

/// Individual change in a diff
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanChange {
    pub change_type: ChangeType,
    pub path: String,            // JSON pointer path to changed field
    pub old_value: Option<serde_json::Value>,
    pub new_value: Option<serde_json::Value>,
    pub description: String,     // Human-readable description
}

/// Complete diff between two plan versions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanDiff {
    pub from_version: PlanVersion,
    pub to_version: PlanVersion,
    pub changes: Vec<PlanChange>,
    pub metrics: DiffMetrics,
}

/// Aggregate metrics about a diff
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffMetrics {
    pub total_changes: u32,
    pub additions: u32,
    pub modifications: u32,
    pub deletions: u32,
    pub segments_added: u32,
    pub segments_removed: u32,
    pub segments_modified: u32,
    pub exercises_added: u32,
    pub exercises_removed: u32,
}

/// Version management for plans
pub struct PlanVersionManager {
    versions: HashMap<String, Vec<VersionedPlan>>, // plan_id -> versions
}

impl PlanVersion {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
            build: None,
        }
    }
    
    pub fn with_build(mut self, build: String) -> Self {
        self.build = Some(build);
        self
    }
    
    pub fn bump_patch(&mut self) {
        self.patch += 1;
    }
    
    pub fn bump_minor(&mut self) {
        self.minor += 1;
        self.patch = 0;
    }
    
    pub fn bump_major(&mut self) {
        self.major += 1;
        self.minor = 0;
        self.patch = 0;
    }
}

impl std::fmt::Display for PlanVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.build {
            Some(build) => write!(f, "{}.{}.{}+{}", self.major, self.minor, self.patch, build),
            None => write!(f, "{}.{}.{}", self.major, self.minor, self.patch),
        }
    }
}

impl VersionedPlan {
    pub fn new_draft(plan: Plan, version: PlanVersion) -> Self {
        Self {
            plan,
            version,
            state: VersionState::Draft,
            metadata: VersionMetadata {
                created_at: chrono::Utc::now().to_rfc3339(),
                author: None,
                message: None,
                tags: vec!["draft".to_string()],
                parent_version: None,
            },
        }
    }
    
    pub fn stage_for_promotion(&mut self, message: Option<String>) {
        self.state = VersionState::Staged;
        self.metadata.message = message;
        if !self.metadata.tags.contains(&"staged".to_string()) {
            self.metadata.tags.push("staged".to_string());
        }
    }
    
    pub fn promote(&mut self, author: Option<String>) {
        self.state = VersionState::Promoted;
        self.metadata.author = author;
        self.metadata.tags.retain(|tag| tag != "draft" && tag != "staged");
        self.metadata.tags.push("promoted".to_string());
    }
}

impl PlanVersionManager {
    pub fn new() -> Self {
        Self {
            versions: HashMap::new(),
        }
    }
    
    /// Create a new draft version of a plan
    pub fn create_draft(&mut self, plan_id: String, plan: Plan) -> Result<VersionedPlan, String> {
        let version = match self.get_latest_version(&plan_id) {
            Some(latest) => {
                let mut new_version = latest.version.clone();
                new_version.bump_patch();
                new_version
            },
            None => PlanVersion::new(1, 0, 0),
        };
        
        let versioned_plan = VersionedPlan::new_draft(plan, version);
        
        self.versions.entry(plan_id).or_default().push(versioned_plan.clone());
        
        Ok(versioned_plan)
    }
    
    /// Get the latest version of a plan
    pub fn get_latest_version(&self, plan_id: &str) -> Option<&VersionedPlan> {
        self.versions.get(plan_id)?.last()
    }
    
    /// Get all versions of a plan
    pub fn get_versions(&self, plan_id: &str) -> Option<&Vec<VersionedPlan>> {
        self.versions.get(plan_id)
    }
    
    /// Compute diff between two versions
    pub fn compute_diff(&self, plan_id: &str, from_version: &PlanVersion, to_version: &PlanVersion) -> Result<PlanDiff, String> {
        let versions = self.get_versions(plan_id)
            .ok_or_else(|| "Plan not found".to_string())?;
        
        let from_plan = versions.iter()
            .find(|v| v.version.major == from_version.major && 
                     v.version.minor == from_version.minor && 
                     v.version.patch == from_version.patch)
            .ok_or_else(|| "From version not found".to_string())?;
        
        let to_plan = versions.iter()
            .find(|v| v.version.major == to_version.major && 
                     v.version.minor == to_version.minor && 
                     v.version.patch == to_version.patch)
            .ok_or_else(|| "To version not found".to_string())?;
        
        self.diff_plans(&from_plan.plan, &to_plan.plan, from_version.clone(), to_version.clone())
    }
    
    /// Compute diff between two plans
    fn diff_plans(&self, from: &Plan, to: &Plan, from_version: PlanVersion, to_version: PlanVersion) -> Result<PlanDiff, String> {
        let mut changes = Vec::new();
        let mut metrics = DiffMetrics {
            total_changes: 0,
            additions: 0,
            modifications: 0,
            deletions: 0,
            segments_added: 0,
            segments_removed: 0,
            segments_modified: 0,
            exercises_added: 0,
            exercises_removed: 0,
        };
        
        // Convert plans to JSON for deep comparison
        let from_json = serde_json::to_value(from)
            .map_err(|e| format!("Failed to serialize from plan: {}", e))?;
        let to_json = serde_json::to_value(to)
            .map_err(|e| format!("Failed to serialize to plan: {}", e))?;
        
        // Compare basic plan metadata
        Self::diff_values(&from_json, &to_json, "", &mut changes, &mut metrics);
        
        // Compute specific metrics for segments and exercises
        self.compute_segment_metrics(from, to, &mut metrics);
        self.compute_exercise_metrics(from, to, &mut metrics);
        
        metrics.total_changes = changes.len() as u32;
        
        Ok(PlanDiff {
            from_version,
            to_version,
            changes,
            metrics,
        })
    }
    
    /// Recursively compare JSON values and generate changes
    fn diff_values(from: &serde_json::Value, to: &serde_json::Value, path: &str, changes: &mut Vec<PlanChange>, metrics: &mut DiffMetrics) {
        use serde_json::Value;
        
        match (from, to) {
            (Value::Object(from_obj), Value::Object(to_obj)) => {
                // Check for removed fields
                for (key, from_value) in from_obj {
                    let current_path = if path.is_empty() { 
                        format!("/{}", key) 
                    } else { 
                        format!("{}/{}", path, key) 
                    };
                    
                    if !to_obj.contains_key(key) {
                        changes.push(PlanChange {
                            change_type: ChangeType::Removed,
                            path: current_path,
                            old_value: Some(from_value.clone()),
                            new_value: None,
                            description: format!("Removed field '{}'", key),
                        });
                        metrics.deletions += 1;
                    }
                }
                
                // Check for added or modified fields
                for (key, to_value) in to_obj {
                    let current_path = if path.is_empty() { 
                        format!("/{}", key) 
                    } else { 
                        format!("{}/{}", path, key) 
                    };
                    
                    match from_obj.get(key) {
                        None => {
                            changes.push(PlanChange {
                                change_type: ChangeType::Added,
                                path: current_path,
                                old_value: None,
                                new_value: Some(to_value.clone()),
                                description: format!("Added field '{}'", key),
                            });
                            metrics.additions += 1;
                        },
                        Some(from_value) => {
                            if from_value != to_value {
                                Self::diff_values(from_value, to_value, &current_path, changes, metrics);
                            }
                        }
                    }
                }
            },
            (Value::Array(from_arr), Value::Array(to_arr)) => {
                // Simple array comparison - could be enhanced with LCS algorithm
                if from_arr.len() != to_arr.len() || from_arr.iter().zip(to_arr.iter()).any(|(a, b)| a != b) {
                    changes.push(PlanChange {
                        change_type: ChangeType::Modified,
                        path: path.to_string(),
                        old_value: Some(from.clone()),
                        new_value: Some(to.clone()),
                        description: format!("Modified array at '{}'", path),
                    });
                    metrics.modifications += 1;
                }
            },
            _ => {
                if from != to {
                    changes.push(PlanChange {
                        change_type: ChangeType::Modified,
                        path: path.to_string(),
                        old_value: Some(from.clone()),
                        new_value: Some(to.clone()),
                        description: format!("Modified value at '{}'", path),
                    });
                    metrics.modifications += 1;
                }
            }
        }
    }
    
    /// Compute metrics specific to segments
    fn compute_segment_metrics(&self, from: &Plan, to: &Plan, metrics: &mut DiffMetrics) {
        let from_segments: u32 = from.schedule.iter().map(|day| day.segments.len() as u32).sum();
        let to_segments: u32 = to.schedule.iter().map(|day| day.segments.len() as u32).sum();
        
        if to_segments > from_segments {
            metrics.segments_added = to_segments - from_segments;
        } else if from_segments > to_segments {
            metrics.segments_removed = from_segments - to_segments;
        }
        
        // This is a simplified metric - could be enhanced with actual segment-by-segment comparison
        if from_segments != to_segments {
            metrics.segments_modified = 1;
        }
    }
    
    /// Compute metrics specific to exercises
    fn compute_exercise_metrics(&self, from: &Plan, to: &Plan, metrics: &mut DiffMetrics) {
        let from_exercises: std::collections::HashSet<_> = from.dictionary.keys().collect();
        let to_exercises: std::collections::HashSet<_> = to.dictionary.keys().collect();
        
        metrics.exercises_added = to_exercises.difference(&from_exercises).count() as u32;
        metrics.exercises_removed = from_exercises.difference(&to_exercises).count() as u32;
    }
    
    /// Promote a staged version
    pub fn promote_version(&mut self, plan_id: &str, version: &PlanVersion, author: Option<String>) -> Result<(), String> {
        let versions = self.versions.get_mut(plan_id)
            .ok_or_else(|| "Plan not found".to_string())?;
        
        let versioned_plan = versions.iter_mut()
            .find(|v| v.version.major == version.major && 
                     v.version.minor == version.minor && 
                     v.version.patch == version.patch)
            .ok_or_else(|| "Version not found".to_string())?;
        
        if !matches!(versioned_plan.state, VersionState::Staged) {
            return Err("Only staged versions can be promoted".to_string());
        }
        
        versioned_plan.promote(author);
        Ok(())
    }
}

impl Default for PlanVersionManager {
    fn default() -> Self {
        Self::new()
    }
}