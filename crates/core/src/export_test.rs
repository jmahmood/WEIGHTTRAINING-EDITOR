//! **Death to Windows!** - Export manifest conflict detection tests

use super::*;
use crate::{Plan, PlanVersion, VersionState, VersionMetadata};
use std::collections::HashMap;

#[test]
fn test_duplicate_plan_conflict() {
    let mut stager = ExportStager::new();
    
    // Create two plans with same ID but different versions
    let plan1 = Plan::new("Test Plan".to_string());
    let versioned1 = VersionedPlan {
        plan: plan1,
        version: PlanVersion::new(1, 0, 0),
        state: VersionState::Draft,
        metadata: VersionMetadata {
            created_at: "2024-01-01T00:00:00Z".to_string(),
            author: None,
            message: None,
            tags: vec!["test".to_string()],
            parent_version: None,
        },
    };
    
    let mut plan2 = Plan::new("Test Plan".to_string());
    plan2.dictionary.insert("EXERCISE.1".to_string(), "Exercise 1".to_string());
    let versioned2 = VersionedPlan {
        plan: plan2,
        version: PlanVersion::new(2, 0, 0),
        state: VersionState::Draft,
        metadata: VersionMetadata {
            created_at: "2024-01-01T00:00:00Z".to_string(),
            author: None,
            message: None,
            tags: vec!["test".to_string()],
            parent_version: None,
        },
    };
    
    // Add first plan - should succeed
    assert!(stager.add_plan("test_plan".to_string(), versioned1).is_ok());
    
    // Add second plan with same ID - should fail with conflict
    let result = stager.add_plan("test_plan".to_string(), versioned2);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Duplicate plan ID"));
    
    // Should have one conflict
    stager.analyze();
    let conflicts = stager.get_conflicts();
    assert_eq!(conflicts.len(), 1);
    assert!(matches!(conflicts[0].conflict_type, ConflictType::DuplicatePlan));
    assert_eq!(conflicts[0].severity, ConflictSeverity::Error);
}

#[test]
fn test_exercise_mismatch_conflict() {
    let mut stager = ExportStager::new();
    
    // Create two plans with same exercise code but different definitions
    let mut plan1 = Plan::new("Plan A".to_string());
    plan1.dictionary.insert("SQUAT".to_string(), "Back Squat".to_string());
    
    let mut plan2 = Plan::new("Plan B".to_string());
    plan2.dictionary.insert("SQUAT".to_string(), "Front Squat".to_string());
    
    let versioned1 = VersionedPlan {
        version: PlanVersion::new(1, 0, 0),
        plan: plan1,
    };
    
    let versioned2 = VersionedPlan {
        version: PlanVersion::new(1, 0, 0),
        plan: plan2,
    };
    
    // Add both plans
    assert!(stager.add_plan("plan_a".to_string(), versioned1).is_ok());
    assert!(stager.add_plan("plan_b".to_string(), versioned2).is_ok());
    
    // Analyze should detect exercise mismatch
    stager.analyze();
    let conflicts = stager.get_conflicts();
    
    assert_eq!(conflicts.len(), 1);
    assert!(matches!(conflicts[0].conflict_type, ConflictType::ExerciseMismatch));
    assert_eq!(conflicts[0].severity, ConflictSeverity::Warning);
    assert!(conflicts[0].description.contains("SQUAT"));
    assert!(conflicts[0].description.contains("plan_a"));
    assert!(conflicts[0].description.contains("plan_b"));
}

#[test]
fn test_missing_dependency_conflict() {
    let mut stager = ExportStager::new();
    
    // Create a plan that uses an exercise from another plan
    let mut plan1 = Plan::new("Plan A".to_string());
    plan1.dictionary.insert("SQUAT".to_string(), "Back Squat".to_string());
    
    let mut plan2 = Plan::new("Plan B".to_string());
    // Plan B uses SQUAT but doesn't define it - will create dependency
    
    let versioned1 = VersionedPlan {
        version: PlanVersion::new(1, 0, 0),
        plan: plan1,
    };
    
    let versioned2 = VersionedPlan {
        version: PlanVersion::new(1, 0, 0),
        plan: plan2,
    };
    
    // Add only plan B (which depends on plan A)
    assert!(stager.add_plan("plan_b".to_string(), versioned2).is_ok());
    
    // Analyze should detect missing dependency
    stager.analyze();
    let conflicts = stager.get_conflicts();
    
    // Should have missing dependency conflict
    assert_eq!(conflicts.len(), 1);
    assert!(matches!(conflicts[0].conflict_type, ConflictType::MissingDependency));
    assert_eq!(conflicts[0].severity, ConflictSeverity::Error);
}

#[test]
fn test_valid_export_with_dependencies() {
    let mut stager = ExportStager::new();
    
    // Create two plans with proper dependency relationship
    let mut plan1 = Plan::new("Base Plan".to_string());
    plan1.dictionary.insert("SQUAT".to_string(), "Back Squat".to_string());
    plan1.dictionary.insert("BENCH".to_string(), "Bench Press".to_string());
    
    let mut plan2 = Plan::new("Specialized Plan".to_string());
    plan2.dictionary.insert("DEADLIFT".to_string(), "Deadlift".to_string());
    // Plan2 uses SQUAT from plan1
    
    let versioned1 = VersionedPlan {
        version: PlanVersion::new(1, 0, 0),
        plan: plan1,
    };
    
    let versioned2 = VersionedPlan {
        version: PlanVersion::new(1, 0, 0),
        plan: plan2,
    };
    
    // Add both plans
    assert!(stager.add_plan("base_plan".to_string(), versioned1).is_ok());
    assert!(stager.add_plan("specialized_plan".to_string(), versioned2).is_ok());
    
    // Analyze should detect dependency but no conflicts
    stager.analyze();
    
    let conflicts = stager.get_conflicts();
    let dependencies = stager.get_dependencies();
    
    // Should have no conflicts
    assert_eq!(conflicts.len(), 0);
    
    // Should have one dependency
    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies[0].from_plan, "specialized_plan");
    assert_eq!(dependencies[0].to_plan, "base_plan");
    assert!(matches!(dependencies[0].dependency_type, DependencyType::Exercise));
    
    // Export should be allowed
    assert!(stager.can_export());
}

#[test]
fn test_manifest_generation() {
    let mut stager = ExportStager::new();
    
    // Create a simple plan
    let plan = Plan::new("Test Plan".to_string());
    let versioned = VersionedPlan {
        version: PlanVersion::new(1, 0, 0),
        plan,
    };
    
    assert!(stager.add_plan("test_plan".to_string(), versioned).is_ok());
    stager.analyze();
    
    // Generate manifest
    let manifest = stager.generate_manifest(
        Some("Test Author".to_string()), 
        Some("Test export".to_string())
    );
    
    // Verify manifest structure
    assert_eq!(manifest.format_version, "1.0.0");
    assert_eq!(manifest.author, Some("Test Author".to_string()));
    assert_eq!(manifest.description, Some("Test export".to_string()));
    assert_eq!(manifest.plans.len(), 1);
    assert_eq!(manifest.plans[0].id, "test_plan");
    assert_eq!(manifest.plans[0].name, "Test Plan");
    assert_eq!(manifest.dependencies.len(), 0);
    assert_eq!(manifest.conflicts.len(), 0);
    
    // Verify metadata
    assert_eq!(manifest.metadata.total_plans, 1);
    assert_eq!(manifest.metadata.total_exercises, 0); // No exercises in test plan
    assert_eq!(manifest.metadata.total_segments, 0);  // No segments in test plan
}