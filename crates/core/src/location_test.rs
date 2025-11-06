//! **Death to Windows!** - Rounding preview unit tests

use super::*;
use crate::location::LocationProfile;

#[test]
fn test_barbell_rounding_nearest() {
    let profile = LocationProfile::home_gym();
    
    // Test exact match
    let preview = profile.round_barbell_load(100.0);
    assert_eq!(preview.original_load, 100.0);
    assert_eq!(preview.rounded_load, 100.0);
    assert!(preview.delta.abs() < 0.01);
    assert_eq!(preview.equipment, "bb");
    
    // Test rounding - use values that should round to available plates
    let preview = profile.round_barbell_load(95.0);
    assert_eq!(preview.original_load, 95.0);
    // 95.0 should round to 95.0 (exact match with plates)
    println!("95.0 rounded to: {}", preview.rounded_load);
    assert!((preview.rounded_load - 95.0).abs() < 0.01);
    
    // Test another value that should round  
    let preview = profile.round_barbell_load(100.0);
    assert_eq!(preview.original_load, 100.0);
    // 100.0 should round to 100.0 (exact match)
    println!("100.0 rounded to: {}", preview.rounded_load);
    assert!((preview.rounded_load - 100.0).abs() < 0.01);
}

#[test]
fn test_barbell_rounding_up() {
    let mut profile = LocationProfile::home_gym();
    profile.strategy = RoundingStrategy::Up;
    
    let preview = profile.round_barbell_load(92.5);
    assert_eq!(preview.original_load, 92.5);
    println!("92.5 rounded up to: {}", preview.rounded_load);
    println!("Delta: {}", preview.delta);
    // Should round up to next available weight or stay the same if exact
    assert!(preview.rounded_load >= 92.5);
    assert!(preview.delta >= 0.0);
    
    let preview = profile.round_barbell_load(100.0);
    assert_eq!(preview.original_load, 100.0);
    assert_eq!(preview.rounded_load, 100.0);
    assert!(preview.delta.abs() < 0.01);
}

#[test]
fn test_barbell_rounding_down() {
    let mut profile = LocationProfile::home_gym();
    profile.strategy = RoundingStrategy::Down;
    
    let preview = profile.round_barbell_load(97.5);
    assert_eq!(preview.original_load, 97.5);
    println!("97.5 rounded down to: {}", preview.rounded_load);
    println!("Delta: {}", preview.delta);
    // Should round down to previous available weight or stay the same if exact
    assert!(preview.rounded_load <= 97.5);
    assert!(preview.delta <= 0.0);
    
    let preview = profile.round_barbell_load(100.0);
    assert_eq!(preview.original_load, 100.0);
    assert_eq!(preview.rounded_load, 100.0);
    assert!(preview.delta.abs() < 0.01);
}

#[test]
fn test_dumbbell_rounding() {
    let profile = LocationProfile::home_gym();
    
    // Test exact match
    let preview = profile.round_dumbbell_load(20.0);
    assert_eq!(preview.original_load, 20.0);
    assert_eq!(preview.rounded_load, 20.0);
    assert!(preview.delta.abs() < 0.01);
    assert_eq!(preview.equipment, "db");
    
    // Test rounding
    let preview = profile.round_dumbbell_load(18.0);
    assert_eq!(preview.original_load, 18.0);
    assert_eq!(preview.rounded_load, 17.5); // Should round to nearest available
    assert!((preview.delta + 0.5).abs() < 0.01);
}

#[test]
fn test_rounding_preview_format() {
    let preview = RoundingPreview {
        original_load: 100.0,
        rounded_load: 100.0,
        delta: 0.0,
        equipment: "bb".to_string(),
        plate_solution: None,
    };
    
    assert!(preview.format_preview().contains("exact"));
    
    let preview = RoundingPreview {
        original_load: 97.5,
        rounded_load: 100.0,
        delta: 2.5,
        equipment: "bb".to_string(),
        plate_solution: None,
    };
    
    assert!(preview.format_preview().contains("+2.5kg"));
    
    let preview = RoundingPreview {
        original_load: 102.5,
        rounded_load: 100.0,
        delta: -2.5,
        equipment: "bb".to_string(),
        plate_solution: None,
    };
    
    assert!(preview.format_preview().contains("-2.5kg"));
}

#[test]
fn test_plate_solution_calculation() {
    let profile = LocationProfile::home_gym();
    
    // Test with a load that should use plates
    let preview = profile.round_barbell_load(120.0);
    assert_eq!(preview.original_load, 120.0);
    assert_eq!(preview.rounded_load, 120.0); // 20kg bar + 2x20kg + 2x10kg per side
    assert!(preview.delta.abs() < 0.01);
    assert!(preview.plate_solution.is_some());
    
    if let Some(solution) = preview.plate_solution {
        assert_eq!(solution.bar_weight, 20.0);
        // Should have 20kg and 10kg plates per side
        assert!(solution.plates_per_side.iter().any(|(w, _)| (*w - 20.0).abs() < 0.1));
        assert!(solution.plates_per_side.iter().any(|(w, _)| (*w - 10.0).abs() < 0.1));
    }
}