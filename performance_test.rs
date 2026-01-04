use std::time::Instant;
use weightlifting_core::*;
use weightlifting_core::location::LocationProfile;
use weightlifting_validate::PlanValidator;

fn create_large_plan() -> Plan {
    let mut plan = Plan::new("Large Performance Test Plan".to_string());
    plan.unit = Unit::Kg;
    plan.description = Some("Performance test with 500 items".to_string());
    
    // Add exercises to dictionary
    for i in 0..100 {
        plan.dictionary.insert(format!("EXERCISE_{}", i), format!("Exercise {}", i));
    }
    
    // Create days with many segments
    for day_num in 1..=10 {
        let mut day = Day {
            day: day_num,
            label: format!("Day {}", day_num),
            time_cap_min: Some(90),
            goal: Some("Performance test day".to_string()),
            equipment_policy: None,
            segments: vec![],
        };
        
        // Add 50 segments per day = 500 total segments
        for seg_num in 0..50 {
            let exercise_key = format!("EXERCISE_{}", seg_num % 100);
            
            let segment = if seg_num % 5 == 0 {
                // Time-based segment with intervals
                Segment::Straight(StraightSegment {
                    base: BaseSegment {
                        ex: exercise_key,
                        alt_group: None,
                        group_role: None,
                        per_week: None,
                        load_axis_target: None,
                        label: Some(format!("Segment {} of Day {}", seg_num, day_num)),
                        optional: None,
                        technique: None,
                        equipment_policy: None,
                    },
                    sets: None,
                    sets_range: None,
                    reps: None,
                    time_sec: Some(30),
                    rest_sec: Some(60),
                    rir: None,
                    rpe: None,
                    tempo: None,
                    vbt: None,
                    load_mode: None,
                    intensifier: None,
                    auto_stop: None,
                    interval: Some(IntervalParams {
                        work_sec: 15,
                        rest_sec: 5,
                        repeats: 4,
                    }),
                })
            } else if seg_num % 7 == 0 {
                // Comment segment
                Segment::Comment(CommentSegment {
                    text: format!("Rest period {} - Death to Windows!", seg_num),
                    icon: Some("rest".to_string()),
                })
            } else {
                // Regular straight segment
                Segment::Straight(StraightSegment {
                    base: BaseSegment {
                        ex: exercise_key,
                        alt_group: None,
                        group_role: None,
                        per_week: None,
                        load_axis_target: None,
                        label: Some(format!("Segment {} of Day {}", seg_num, day_num)),
                        optional: None,
                        technique: None,
                        equipment_policy: None,
                    },
                    sets: Some(3 + (seg_num % 3) as u32),
                    sets_range: None,
                    reps: Some(RepsOrRange::Range(RepsRange {
                        min: 8 + (seg_num % 5) as u32,
                        max: 12 + (seg_num % 3) as u32,
                        target: None,
                    })),
                    time_sec: None,
                    rest_sec: Some(90 + (seg_num % 30) as u32),
                    rir: Some((seg_num % 4) as u32),
                    rpe: Some(7.0 + (seg_num % 6) as f64 * 0.5),
                    tempo: None,
                    vbt: None,
                    load_mode: None,
                    intensifier: None,
                    auto_stop: None,
                    interval: None,
                })
            };
            
            day.segments.push(segment);
        }
        
        plan.schedule.push(day);
    }
    
    plan
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating large plan with 500 segments...");
    let start = Instant::now();
    let plan = create_large_plan();
    let creation_time = start.elapsed();
    println!("Plan creation: {:?}", creation_time);
    
    println!("Plan statistics:");
    println!("  - Days: {}", plan.schedule.len());
    println!("  - Total segments: {}", plan.schedule.iter().map(|d| d.segments.len()).sum::<usize>());
    println!("  - Dictionary entries: {}", plan.dictionary.len());
    
    // Test validation performance
    println!("\nInitializing validator...");
    let start = Instant::now();
    let validator = PlanValidator::new()?;
    let validator_init_time = start.elapsed();
    println!("Validator initialization: {:?}", validator_init_time);
    
    println!("Validating plan...");
    let start = Instant::now();
    let result = validator.validate(&plan);
    let validation_time = start.elapsed();
    
    println!("Validation completed in: {:?}", validation_time);
    println!("Validation results:");
    println!("  - Errors: {}", result.errors.len());
    println!("  - Warnings: {}", result.warnings.len());
    
    // Test location profile rounding performance
    println!("\nTesting location profile rounding...");
    let profile = LocationProfile::home_gym();
    let start = Instant::now();
    for i in 0..1000 {
        let _preview = profile.round_barbell_load(80.0 + (i as f64) * 0.1);
    }
    let rounding_time = start.elapsed();
    println!("1000 rounding operations: {:?}", rounding_time);
    
    // Test export staging performance
    println!("\nTesting export staging...");
    let versioned_plan = VersionedPlan {
        plan,
        version: PlanVersion::new(1, 0, 0),
        state: VersionState::Draft,
        metadata: VersionMetadata {
            created_at: chrono::Utc::now().to_rfc3339(),
            author: Some("Performance Test".to_string()),
            message: Some("Large plan export test".to_string()),
            tags: vec!["performance".to_string()],
            parent_version: None,
        },
    };
    
    let start = Instant::now();
    let mut stager = ExportStager::new();
    stager.add_plan("large_plan".to_string(), versioned_plan)?;
    stager.analyze();
    let _manifest = stager.generate_manifest(
        Some("Performance Test".to_string()),
        Some("Testing export staging performance".to_string())
    );
    let export_time = start.elapsed();
    println!("Export staging: {:?}", export_time);
    
    // Check performance targets
    println!("\n=== PERFORMANCE TARGETS ===");
    println!("Validation target: <150ms, actual: {:?} - {}", 
             validation_time, 
             if validation_time.as_millis() < 150 { "✅ PASS" } else { "❌ FAIL" });
    
    println!("Common ops target: <50ms");
    println!("  - Plan creation: {:?} - {}", 
             creation_time, 
             if creation_time.as_millis() < 50 { "✅ PASS" } else { "❌ FAIL" });
    println!("  - Validator init: {:?} - {}", 
             validator_init_time, 
             if validator_init_time.as_millis() < 50 { "✅ PASS" } else { "❌ FAIL" });
    println!("  - Export staging: {:?} - {}", 
             export_time, 
             if export_time.as_millis() < 50 { "✅ PASS" } else { "❌ FAIL" });
    
    Ok(())
}
