use crate::canvas::formatting::format_segment;
use crate::state::AppState;
use crate::canvas::update_canvas_content;
use crate::dialogs::segment::edit_segment_if_applicable;
use glib::clone;
use gtk4::prelude::*;
use gtk4::{Orientation, Label};
use std::sync::{Arc, Mutex};


pub(crate) fn move_selected_segments_up(state: Arc<Mutex<AppState>>) {
    let mut app_state = state.lock().unwrap();
    
    if !app_state.has_selection() {
        println!("No segments selected. Select segments first, then use Ctrl+Up to move them up.");
        return;
    }
    
    // Save to undo history before making changes
    app_state.save_to_undo_history();
    
    // Get selected segments sorted by position
    let selected: Vec<_> = app_state.selected_segments.iter().cloned().collect();
    let mut sorted_selected = selected.clone();
    sorted_selected.sort_by_key(|(day_idx, seg_idx)| (*day_idx, *seg_idx));
    
    let mut moved_any = false;
    
    if let Some(plan) = &mut app_state.current_plan {
        for (day_idx, seg_idx) in sorted_selected {
            if let Some(day) = plan.schedule.get_mut(day_idx) {
                if seg_idx > 0 && seg_idx < day.segments.len() {
                    // Swap with previous segment
                    day.segments.swap(seg_idx - 1, seg_idx);
                    moved_any = true;
                }
            }
        }
    }
    
    // Update selection positions after movement
    if moved_any {
        let mut new_selection = std::collections::HashSet::new();
        for (day_idx, seg_idx) in &selected {
            if *seg_idx > 0 {
                new_selection.insert((*day_idx, seg_idx - 1));
            } else {
                new_selection.insert((*day_idx, *seg_idx));
            }
        }
        app_state.selected_segments = new_selection;
        app_state.mark_modified();
        println!("Moved {} selected segments up", app_state.selected_segments.len());
        
        // Update UI
        drop(app_state);
        update_canvas_content(state.clone());
    } else {
        println!("Cannot move segments up - already at top or invalid positions");
    }
}

pub(crate) fn move_selected_segments_down(state: Arc<Mutex<AppState>>) {
    let mut app_state = state.lock().unwrap();
    
    if !app_state.has_selection() {
        println!("No segments selected. Select segments first, then use Ctrl+Down to move them down.");
        return;
    }
    
    // Save to undo history before making changes
    app_state.save_to_undo_history();
    
    // Get selected segments sorted by position (reverse order for down movement)
    let selected: Vec<_> = app_state.selected_segments.iter().cloned().collect();
    let mut sorted_selected = selected.clone();
    sorted_selected.sort_by_key(|(day_idx, seg_idx)| (*day_idx, std::cmp::Reverse(*seg_idx)));
    
    let mut moved_any = false;
    
    if let Some(plan) = &mut app_state.current_plan {
        for (day_idx, seg_idx) in sorted_selected {
            if let Some(day) = plan.schedule.get_mut(day_idx) {
                if seg_idx < day.segments.len().saturating_sub(1) {
                    // Swap with next segment
                    day.segments.swap(seg_idx, seg_idx + 1);
                    moved_any = true;
                }
            }
        }
    }
    
    // Update selection positions after movement
    if moved_any {
        let mut new_selection = std::collections::HashSet::new();
        for (day_idx, seg_idx) in &selected {
            if *seg_idx < 100 { // Safety check for overflow
                new_selection.insert((*day_idx, seg_idx + 1));
            } else {
                new_selection.insert((*day_idx, *seg_idx));
            }
        }
        app_state.selected_segments = new_selection;
        app_state.mark_modified();
        println!("Moved {} selected segments down", app_state.selected_segments.len());
        
        // Update UI
        drop(app_state);
        update_canvas_content(state.clone());
    } else {
        println!("Cannot move segments down - already at bottom or invalid positions");
    }
}

#[allow(dead_code)]
pub(crate) fn delete_selected_segments_with_confirmation(state: Arc<Mutex<AppState>>) {
    use gtk4::{Dialog, DialogFlags, ResponseType, Box as GtkBox};
    
    let app_state = state.lock().unwrap();
    
    if !app_state.has_selection() {
        println!("No segments selected. Select segments first, then press Delete to remove them.");
        return;
    }
    
    let selection_count = app_state.selected_segments.len();
    
    // Collect information about selected segments for impact summary
    let mut impact_summary = String::new();
    if let Some(plan) = &app_state.current_plan {
        for (day_idx, seg_idx) in &app_state.selected_segments {
            if let Some(day) = plan.schedule.get(*day_idx) {
                if let Some(segment) = day.segments.get(*seg_idx) {
                    let description = format_segment(segment, Some(&plan.dictionary));
                    impact_summary.push_str(&format!("• Day {}: {}\n", day.day, description));
                }
            }
        }
    }
    
    drop(app_state); // Release lock before showing dialog
    
    let dialog = Dialog::with_buttons(
        Some("Confirm Deletion"),
        crate::ui::util::parent_for_dialog().as_ref(),
        DialogFlags::MODAL,
        &[("Cancel", ResponseType::Cancel), ("Delete", ResponseType::Accept)]
    );
    
    let content = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .margin_start(20)
        .margin_end(20)
        .margin_top(20)
        .margin_bottom(20)
        .spacing(12)
        .build();
    
    let warning_label = Label::builder()
        .label(format!("⚠️ Delete {} segment{}?", selection_count, if selection_count == 1 { "" } else { "s" }))
        .css_classes(vec!["title-1".to_string()])
        .halign(gtk4::Align::Start)
        .build();
    
    let impact_label = Label::builder()
        .label(format!("This will permanently remove:\n\n{}", impact_summary))
        .halign(gtk4::Align::Start)
        .wrap(true)
        .selectable(true)
        .build();
    
    let undo_note = Label::builder()
        .label("💡 You can undo this action with Ctrl+Z")
        .css_classes(vec!["dim-label".to_string()])
        .halign(gtk4::Align::Start)
        .build();
    
    content.append(&warning_label);
    content.append(&impact_label);
    content.append(&undo_note);
    
    dialog.content_area().append(&content);
    
    dialog.connect_response(clone!(@strong state => move |dialog, response| {
        if response == ResponseType::Accept {
            perform_segment_deletion(state.clone());
        } else {
            println!("Deletion cancelled - no changes made");
        }
        dialog.close();
    }));
    
    dialog.present();
}

pub(crate) fn perform_segment_deletion(state: Arc<Mutex<AppState>>) {
    println!("DEBUG: Delete function called");
    let mut app_state = state.lock().unwrap();
    
    // Determine what to delete: selected segments or focused segment
    let segments_to_delete = if app_state.has_selection() {
        println!("DEBUG: Deleting selected segments");
        // Delete selected segments
        app_state.selected_segments.iter().cloned().collect()
    } else if let Some(focused) = app_state.focused_segment {
        println!("DEBUG: Deleting focused segment: {:?}", focused);
        // Delete focused segment if nothing is selected
        vec![focused]
    } else {
        println!("DEBUG: No focus, trying to initialize");
        // Initialize focus and try again
        app_state.ensure_focus_initialized();
        if let Some(focused) = app_state.focused_segment {
            println!("DEBUG: Initialized focus to: {:?}", focused);
            vec![focused]
        } else {
            println!("No segments to delete. Use Up/Down arrows to focus a segment first.");
            return;
        }
    };
    
    if segments_to_delete.is_empty() {
        println!("No segments to delete.");
        return;
    }
    
    // Save to undo history before making changes
    app_state.save_to_undo_history();
    
    // Sort segments by position (reverse order for safe deletion)
    let mut sorted_segments = segments_to_delete.clone();
    sorted_segments.sort_by_key(|(day_idx, seg_idx)| (*day_idx, std::cmp::Reverse(*seg_idx)));
    
    let mut deleted_count = 0;
    let was_selection = app_state.has_selection();
    let original_focused = app_state.focused_segment;
    
    if let Some(plan) = &mut app_state.current_plan {
        for (day_idx, seg_idx) in sorted_segments {
            if let Some(day) = plan.schedule.get_mut(day_idx) {
                if seg_idx < day.segments.len() {
                    day.segments.remove(seg_idx);
                    deleted_count += 1;
                }
            }
        }
    }
    
    // Update focus after deletions
    if let Some((focus_day, focus_seg)) = original_focused {
        let mut new_focus = Some((focus_day, focus_seg));
        
        // Check if we deleted the focused segment or need to adjust focus
        for (day_idx, seg_idx) in &segments_to_delete {
            if (*day_idx, *seg_idx) == (focus_day, focus_seg) {
                // We deleted the focused segment, clear focus
                new_focus = None;
                break;
            } else if *day_idx == focus_day && *seg_idx < focus_seg {
                // A segment before the focused one was deleted, adjust focus
                if let Some((_, ref mut adjusted_seg)) = new_focus {
                    *adjusted_seg = adjusted_seg.saturating_sub(1);
                }
            }
        }
        
        app_state.focused_segment = new_focus;
    }
    
    if deleted_count > 0 {
        app_state.mark_modified();
        if was_selection {
            app_state.clear_selection();
        }
        
        let delete_type = if was_selection { "selected" } else { "focused" };
        println!("Deleted {} {} segment{} - use Ctrl+Z to undo", 
                deleted_count, delete_type, if deleted_count == 1 { "" } else { "s" });
        
        // Update UI
        drop(app_state);
        update_canvas_content(state.clone());
    } else {
        println!("No valid segments to delete.");
    }
}

#[allow(dead_code)]
pub(crate) fn toggle_segment_selection(state: Arc<Mutex<AppState>>, day_idx: usize, seg_idx: usize) {
    let mut app_state = state.lock().unwrap();
    app_state.toggle_segment_selection(day_idx, seg_idx);
    
    let selection_count = app_state.selected_segments.len();
    if selection_count > 0 {
        println!("Selected {} segments (use Shift+Ctrl+Up/Down to reorder, Delete to remove)", selection_count);
        // Set focus to the toggled segment
        app_state.set_focused_segment(day_idx, seg_idx);
    } else {
        println!("Selection cleared");
    }
    
    // Update UI to reflect selection changes
    drop(app_state);
    update_canvas_content(state.clone());
}

/// Handle exclusive selection (single click)
pub(crate) fn select_segment_exclusively(state: Arc<Mutex<AppState>>, day_idx: usize, seg_idx: usize) {
    let mut app_state = state.lock().unwrap();
    app_state.select_segment_exclusively(day_idx, seg_idx);
    
    // Set focus to the selected segment
    app_state.set_focused_segment(day_idx, seg_idx);
    
    println!("Selected segment (use Shift+Ctrl+Up/Down to reorder, Delete to remove)");
    
    // Update UI to reflect selection changes
    drop(app_state);
    update_canvas_content(state.clone());
}

/// Handle toggle selection with Ctrl (Ctrl+Click)
pub(crate) fn toggle_segment_selection_with_ctrl(state: Arc<Mutex<AppState>>, day_idx: usize, seg_idx: usize) {
    let mut app_state = state.lock().unwrap();
    app_state.toggle_segment_selection_with_ctrl(day_idx, seg_idx);
    
    let selection_count = app_state.selected_segments.len();
    if selection_count > 0 {
        println!("Selected {} segments (use Shift+Ctrl+Up/Down to reorder, Delete to remove)", selection_count);
        // Set focus to the toggled segment
        app_state.set_focused_segment(day_idx, seg_idx);
    } else {
        println!("Selection cleared");
    }
    
    // Update UI to reflect selection changes
    drop(app_state);
    update_canvas_content(state.clone());
}

/// Handle range selection with Shift (Shift+Click)
pub(crate) fn select_segment_range(state: Arc<Mutex<AppState>>, day_idx: usize, seg_idx: usize) {
    let mut app_state = state.lock().unwrap();
    app_state.select_segment_range(day_idx, seg_idx);
    
    let selection_count = app_state.selected_segments.len();
    println!("Selected {} segments (use Shift+Ctrl+Up/Down to reorder, Delete to remove)", selection_count);
    
    // Set focus to the range end segment
    app_state.set_focused_segment(day_idx, seg_idx);
    
    // Update UI to reflect selection changes
    drop(app_state);
    update_canvas_content(state.clone());
}

pub(crate) fn navigate_to_previous_segment(state: Arc<Mutex<AppState>>) {
    let mut app_state = state.lock().unwrap();
    
    // Initialize focus if not set
    app_state.ensure_focus_initialized_with_days();
    
    if let Some(plan) = &app_state.current_plan {
        // Determine current position in the unified flow
        let current_position = if let Some((day_idx, seg_idx)) = app_state.focused_segment {
            // Currently focused on a segment
            Some((day_idx, Some(seg_idx)))
        } else { app_state.focused_day.map(|day_idx| (day_idx, None)) };
        
        if let Some((current_day_idx, current_seg_option)) = current_position {
            // Find the previous item in the unified flow
            if let Some(current_seg_idx) = current_seg_option {
                // Currently on a segment - go to previous segment or day header
                if current_seg_idx > 0 {
                    // Go to previous segment in same day
                    app_state.set_focused_segment(current_day_idx, current_seg_idx - 1);
                } else {
                    // Go to day header of current day
                    app_state.set_focused_day(current_day_idx);
                }
            } else {
                // Currently on a day header - go to last segment of previous day or previous day header
                if current_day_idx > 0 {
                    let prev_day_idx = current_day_idx - 1;
                    if let Some(prev_day) = plan.schedule.get(prev_day_idx) {
                        if !prev_day.segments.is_empty() {
                            // Go to last segment of previous day
                            let last_seg_idx = prev_day.segments.len() - 1;
                            app_state.set_focused_segment(prev_day_idx, last_seg_idx);
                        } else {
                            // Go to previous day header (empty day)
                            app_state.set_focused_day(prev_day_idx);
                        }
                    }
                } else {
                    println!("Already at the beginning");
                }
            }
        } else {
            // No focus set - initialize to end
            navigate_to_end_of_plan(&mut app_state);
        }
    }
    
    // Update UI
    drop(app_state);
    update_canvas_content(state.clone());
}

// Helper function to navigate to the next day in the unified flow
fn navigate_to_next_day_or_end(app_state: &mut AppState, current_day_idx: usize, plan_schedule_len: usize) {
    if current_day_idx + 1 < plan_schedule_len {
        let next_day_idx = current_day_idx + 1;
        app_state.set_focused_day(next_day_idx);
    } else {
        println!("Already at the end");
    }
}

// Helper function to navigate to the beginning of the plan
fn navigate_to_beginning_of_plan(app_state: &mut AppState) {
    if let Some(plan) = &app_state.current_plan {
        if !plan.schedule.is_empty() {
            app_state.set_focused_day(0);
        }
    }
}

// Helper function to navigate to the end of the plan
fn navigate_to_end_of_plan(app_state: &mut AppState) {
    if let Some(plan) = &app_state.current_plan {
        if !plan.schedule.is_empty() {
            let last_day_idx = plan.schedule.len() - 1;
            if let Some(last_day) = plan.schedule.get(last_day_idx) {
                if !last_day.segments.is_empty() {
                    let last_seg_idx = last_day.segments.len() - 1;
                    app_state.set_focused_segment(last_day_idx, last_seg_idx);
                } else {
                    app_state.set_focused_day(last_day_idx);
                }
            }
        }
    }
}

pub(crate) fn navigate_to_next_segment(state: Arc<Mutex<AppState>>) {
    let mut app_state = state.lock().unwrap();
    
    // Initialize focus if not set
    app_state.ensure_focus_initialized_with_days();
    
    if let Some(plan) = &app_state.current_plan {
        let plan_schedule_len = plan.schedule.len();
        
        // Determine current position in the unified flow
        let current_position = if let Some((day_idx, seg_idx)) = app_state.focused_segment {
            // Currently focused on a segment
            Some((day_idx, Some(seg_idx)))
        } else { app_state.focused_day.map(|day_idx| (day_idx, None)) };
        
        if let Some((current_day_idx, current_seg_option)) = current_position {
            // Find the next item in the unified flow
            if let Some(current_seg_idx) = current_seg_option {
                // Currently on a segment
                if let Some(current_day) = plan.schedule.get(current_day_idx) {
                    if current_seg_idx + 1 < current_day.segments.len() {
                        // Go to next segment in same day
                        app_state.set_focused_segment(current_day_idx, current_seg_idx + 1);
                    } else {
                        // Go to next day header or first segment of next day
                        navigate_to_next_day_or_end(&mut app_state, current_day_idx, plan_schedule_len);
                    }
                }
            } else {
                // Currently on a day header - go to first segment or next day
                if let Some(current_day) = plan.schedule.get(current_day_idx) {
                    if !current_day.segments.is_empty() {
                        // Go to first segment of current day
                        app_state.set_focused_segment(current_day_idx, 0);
                    } else {
                        // Go to next day header or first segment of next day
                        navigate_to_next_day_or_end(&mut app_state, current_day_idx, plan_schedule_len);
                    }
                }
            }
        } else {
            // No focus set - initialize to beginning
            navigate_to_beginning_of_plan(&mut app_state);
        }
    }
    
    // Update UI
    drop(app_state);
    update_canvas_content(state.clone());
}

pub(crate) fn navigate_to_previous_day(state: Arc<Mutex<AppState>>) {
    let mut app_state = state.lock().unwrap();
    
    // Initialize focus if not set
    app_state.ensure_focus_initialized();
    
    if let Some((current_day_idx, _current_seg_idx)) = app_state.get_focused_segment() {
        if let Some(plan) = &app_state.current_plan {
            if current_day_idx > 0 {
                // Go to first segment of previous day
                if let Some(prev_day) = plan.schedule.get(current_day_idx - 1) {
                    if !prev_day.segments.is_empty() {
                        app_state.set_focused_segment(current_day_idx - 1, 0);
                        println!("Navigate to day {}", current_day_idx);
                        
                        // Update UI
                        drop(app_state);
                        update_canvas_content(state.clone());
                    }
                }
            } else {
                println!("Already at first day");
            }
        }
    }
}

pub(crate) fn navigate_to_next_day(state: Arc<Mutex<AppState>>) {
    let mut app_state = state.lock().unwrap();
    
    // Initialize focus if not set
    app_state.ensure_focus_initialized();
    
    if let Some((current_day_idx, _current_seg_idx)) = app_state.get_focused_segment() {
        if let Some(plan) = &app_state.current_plan {
            if current_day_idx + 1 < plan.schedule.len() {
                // Go to first segment of next day
                if let Some(next_day) = plan.schedule.get(current_day_idx + 1) {
                    if !next_day.segments.is_empty() {
                        app_state.set_focused_segment(current_day_idx + 1, 0);
                        println!("Navigate to day {}", current_day_idx + 2);
                        
                        // Update UI
                        drop(app_state);
                        update_canvas_content(state.clone());
                    }
                }
            } else {
                println!("Already at last day");
            }
        }
    }
}

pub(crate) fn edit_focused_segment(state: Arc<Mutex<AppState>>) {
    use crate::dialogs::day::show_edit_day_dialog;
    
    // Initialize focus if not set
    {
        let mut app_state = state.lock().unwrap();
        app_state.ensure_focus_initialized_with_days();
    }
    
    let app_state = state.lock().unwrap();
    if let Some(day_idx) = app_state.focused_day {
        // Edit day name when day is focused
        println!("Editing day {}", day_idx + 1);
        drop(app_state);
        show_edit_day_dialog(state.clone(), day_idx);
    } else if let Some((day_idx, seg_idx)) = app_state.focused_segment {
        // Edit segment when segment is focused
        if let Some(plan) = &app_state.current_plan {
            if let Some(day) = plan.schedule.get(day_idx) {
                if let Some(segment) = day.segments.get(seg_idx) {
                    println!("Editing segment {} in day {}", seg_idx, day_idx + 1);
                    let segment_clone = segment.clone();
                    drop(app_state);
                    edit_segment_if_applicable(state.clone(), &segment_clone, day_idx, seg_idx);
                } else {
                    println!("Focused segment no longer exists");
                }
            } else {
                println!("Focused day no longer exists");
            }
        } else {
            println!("No plan open");
        }
    } else {
        println!("No item focused. Use Up/Down arrows to focus an item first.");
    }
}

pub(crate) fn add_segment_to_focused_day(state: Arc<Mutex<AppState>>) {
    use crate::dialogs::segment::show_add_segment_to_day_dialog;
    
    let app_state = state.lock().unwrap();
    let target_day_idx = if let Some(day_idx) = app_state.focused_day {
        Some(day_idx)
    } else if let Some((day_idx, _)) = app_state.focused_segment {
        Some(day_idx)
    } else {
        None
    };
    
    if let Some(day_idx) = target_day_idx {
        println!("Adding segment to day {}", day_idx + 1);
        drop(app_state);
        show_add_segment_to_day_dialog(state.clone(), day_idx);
    } else {
        println!("No day focused. Use Up/Down arrows to focus a day first.");
    }
}
