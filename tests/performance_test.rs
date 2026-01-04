use std::time::Instant;
use weightlifting_core::*;
use weightlifting_validate::PlanValidator;

#[test]
fn test_500_item_validation_performance() {
    // Create large plan with 500 segments
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
    
    println!("Plan statistics:");
    println!("  - Days: {}", plan.schedule.len());
    println!("  - Total segments: {}", plan.schedule.iter().map(|d| d.segments.len()).sum::<usize>());
    println!("  - Dictionary entries: {}", plan.dictionary.len());
    
    // Test validation performance
    let validator = PlanValidator::new().expect("Failed to create validator");
    
    let start = Instant::now();
    let result = validator.validate(&plan);
    let validation_time = start.elapsed();
    
    println!("Validation completed in: {:?}", validation_time);
    println!("Validation results:");
    println!("  - Errors: {}", result.errors.len());
    println!("  - Warnings: {}", result.warnings.len());
    
    // Performance target: <150ms for 500-item validation
    assert!(validation_time.as_millis() < 150, 
            "Validation took {}ms, target is <150ms", validation_time.as_millis());
    
    println!("✅ Performance test PASSED: {}ms < 150ms target", validation_time.as_millis());
}
