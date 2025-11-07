use crate::canvas::update_canvas_content;
use crate::operations::segment::update_day_label;
use crate::state::AppState;
use glib::clone;
use gtk4::prelude::*;
use gtk4::{
    Box as GtkBox, Dialog, DialogFlags, Entry, Label, Orientation, ResponseType, SpinButton,
};
use std::sync::{Arc, Mutex};
use weightlifting_core::Day;

pub fn show_edit_day_dialog(state: Arc<Mutex<AppState>>, day_index: usize) {
    let (day_number, current_label) = {
        let app_state = state.lock().unwrap();
        if let Some(plan) = &app_state.current_plan {
            if day_index < plan.schedule.len() {
                (
                    plan.schedule[day_index].day,
                    plan.schedule[day_index].label.clone(),
                )
            } else {
                (1, "Training Day".to_string())
            }
        } else {
            (1, "Training Day".to_string())
        }
    };

    let dialog = Dialog::with_buttons(
        Some(&format!("Edit Day {} Label", day_number)),
        crate::ui::util::parent_for_dialog().as_ref(),
        DialogFlags::MODAL,
        &[
            ("Cancel", ResponseType::Cancel),
            ("Save", ResponseType::Accept),
        ],
    );
    crate::ui::util::standardize_dialog(&dialog);
    let content = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .margin_start(20)
        .margin_end(20)
        .margin_top(20)
        .margin_bottom(20)
        .spacing(12)
        .build();

    // Day number (read-only display)
    let day_number_label = Label::new(Some(&format!("Day Number: {}", day_number)));
    content.append(&day_number_label);

    // Day label entry (editable)
    let label_label = Label::new(Some("Day Label:"));
    let label_entry = Entry::builder().text(&current_label).build();

    content.append(&label_label);
    content.append(&label_entry);

    dialog.content_area().append(&content);
    // Ctrl+Enter triggers Save
    crate::ui::util::bind_ctrl_enter_to_accept(&dialog);

    dialog.connect_response(clone!(@strong state => move |dialog, response| {
        if response == ResponseType::Accept {
            let new_label = label_entry.text().to_string();
            if !new_label.trim().is_empty() {
                update_day_label(state.clone(), day_index, new_label);
            }
        }
        dialog.close();
    }));

    dialog.present();
}

pub fn show_add_day_dialog(state: Arc<Mutex<AppState>>) {
    let dialog = Dialog::with_buttons(
        Some("Add New Day"),
        crate::ui::util::parent_for_dialog().as_ref(),
        DialogFlags::MODAL,
        &[
            ("Cancel", ResponseType::Cancel),
            ("Add", ResponseType::Accept),
        ],
    );

    crate::ui::util::standardize_dialog(&dialog);
    let content = create_day_dialog_content(state.clone());
    dialog.content_area().append(&content.0);
    // Ctrl+Enter triggers Add
    crate::ui::util::bind_ctrl_enter_to_accept(&dialog);

    dialog.connect_response(clone!(@strong state => move |dialog, response| {
        if response == ResponseType::Accept {
            handle_add_day_response(state.clone(), &content.1, &content.2, &content.3, &content.4);
        }
        dialog.close();
    }));

    dialog.present();
}

fn create_day_dialog_content(
    state: Arc<Mutex<AppState>>,
) -> (GtkBox, SpinButton, Entry, SpinButton, Entry) {
    let content = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .margin_start(20)
        .margin_end(20)
        .margin_top(20)
        .margin_bottom(20)
        .spacing(12)
        .build();

    // Day number entry
    let day_label = Label::new(Some("Day Number:"));
    let day_entry = SpinButton::with_range(1.0, 999.0, 1.0);

    // Set default to next day number
    {
        let app_state = state.lock().unwrap();
        if let Some(plan) = &app_state.current_plan {
            let next_day = plan.schedule.iter().map(|d| d.day).max().unwrap_or(0) + 1;
            day_entry.set_value(next_day as f64);
        }
    }

    content.append(&day_label);
    content.append(&day_entry);

    // Day label entry
    let label_label = Label::new(Some("Day Label:"));
    let label_entry = Entry::builder().text("Training Day").build();

    content.append(&label_label);
    content.append(&label_entry);

    // Optional time cap
    let time_cap_label = Label::new(Some("Time Cap (minutes, optional):"));
    let time_cap_entry = SpinButton::with_range(0.0, 300.0, 5.0);
    time_cap_entry.set_value(0.0);

    content.append(&time_cap_label);
    content.append(&time_cap_entry);

    // Optional goal entry
    let goal_label = Label::new(Some("Goal (optional):"));
    let goal_entry = Entry::builder().text("").build();

    content.append(&goal_label);
    content.append(&goal_entry);

    (content, day_entry, label_entry, time_cap_entry, goal_entry)
}

fn handle_add_day_response(
    state: Arc<Mutex<AppState>>,
    day_entry: &SpinButton,
    label_entry: &Entry,
    time_cap_entry: &SpinButton,
    goal_entry: &Entry,
) {
    let day_number = day_entry.value() as u32;
    let day_label = label_entry.text().to_string();
    let time_cap = if time_cap_entry.value() > 0.0 {
        Some(time_cap_entry.value() as u32)
    } else {
        None
    };
    let goal = if goal_entry.text().is_empty() {
        None
    } else {
        Some(goal_entry.text().to_string())
    };

    add_new_day_to_plan(state, day_number, day_label, time_cap, goal);
}

fn add_new_day_to_plan(
    state: Arc<Mutex<AppState>>,
    day_number: u32,
    day_label: String,
    time_cap: Option<u32>,
    goal: Option<String>,
) {
    let mut app_state = state.lock().unwrap();

    // Save to undo history before making changes
    app_state.save_to_undo_history();

    if let Some(plan) = &mut app_state.current_plan {
        let new_day = Day {
            day: day_number,
            label: day_label.clone(),
            time_cap_min: time_cap,
            goal,
            equipment_policy: None,
            segments: vec![],
        };

        plan.schedule.push(new_day);

        // Sort schedule by day number
        plan.schedule.sort_by_key(|d| d.day);

        app_state.mark_modified();
        app_state.clear_selection();

        println!("Added new day: Day {} - {}", day_number, day_label);

        // Update UI
        drop(app_state);
        update_canvas_content(state);
    } else {
        // Auto-create a new empty plan and add the day
        let mut plan = weightlifting_core::Plan::new("New Plan".to_string());
        let new_day = Day {
            day: day_number,
            label: day_label.clone(),
            time_cap_min: time_cap,
            goal,
            equipment_policy: None,
            segments: vec![],
        };
        plan.schedule.push(new_day);
        app_state.current_plan = Some(plan);
        app_state.plan_id = Some("new_plan".to_string());
        app_state.current_file_path = None;
        app_state.mark_modified();
        app_state.clear_selection();
        drop(app_state);
        println!(
            "Created new plan and added Day {} - {}",
            day_number, day_label
        );
        update_canvas_content(state);
    }
}
