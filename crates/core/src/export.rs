/// **Death to Windows!** - Export staging and manifest generation for Sprint 2
/// Multi-plan export with conflict detection and dependency management
use serde::{Deserialize, Serialize};
use crate::{Plan, PlanVersion, VersionedPlan};
use std::collections::{HashMap, HashSet};

/// Export manifest describing a collection of plans
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportManifest {
    pub format_version: String,
    pub created_at: String,
    pub author: Option<String>,
    pub description: Option<String>,
    pub plans: Vec<ExportPlanInfo>,
    pub dependencies: Vec<ExportDependency>,
    pub conflicts: Vec<ExportConflict>,
    pub metadata: ExportMetadata,
}

/// Information about a plan in the export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportPlanInfo {
    pub id: String,
    pub name: String,
    pub version: PlanVersion,
    pub path: String,           // Relative path in export
    pub checksum: String,       // SHA-256 hash of plan content
    pub size_bytes: u64,        // File size
    pub exercises: Vec<String>, // Exercise codes used
}

/// Dependency between plans in the export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportDependency {
    pub from_plan: String,      // Plan ID that depends
    pub to_plan: String,        // Plan ID being depended on
    pub dependency_type: DependencyType,
    pub description: String,
}

/// Type of dependency between plans
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DependencyType {
    Exercise,     // Shared exercise reference
    Template,     // Shared scheme template
    Sequence,     // Plan ordering dependency
}

/// Conflict detected during export staging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportConflict {
    pub conflict_type: ConflictType,
    pub description: String,
    pub affected_plans: Vec<String>,
    pub severity: ConflictSeverity,
    pub resolution: Option<String>,
}

/// Type of export conflict
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConflictType {
    ExerciseMismatch,    // Same exercise code, different definitions
    VersionMismatch,     // Incompatible plan versions
    DuplicatePlan,       // Same plan ID with different content
    MissingDependency,   // Referenced plan not included
}

/// Severity of export conflict
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConflictSeverity {
    Info,       // Informational, export can proceed
    Warning,    // Potential issue, user should review
    Error,      // Blocking issue, export cannot proceed
}

/// Export metadata and statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportMetadata {
    pub total_plans: u32,
    pub total_exercises: u32,
    pub total_segments: u32,
    pub size_bytes: u64,
    pub compression: Option<CompressionInfo>,
    pub validation: ValidationSummary,
}

/// Compression information for the export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionInfo {
    pub algorithm: String,      // e.g., "gzip", "zip"
    pub original_size: u64,
    pub compressed_size: u64,
    pub ratio: f64,             // Compression ratio
}

/// Validation summary for all plans in export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationSummary {
    pub total_errors: u32,
    pub total_warnings: u32,
    pub validated_plans: u32,
    pub validation_time_ms: u64,
}

/// Export staging area for building exports
pub struct ExportStager {
    plans: HashMap<String, VersionedPlan>,
    conflicts: Vec<ExportConflict>,
    dependencies: Vec<ExportDependency>,
}

impl ExportStager {
    pub fn new() -> Self {
        Self {
            plans: HashMap::new(),
            conflicts: Vec::new(),
            dependencies: Vec::new(),
        }
    }
    
    /// Add a plan to the export staging area
    pub fn add_plan(&mut self, plan_id: String, versioned_plan: VersionedPlan) -> Result<(), String> {
        // Check for duplicate plan IDs
        if let Some(existing) = self.plans.get(&plan_id) {
            if existing.version.to_string() != versioned_plan.version.to_string() {
                self.conflicts.push(ExportConflict {
                    conflict_type: ConflictType::DuplicatePlan,
                    description: format!(
                        "Plan '{}' already exists with version {} but trying to add version {}",
                        plan_id, existing.version, versioned_plan.version
                    ),
                    affected_plans: vec![plan_id.clone()],
                    severity: ConflictSeverity::Error,
                    resolution: Some("Choose one version or rename one of the plans".to_string()),
                });
                return Err("Duplicate plan ID with different versions".to_string());
            }
        }
        
        self.plans.insert(plan_id, versioned_plan);
        Ok(())
    }
    
    /// Analyze dependencies and conflicts across staged plans
    pub fn analyze(&mut self) {
        self.detect_exercise_conflicts();
        self.detect_dependencies();
        self.detect_missing_dependencies();
    }
    
    /// Detect conflicts in exercise definitions
    fn detect_exercise_conflicts(&mut self) {
        let mut exercise_definitions: HashMap<String, (String, serde_json::Value)> = HashMap::new();
        
        for (plan_id, versioned_plan) in &self.plans {
            for (exercise_code, exercise_def) in &versioned_plan.plan.dictionary {
                let exercise_json = serde_json::to_value(exercise_def).unwrap_or_default();
                
                if let Some((existing_plan, existing_def)) = exercise_definitions.get(exercise_code) {
                    if existing_def != &exercise_json {
                        self.conflicts.push(ExportConflict {
                            conflict_type: ConflictType::ExerciseMismatch,
                            description: format!(
                                "Exercise '{}' has different definitions in plans '{}' and '{}'",
                                exercise_code, existing_plan, plan_id
                            ),
                            affected_plans: vec![existing_plan.clone(), plan_id.clone()],
                            severity: ConflictSeverity::Warning,
                            resolution: Some("Review exercise definitions and ensure consistency".to_string()),
                        });
                    }
                } else {
                    exercise_definitions.insert(exercise_code.clone(), (plan_id.clone(), exercise_json));
                }
            }
        }
    }
    
    /// Detect dependencies between plans
    fn detect_dependencies(&mut self) {
        for (plan_id, versioned_plan) in &self.plans {
            let plan_exercises: HashSet<String> = versioned_plan.plan.dictionary.keys().cloned().collect();
            let used_exercises = self.extract_used_exercises(&versioned_plan.plan);
            
            // Find exercises used but not defined in this plan
            for used_exercise in &used_exercises {
                if !plan_exercises.contains(used_exercise) {
                    // Find which other plans provide this exercise
                    for (other_plan_id, other_versioned_plan) in &self.plans {
                        if other_plan_id != plan_id && other_versioned_plan.plan.dictionary.contains_key(used_exercise) {
                            self.dependencies.push(ExportDependency {
                                from_plan: plan_id.clone(),
                                to_plan: other_plan_id.clone(),
                                dependency_type: DependencyType::Exercise,
                                description: format!(
                                    "Plan '{}' uses exercise '{}' defined in plan '{}'",
                                    plan_id, used_exercise, other_plan_id
                                ),
                            });
                        }
                    }
                }
            }
        }
    }
    
    /// Extract all exercise codes used in a plan's segments
    fn extract_used_exercises(&self, plan: &Plan) -> HashSet<String> {
        let mut used = HashSet::new();
        
        for day in &plan.schedule {
            for segment in &day.segments {
                Self::extract_exercises_from_segment(segment, &mut used);
            }
        }
        
        used
    }
    
    /// Recursively extract exercise codes from a segment
    fn extract_exercises_from_segment(segment: &crate::Segment, used: &mut HashSet<String>) {
        use crate::Segment;
        
        match segment {
            Segment::Straight(s) => { used.insert(s.base.ex.clone()); },
            Segment::Rpe(s) => { used.insert(s.base.ex.clone()); },
            Segment::Percentage(s) => { used.insert(s.base.ex.clone()); },
            Segment::Amrap(s) => { used.insert(s.base.ex.clone()); },
            Segment::Scheme(s) => { used.insert(s.base.ex.clone()); },
            Segment::Time(s) => { used.insert(s.base.ex.clone()); },
            Segment::GroupChoose(g) => {
                for sub_segment in &g.from {
                    Self::extract_exercises_from_segment(sub_segment, used);
                }
            },
            Segment::GroupRotate(g) => {
                for sub_segment in &g.items {
                    Self::extract_exercises_from_segment(sub_segment, used);
                }
            },
            Segment::GroupOptional(g) => {
                for sub_segment in &g.items {
                    Self::extract_exercises_from_segment(sub_segment, used);
                }
            },
            Segment::GroupSuperset(g) => {
                for sub_segment in &g.items {
                    Self::extract_exercises_from_segment(sub_segment, used);
                }
            },
            _ => {
                // Handle other segment types as needed
            }
        }
    }
    
    /// Detect missing dependencies
    fn detect_missing_dependencies(&mut self) {
        for dependency in &self.dependencies.clone() {
            if !self.plans.contains_key(&dependency.to_plan) {
                self.conflicts.push(ExportConflict {
                    conflict_type: ConflictType::MissingDependency,
                    description: format!(
                        "Plan '{}' depends on plan '{}' which is not included in the export",
                        dependency.from_plan, dependency.to_plan
                    ),
                    affected_plans: vec![dependency.from_plan.clone()],
                    severity: ConflictSeverity::Error,
                    resolution: Some(format!("Add plan '{}' to the export or remove the dependency", dependency.to_plan)),
                });
            }
        }
    }
    
    /// Generate export manifest
    pub fn generate_manifest(&self, author: Option<String>, description: Option<String>) -> ExportManifest {
        let mut plan_infos = Vec::new();
        let mut total_segments = 0u32;
        let mut total_exercises = HashSet::new();
        let mut total_size = 0u64;
        
        for (plan_id, versioned_plan) in &self.plans {
            let plan_json = serde_json::to_string(&versioned_plan.plan).unwrap_or_default();
            let plan_bytes = plan_json.as_bytes();
            let checksum = format!("{:x}", md5::compute(plan_bytes)); // Simple checksum
            
            // Count segments in this plan
            let plan_segments: u32 = versioned_plan.plan.schedule.iter()
                .map(|day| day.segments.len() as u32)
                .sum();
            total_segments += plan_segments;
            
            // Collect exercises
            for exercise_code in versioned_plan.plan.dictionary.keys() {
                total_exercises.insert(exercise_code.clone());
            }
            
            total_size += plan_bytes.len() as u64;
            
            plan_infos.push(ExportPlanInfo {
                id: plan_id.clone(),
                name: versioned_plan.plan.name.clone(),
                version: versioned_plan.version.clone(),
                path: format!("plans/{}.json", plan_id),
                checksum,
                size_bytes: plan_bytes.len() as u64,
                exercises: versioned_plan.plan.dictionary.keys().cloned().collect(),
            });
        }
        
        ExportManifest {
            format_version: "1.0.0".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            author,
            description,
            plans: plan_infos,
            dependencies: self.dependencies.clone(),
            conflicts: self.conflicts.clone(),
            metadata: ExportMetadata {
                total_plans: self.plans.len() as u32,
                total_exercises: total_exercises.len() as u32,
                total_segments,
                size_bytes: total_size,
                compression: None, // Would be filled during actual export
                validation: ValidationSummary {
                    total_errors: 0,   // Would be filled by validator
                    total_warnings: 0, // Would be filled by validator
                    validated_plans: self.plans.len() as u32,
                    validation_time_ms: 0, // Would be filled by validator
                },
            },
        }
    }
    
    /// Check if export can proceed (no error-level conflicts)
    pub fn can_export(&self) -> bool {
        !self.conflicts.iter().any(|c| matches!(c.severity, ConflictSeverity::Error))
    }
    
    /// Get all conflicts
    pub fn get_conflicts(&self) -> &[ExportConflict] {
        &self.conflicts
    }
    
    /// Get all dependencies
    pub fn get_dependencies(&self) -> &[ExportDependency] {
        &self.dependencies
    }
    
    /// Get staged plans
    pub fn get_plans(&self) -> &HashMap<String, VersionedPlan> {
        &self.plans
    }
}

impl Default for ExportStager {
    fn default() -> Self {
        Self::new()
    }
}
