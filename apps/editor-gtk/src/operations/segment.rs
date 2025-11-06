use crate::state::AppState;
use crate::canvas::update_canvas_content;
use std::sync::{Arc, Mutex};
use weightlifting_core::{Day, Segment, StraightSegment, BaseSegment, RepsOrRange, RepsRange, CommentSegment, GroupOptionalSegment};

pub fn group_selected_segments(state: Arc<Mutex<AppState>>) {
    let mut app_state = state.lock().unwrap();
    
    if !app_state.has_selection() {
        println!("No segments selected. Select segments first, then press G to group them.");
        return;
    }
    
    if app_state.selected_segments.len() < 2 {
        println!("Need at least 2 segments to create a group. Current selection: {}", app_state.selected_segments.len());
        return;
    }
    
    // Extract the selected segments first to avoid borrowing issues
    let selected: Vec<_> = app_state.selected_segments.iter().cloned().collect();
    
    // Save current state to undo history before making changes
    app_state.save_to_undo_history();
    
    if let Some(plan) = &mut app_state.current_plan {
        
        // Extract selected segments (this is a simplified implementation)
        let mut segments_to_group = Vec::new();
        
        for (day_idx, seg_idx) in &selected {
            if let Some(day) = plan.schedule.get(*day_idx) {
                if let Some(segment) = day.segments.get(*seg_idx) {
                    segments_to_group.push(segment.clone());
                }
            }
        }
        
        if !segments_to_group.is_empty() {
            let group = GroupOptionalSegment::new(segments_to_group);
            let group_segment = Segment::GroupOptional(group);
            
            // Add to first day (simplified)
            if !plan.schedule.is_empty() {
                plan.schedule[0].segments.push(group_segment);
                app_state.mark_modified();
                app_state.clear_selection();
                println!("Created optional group with {} segments", selected.len());
            }
        }
    } else {
        println!("No plan open. Create a new plan first.");
    }
}

pub fn ungroup_selected_segments(state: Arc<Mutex<AppState>>) {
    let mut app_state = state.lock().unwrap();
    
    if !app_state.has_selection() {
        println!("No segments selected. Select group segments first, then press U to ungroup them.");
        return;
    }
    
    // This would implement ungrouping logic
    println!("Ungrouping {} selected segments (not fully implemented)", app_state.selected_segments.len());
    app_state.clear_selection();
    app_state.mark_modified();
}

pub fn clear_segment_selection(state: Arc<Mutex<AppState>>) {
    let mut app_state = state.lock().unwrap();
    let count = app_state.selected_segments.len();
    app_state.clear_selection();
    if count > 0 {
        println!("Cleared selection of {} segments", count);
    }
}

pub fn undo_last_action(state: Arc<Mutex<AppState>>) {
    let mut app_state = state.lock().unwrap();
    
    if app_state.undo() {
        println!("Undo successful - restored previous state");
        // Drop the lock before calling update_canvas_content to avoid deadlock
        drop(app_state);
        update_canvas_content(state.clone());
    } else {
        println!("Nothing to undo");
    }
}

pub fn move_segment_up(state: Arc<Mutex<AppState>>, day_index: usize, segment_index: usize) {
    let mut app_state = state.lock().unwrap();
    
    // Save to undo history before making changes
    app_state.save_to_undo_history();
    
    if let Some(plan) = &mut app_state.current_plan {
        if day_index < plan.schedule.len() && segment_index > 0 {
            let day = &mut plan.schedule[day_index];
            if segment_index < day.segments.len() {
                day.segments.swap(segment_index, segment_index - 1);
                app_state.mark_modified();
                println!("Moved segment up in day {}", day_index + 1);
                
                // Update UI
                drop(app_state);
                update_canvas_content(state.clone());
            }
        }
    }
}

pub fn move_segment_down(state: Arc<Mutex<AppState>>, day_index: usize, segment_index: usize) {
    let mut app_state = state.lock().unwrap();
    
    // Save to undo history before making changes
    app_state.save_to_undo_history();
    
    if let Some(plan) = &mut app_state.current_plan {
        if day_index < plan.schedule.len() {
            let day = &mut plan.schedule[day_index];
            if segment_index < day.segments.len() - 1 {
                day.segments.swap(segment_index, segment_index + 1);
                app_state.mark_modified();
                println!("Moved segment down in day {}", day_index + 1);
                
                // Update UI
                drop(app_state);
                update_canvas_content(state.clone());
            }
        }
    }
}

pub fn add_custom_exercise_to_plan(state: Arc<Mutex<AppState>>, ex_code: String, ex_label: String, alt_group: Option<String>, sets: u32, min_reps: u32, max_reps: u32, rpe: f64) {
    // Check if a target day was set, otherwise use first day (0)
    let target_day = {
        let mut app_state = state.lock().unwrap();
        app_state.get_and_clear_target_day().unwrap_or(0)
    };
    add_custom_exercise_to_day(state, target_day, ex_code, ex_label, alt_group, sets, min_reps, max_reps, rpe);
}

pub fn add_custom_exercise_to_day(state: Arc<Mutex<AppState>>, day_index: usize, ex_code: String, ex_label: String, alt_group: Option<String>, sets: u32, min_reps: u32, max_reps: u32, rpe: f64) {
    let mut app_state = state.lock().unwrap();
    
    // Save to undo history before making changes
    app_state.save_to_undo_history();
    
    if let Some(plan) = &mut app_state.current_plan {
        // Add exercise to dictionary if not exists
        if !plan.dictionary.contains_key(&ex_code) {
            plan.dictionary.insert(ex_code.clone(), ex_label.clone());
        }
        
        // Ensure we have at least one day
        if plan.schedule.is_empty() {
            let day = Day {
                day: 1,
                label: "Training Day".to_string(),
                time_cap_min: None,
                goal: None,
                equipment_policy: None,
                segments: vec![],
            };
            plan.schedule.push(day);
        }
        
        let target_day_index = if day_index >= plan.schedule.len() { 0 } else { day_index };
        
        // Create exercise segment
        let straight_segment = StraightSegment {
            base: BaseSegment {
                ex: ex_code,
                alt_group,
                label: Some(ex_label.clone()),
                optional: None,
                technique: None,
                equipment_policy: None,
            },
            sets: Some(sets),
            sets_range: None,
            reps: Some(RepsOrRange::Range(RepsRange { min: min_reps, max: max_reps, target: None })),
            time_sec: None,
            rest_sec: None,
            rir: None,
            rpe: Some(rpe),
            tempo: None,
            vbt: None,
            load_mode: None,
            intensifier: None,
            auto_stop: None,
            interval: None,
        };
        
        let segment = Segment::Straight(straight_segment);
        plan.schedule[target_day_index].segments.push(segment);
        
        app_state.mark_modified();
        println!("Added exercise to day {}: {} ({}x{}-{} @ RPE {})", target_day_index + 1, ex_label, sets, min_reps, max_reps, rpe);
        
        // Update UI
        drop(app_state);
        update_canvas_content(state.clone());
    } else {
        println!("No plan open. Create a new plan first.");
    }
}

pub fn add_custom_comment_to_plan(state: Arc<Mutex<AppState>>, text: String, icon: Option<String>) {
    // Check if a target day was set, otherwise use first day (0)
    let target_day = {
        let mut app_state = state.lock().unwrap();
        app_state.get_and_clear_target_day().unwrap_or(0)
    };
    add_custom_comment_to_day(state, target_day, text, icon);
}

pub fn add_custom_comment_to_day(state: Arc<Mutex<AppState>>, day_index: usize, text: String, icon: Option<String>) {
    let mut app_state = state.lock().unwrap();
    
    // Save to undo history before making changes
    app_state.save_to_undo_history();
    
    if let Some(plan) = &mut app_state.current_plan {
        // Ensure we have at least one day
        if plan.schedule.is_empty() {
            let day = Day {
                day: 1,
                label: "Training Day".to_string(),
                time_cap_min: None,
                goal: None,
                equipment_policy: None,
                segments: vec![],
            };
            plan.schedule.push(day);
        }
        
        let target_day_index = if day_index >= plan.schedule.len() { 0 } else { day_index };
        
        let comment_segment = CommentSegment { text: text.clone(), icon };
        let segment = Segment::Comment(comment_segment);
        plan.schedule[target_day_index].segments.push(segment);
        
        app_state.mark_modified();
        println!("Added comment segment to day {}: {}", target_day_index + 1, text);
        
        // Update UI
        drop(app_state);
        update_canvas_content(state.clone());
    } else {
        println!("No plan open. Create a new plan first.");
    }
}

pub fn update_day_label(state: Arc<Mutex<AppState>>, day_index: usize, new_label: String) {
    let mut app_state = state.lock().unwrap();
    
    // Save to undo history before making changes
    app_state.save_to_undo_history();
    
    if let Some(plan) = &mut app_state.current_plan {
        if day_index < plan.schedule.len() {
            let old_label = plan.schedule[day_index].label.clone();
            plan.schedule[day_index].label = new_label.clone();
            
            app_state.mark_modified();
            println!("Updated day {} label from '{}' to '{}'", day_index + 1, old_label, new_label);
            
            // Update UI
            drop(app_state);
            update_canvas_content(state.clone());
        } else {
            println!("Invalid day index: {}", day_index);
        }
    } else {
        println!("No plan open. Cannot update day label.");
    }
}

pub fn set_target_day_for_next_segment(state: Arc<Mutex<AppState>>, day_index: usize) {
    let mut app_state = state.lock().unwrap();
    app_state.set_target_day_for_next_segment(day_index);
}

pub fn remove_exercise_from_superset(state: Arc<Mutex<AppState>>, day_index: usize, segment_index: usize, exercise_index: usize) {
    let mut app_state = state.lock().unwrap();
    app_state.save_to_undo_history();
    
    if let Some(plan) = &mut app_state.current_plan {
        if day_index < plan.schedule.len() && segment_index < plan.schedule[day_index].segments.len() {
            if let weightlifting_core::Segment::Superset(ref mut superset) = &mut plan.schedule[day_index].segments[segment_index] {
                if exercise_index < superset.items.len() {
                    let removed_exercise = superset.items.remove(exercise_index);
                    app_state.mark_modified();
                    println!("Removed exercise '{}' from superset", removed_exercise.ex);
                    
                    drop(app_state);
                    update_canvas_content(state.clone());
                } else {
                    println!("Invalid exercise index: {}", exercise_index);
                }
            } else {
                println!("Segment is not a superset");
            }
        } else {
            println!("Invalid day or segment index");
        }
    } else {
        println!("No plan open");
    }
}

pub fn remove_exercise_from_circuit(state: Arc<Mutex<AppState>>, day_index: usize, segment_index: usize, exercise_index: usize) {
    let mut app_state = state.lock().unwrap();
    app_state.save_to_undo_history();
    
    if let Some(plan) = &mut app_state.current_plan {
        if day_index < plan.schedule.len() && segment_index < plan.schedule[day_index].segments.len() {
            if let weightlifting_core::Segment::Circuit(ref mut circuit) = &mut plan.schedule[day_index].segments[segment_index] {
                if exercise_index < circuit.items.len() {
                    let removed_exercise = circuit.items.remove(exercise_index);
                    app_state.mark_modified();
                    println!("Removed exercise '{}' from circuit", removed_exercise.ex);
                    
                    drop(app_state);
                    update_canvas_content(state.clone());
                } else {
                    println!("Invalid exercise index: {}", exercise_index);
                }
            } else {
                println!("Segment is not a circuit");
            }
        } else {
            println!("Invalid day or segment index");
        }
    } else {
        println!("No plan open");
    }
}

pub fn update_comment_segment(state: Arc<Mutex<AppState>>, day_index: usize, segment_index: usize, text: String, icon: Option<String>) {
    let mut app_state = state.lock().unwrap();
    
    if app_state.current_plan.is_some() {
        // Save to undo history before making changes
        app_state.save_to_undo_history();
        
        if let Some(ref mut plan) = app_state.current_plan {
            if day_index < plan.schedule.len() && segment_index < plan.schedule[day_index].segments.len() {
                if let weightlifting_core::Segment::Comment(ref mut comment) = &mut plan.schedule[day_index].segments[segment_index] {
                    comment.text = text;
                    comment.icon = icon;
                    
                    app_state.mark_modified();
                    println!("Updated comment segment");
                    
                    drop(app_state);
                    update_canvas_content(state.clone());
                } else {
                    println!("Segment is not a comment");
                }
            } else {
                println!("Invalid day or segment index");
            }
        }
    } else {
        println!("No plan open");
    }
}

pub fn update_straight_segment(
    state: Arc<Mutex<AppState>>, 
    day_index: usize, 
    segment_index: usize, 
    ex: String,
    label: Option<String>,
    alt_group: Option<String>,
    sets: Option<u32>,
    min_reps: Option<u32>,
    max_reps: Option<u32>,
    rpe: Option<f64>,
    rest_sec: Option<u32>
) {
    let mut app_state = state.lock().unwrap();
    
    if app_state.current_plan.is_some() {
        app_state.save_to_undo_history();
        
        if let Some(ref mut plan) = app_state.current_plan {
            if day_index < plan.schedule.len() && segment_index < plan.schedule[day_index].segments.len() {
                if let weightlifting_core::Segment::Straight(ref mut straight) = &mut plan.schedule[day_index].segments[segment_index] {
                    straight.base.ex = ex;
                    straight.base.label = label;
                    straight.base.alt_group = alt_group;
                    straight.sets = sets;
                    if let (Some(min), Some(max)) = (min_reps, max_reps) {
                        straight.reps = Some(weightlifting_core::RepsOrRange::Range(weightlifting_core::RepsRange { min, max, target: None }));
                    }
                    straight.rpe = rpe;
                    if let Some(rest) = rest_sec {
                        straight.rest_sec = Some(weightlifting_core::RestOrRange::Fixed(rest));
                    }
                    
                    app_state.mark_modified();
                    println!("Updated straight segment");
                    
                    drop(app_state);
                    update_canvas_content(state.clone());
                } else {
                    println!("Segment is not a straight segment");
                }
            } else {
                println!("Invalid day or segment index");
            }
        }
    } else {
        println!("No plan open");
    }
}

pub fn update_rpe_segment(
    state: Arc<Mutex<AppState>>, 
    day_index: usize, 
    segment_index: usize, 
    ex: String,
    label: Option<String>,
    alt_group: Option<String>,
    sets: u32,
    min_reps: Option<u32>,
    max_reps: Option<u32>,
    rpe: f64,
    rest_sec: Option<u32>
) {
    let mut app_state = state.lock().unwrap();
    
    if app_state.current_plan.is_some() {
        app_state.save_to_undo_history();
        
        if let Some(ref mut plan) = app_state.current_plan {
            if day_index < plan.schedule.len() && segment_index < plan.schedule[day_index].segments.len() {
                if let weightlifting_core::Segment::Rpe(ref mut rpe_seg) = &mut plan.schedule[day_index].segments[segment_index] {
                    rpe_seg.base.ex = ex;
                    rpe_seg.base.label = label;
                    rpe_seg.base.alt_group = alt_group;
                    rpe_seg.sets = sets;
                    if let (Some(min), Some(max)) = (min_reps, max_reps) {
                        rpe_seg.reps = Some(weightlifting_core::RepsOrRange::Range(weightlifting_core::RepsRange { min, max, target: None }));
                    }
                    rpe_seg.rpe = rpe;
                    if let Some(rest) = rest_sec {
                        rpe_seg.rest_sec = Some(weightlifting_core::RestOrRange::Fixed(rest));
                    }
                    
                    app_state.mark_modified();
                    println!("Updated RPE segment");
                    
                    drop(app_state);
                    update_canvas_content(state.clone());
                } else {
                    println!("Segment is not an RPE segment");
                }
            } else {
                println!("Invalid day or segment index");
            }
        }
    } else {
        println!("No plan open");
    }
}

pub fn update_amrap_segment(
    state: Arc<Mutex<AppState>>, 
    day_index: usize, 
    segment_index: usize, 
    ex: String,
    label: Option<String>,
    alt_group: Option<String>,
    base_reps: u32,
    cap_reps: u32
) {
    let mut app_state = state.lock().unwrap();
    
    if app_state.current_plan.is_some() {
        app_state.save_to_undo_history();
        
        if let Some(ref mut plan) = app_state.current_plan {
            if day_index < plan.schedule.len() && segment_index < plan.schedule[day_index].segments.len() {
                if let weightlifting_core::Segment::Amrap(ref mut amrap) = &mut plan.schedule[day_index].segments[segment_index] {
                    amrap.base.ex = ex;
                    amrap.base.label = label;
                    amrap.base.alt_group = alt_group;
                    amrap.base_reps = base_reps;
                    amrap.cap_reps = cap_reps;
                    
                    app_state.mark_modified();
                    println!("Updated AMRAP segment");
                    
                    drop(app_state);
                    update_canvas_content(state.clone());
                } else {
                    println!("Segment is not an AMRAP segment");
                }
            } else {
                println!("Invalid day or segment index");
            }
        }
    } else {
        println!("No plan open");
    }
}

pub fn update_time_segment(
    state: Arc<Mutex<AppState>>, 
    day_index: usize, 
    segment_index: usize, 
    ex: String,
    label: Option<String>,
    alt_group: Option<String>,
    rpe: Option<f64>,
    _rest_sec: Option<u32>
) {
    let mut app_state = state.lock().unwrap();
    
    if app_state.current_plan.is_some() {
        app_state.save_to_undo_history();
        
        if let Some(ref mut plan) = app_state.current_plan {
            if day_index < plan.schedule.len() && segment_index < plan.schedule[day_index].segments.len() {
                if let weightlifting_core::Segment::Time(ref mut time_seg) = &mut plan.schedule[day_index].segments[segment_index] {
                    time_seg.base.ex = ex;
                    time_seg.base.label = label;
                    time_seg.base.alt_group = alt_group;
                    time_seg.rpe = rpe;
                    
                    app_state.mark_modified();
                    println!("Updated time segment");
                    
                    drop(app_state);
                    update_canvas_content(state.clone());
                } else {
                    println!("Segment is not a time segment");
                }
            } else {
                println!("Invalid day or segment index");
            }
        }
    } else {
        println!("No plan open");
    }
}

pub fn update_percentage_segment(
    state: Arc<Mutex<AppState>>, 
    day_index: usize, 
    segment_index: usize, 
    ex: String,
    label: Option<String>,
    alt_group: Option<String>,
    prescriptions: Vec<weightlifting_core::PercentagePrescription>
) {
    let mut app_state = state.lock().unwrap();
    
    if app_state.current_plan.is_some() {
        app_state.save_to_undo_history();
        
        if let Some(ref mut plan) = app_state.current_plan {
            if day_index < plan.schedule.len() && segment_index < plan.schedule[day_index].segments.len() {
                if let weightlifting_core::Segment::Percentage(ref mut pct_seg) = &mut plan.schedule[day_index].segments[segment_index] {
                    pct_seg.base.ex = ex;
                    pct_seg.base.label = label;
                    pct_seg.base.alt_group = alt_group;
                    pct_seg.prescriptions = prescriptions;
                    
                    app_state.mark_modified();
                    println!("Updated percentage segment");
                    
                    drop(app_state);
                    update_canvas_content(state.clone());
                } else {
                    println!("Segment is not a percentage segment");
                }
            } else {
                println!("Invalid day or segment index");
            }
        }
    } else {
        println!("No plan open");
    }
}