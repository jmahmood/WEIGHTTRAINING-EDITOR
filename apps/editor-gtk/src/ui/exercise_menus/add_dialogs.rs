// Dialog functions for adding single exercises (RPE, percentage, AMRAP, time-based)

use crate::state::AppState;
use crate::operations::plan_ops::{add_rpe_set_to_plan, add_percentage_set_to_plan, add_amrap_to_plan, add_time_based_to_plan};
use crate::ui::widgets::{ExerciseSearchWidget, GroupSearchWidget};
use glib::clone;
use gtk4::prelude::*;
use gtk4::{Orientation, Label, Box as GtkBox, Entry, SpinButton, CheckButton, Dialog, DialogFlags, ResponseType, Expander};
use std::sync::{Arc, Mutex};

// Helper function to create exercise search section with group selection
fn create_exercise_search_section(state: Arc<Mutex<AppState>>) -> (Expander, Expander, Entry, Entry, Entry, ExerciseSearchWidget) {
    let search_widget = ExerciseSearchWidget::new();
    search_widget.set_state(state.clone());
    if let Err(e) = search_widget.set_database_path("/home/jawaad/weightlifting-desktop/exercises.db") {
        println!("Failed to connect to exercise database: {}", e);
    }
    
    let search_expander = Expander::builder()
        .label("🔍 Search Exercise Database")
        .child(&search_widget.container)
        .expanded(true)
        .build();
    
    // Group search widget
    // When adding new exercises, no current alt_group exists
    let group_search_widget = GroupSearchWidget::new(state, None);
    let group_search_expander = Expander::builder()
        .label("🔗 Select Alternative Group (right-click to edit)")
        .child(&group_search_widget.container)
        .expanded(false)
        .build();

    // Enable right-click to edit groups
    group_search_widget.enable_right_click_edit();
    
    // Manual input fields
    let ex_entry = Entry::new();
    let label_entry = Entry::new();
    let alt_group_entry = Entry::new();
    alt_group_entry.set_placeholder_text(Some("e.g., GROUP_CHEST_PRESS"));
    
    // Connect search widget selection to populate manual fields (both selection and activation)
    let ex_entry_clone = ex_entry.clone();
    let label_entry_clone = label_entry.clone();
    let search_widget_clone = search_widget.clone();
    search_widget.connect_row_activated(move |result| {
        ex_entry_clone.set_text(&result.code);
        label_entry_clone.set_text(&result.name);
        search_widget_clone.set_selected_exercise(result);
    });
    
    let ex_entry_clone2 = ex_entry.clone();
    let label_entry_clone2 = label_entry.clone();
    search_widget.connect_row_selected(move |result| {
        ex_entry_clone2.set_text(&result.code);
        label_entry_clone2.set_text(&result.name);
    });
    
    // Connect group search widget to populate alt_group field
    let alt_group_entry_clone = alt_group_entry.clone();
    group_search_widget.connect_row_activated(move |result| {
        alt_group_entry_clone.set_text(&result.name);
    });
    
    (search_expander, group_search_expander, ex_entry, label_entry, alt_group_entry, search_widget)
}

pub fn show_add_rpe_set_dialog(state: Arc<Mutex<AppState>>) {
    let dialog = Dialog::with_buttons(
        Some("Add RPE Set"),
        crate::ui::util::parent_for_dialog().as_ref(),
        DialogFlags::MODAL,
        &[("Cancel", ResponseType::Cancel), ("Add", ResponseType::Accept)]
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
    
    // Exercise Search Widget (Default)
    let (search_expander, group_search_expander, ex_entry, label_entry, alt_group_entry, _search_widget) = create_exercise_search_section(state.clone());
    content.append(&search_expander);
    content.append(&group_search_expander);

    // Manual Input Fields (Advanced option)
    let manual_expander = Expander::builder()
        .label("⚙️ Manual Entry (Advanced)")
        .expanded(false)
        .build();
    
    let manual_content = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .build();

    let ex_label = Label::new(Some("Exercise Code:"));
    ex_entry.set_text("SQUAT.BB.HB");
    
    let label_label = Label::new(Some("Exercise Name:"));
    label_entry.set_text("High Bar Back Squat");
    
    let sets_label = Label::new(Some("Sets:"));
    let sets_entry = SpinButton::with_range(1.0, 20.0, 1.0);
    sets_entry.set_value(4.0);
    
    // Reps range entries
    let reps_label = Label::new(Some("Reps (min-max):"));
    let reps_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .build();
    
    let min_reps = SpinButton::with_range(1.0, 50.0, 1.0);
    min_reps.set_value(5.0);
    let max_reps = SpinButton::with_range(1.0, 50.0, 1.0);
    max_reps.set_value(5.0);
    
    reps_box.append(&min_reps);
    reps_box.append(&Label::new(Some("-")));
    reps_box.append(&max_reps);
    
    let rpe_label = Label::new(Some("RPE:"));
    let rpe_entry = SpinButton::with_range(1.0, 10.0, 0.5);
    rpe_entry.set_value(8.5);
    
    let rir_label = Label::new(Some("RIR (optional):"));
    let rir_entry = SpinButton::with_range(0.0, 10.0, 1.0);
    rir_entry.set_value(1.0);
    
    let alt_group_label = Label::new(Some("Alternative Group (optional):"));
    
    manual_content.append(&ex_label);
    manual_content.append(&ex_entry);
    manual_content.append(&label_label);
    manual_content.append(&label_entry);
    manual_content.append(&alt_group_label);
    manual_content.append(&alt_group_entry);
    
    manual_expander.set_child(Some(&manual_content));
    content.append(&manual_expander);
    content.append(&sets_label);
    content.append(&sets_entry);
    content.append(&reps_label);
    content.append(&reps_box);
    content.append(&rpe_label);
    content.append(&rpe_entry);
    content.append(&rir_label);
    content.append(&rir_entry);
    
    dialog.content_area().append(&content);
    
    dialog.connect_response(clone!(@strong state, @strong ex_entry, @strong label_entry, @strong alt_group_entry, @strong sets_entry, @strong min_reps, @strong max_reps, @strong rpe_entry, @strong rir_entry => move |dialog, response| {
        if response == ResponseType::Accept {
            let ex_code = ex_entry.text().to_string();
            let ex_label = label_entry.text().to_string();
            let alt_group = alt_group_entry.text().to_string();
            let alt_group = if alt_group.trim().is_empty() { None } else { Some(alt_group) };
            let sets = sets_entry.value() as u32;
            let min_reps_val = min_reps.value() as u32;
            let max_reps_val = max_reps.value() as u32;
            let rpe = rpe_entry.value();
            let rir = if rir_entry.value() > 0.0 { Some(rir_entry.value() as u32) } else { None };
            
            add_rpe_set_to_plan(state.clone(), ex_code, ex_label, alt_group, sets, min_reps_val, max_reps_val, rpe, rir);
        }
        dialog.close();
    }));
    
    dialog.present();
}

pub fn show_add_percentage_set_dialog(state: Arc<Mutex<AppState>>) {
    let dialog = Dialog::with_buttons(
        Some("Add Percentage Set"),
        crate::ui::util::parent_for_dialog().as_ref(),
        DialogFlags::MODAL,
        &[("Cancel", ResponseType::Cancel), ("Add", ResponseType::Accept)]
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
    
    // Exercise Search Widget (Default)
    let (search_expander, group_search_expander, ex_entry, label_entry, _alt_group_entry, _search_widget) = create_exercise_search_section(state.clone());
    content.append(&search_expander);
    content.append(&group_search_expander);

    // Manual Input Fields (Advanced option)
    let manual_expander = Expander::builder()
        .label("⚙️ Manual Entry (Advanced)")
        .expanded(false)
        .build();
    
    let manual_content = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .build();

    let ex_label = Label::new(Some("Exercise Code:"));
    ex_entry.set_text("DEADLIFT.BB.CON");
    
    let label_label = Label::new(Some("Exercise Name:"));
    label_entry.set_text("Conventional Deadlift");
    
    let sets_label = Label::new(Some("Sets:"));
    let sets_entry = SpinButton::with_range(1.0, 20.0, 1.0);
    sets_entry.set_value(3.0);
    
    let reps_label = Label::new(Some("Reps:"));
    let reps_entry = SpinButton::with_range(1.0, 50.0, 1.0);
    reps_entry.set_value(3.0);
    
    let percentage_label = Label::new(Some("% of 1RM:"));
    let percentage_entry = SpinButton::with_range(30.0, 120.0, 5.0);
    percentage_entry.set_value(85.0);
    
    manual_content.append(&ex_label);
    manual_content.append(&ex_entry);
    manual_content.append(&label_label);
    manual_content.append(&label_entry);
    
    manual_expander.set_child(Some(&manual_content));
    content.append(&manual_expander);
    
    content.append(&sets_label);
    content.append(&sets_entry);
    content.append(&reps_label);
    content.append(&reps_entry);
    content.append(&percentage_label);
    content.append(&percentage_entry);
    
    dialog.content_area().append(&content);
    
    dialog.connect_response(clone!(@strong state, @strong ex_entry, @strong label_entry, @strong sets_entry, @strong reps_entry, @strong percentage_entry => move |dialog, response| {
        if response == ResponseType::Accept {
            let ex_code = ex_entry.text().to_string();
            let ex_label = label_entry.text().to_string();
            let sets = sets_entry.value() as u32;
            let reps = reps_entry.value() as u32;
            let percentage = percentage_entry.value();
            
            add_percentage_set_to_plan(state.clone(), ex_code, ex_label, sets, reps, percentage);
        }
        dialog.close();
    }));
    
    dialog.present();
}

pub fn show_add_amrap_dialog(state: Arc<Mutex<AppState>>) {
    let dialog = Dialog::with_buttons(
        Some("Add AMRAP Set"),
        crate::ui::util::parent_for_dialog().as_ref(),
        DialogFlags::MODAL,
        &[("Cancel", ResponseType::Cancel), ("Add", ResponseType::Accept)]
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
    
    // Exercise Search Widget (Default)
    let (search_expander, group_search_expander, ex_entry, label_entry, _alt_group_entry, _search_widget) = create_exercise_search_section(state.clone());
    content.append(&search_expander);
    content.append(&group_search_expander);

    // Manual Input Fields (Advanced option)
    let manual_expander = Expander::builder()
        .label("⚙️ Manual Entry (Advanced)")
        .expanded(false)
        .build();
    
    let manual_content = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .build();

    let ex_label = Label::new(Some("Exercise Code:"));
    ex_entry.set_text("PUSHUP.BW");
    
    let label_label = Label::new(Some("Exercise Name:"));
    label_entry.set_text("Push-ups");
    
    let time_label = Label::new(Some("Time (seconds):"));
    let time_entry = SpinButton::with_range(30.0, 600.0, 15.0);
    time_entry.set_value(60.0);
    
    let rest_label = Label::new(Some("Rest (seconds):"));
    let rest_entry = SpinButton::with_range(30.0, 300.0, 15.0);
    rest_entry.set_value(120.0);
    
    let auto_stop_check = CheckButton::with_label("Auto-stop when form breaks");
    
    manual_content.append(&ex_label);
    manual_content.append(&ex_entry);
    manual_content.append(&label_label);
    manual_content.append(&label_entry);
    
    manual_expander.set_child(Some(&manual_content));
    content.append(&manual_expander);
    
    content.append(&time_label);
    content.append(&time_entry);
    content.append(&rest_label);
    content.append(&rest_entry);
    content.append(&auto_stop_check);
    
    dialog.content_area().append(&content);
    
    dialog.connect_response(clone!(@strong state, @strong ex_entry, @strong label_entry, @strong time_entry, @strong rest_entry, @strong auto_stop_check => move |dialog, response| {
        if response == ResponseType::Accept {
            let ex_code = ex_entry.text().to_string();
            let ex_label = label_entry.text().to_string();
            let time_sec = time_entry.value() as u32;
            let rest_sec = rest_entry.value() as u32;
            let auto_stop = auto_stop_check.is_active();
            
            add_amrap_to_plan(state.clone(), ex_code, ex_label, time_sec, rest_sec, auto_stop);
        }
        dialog.close();
    }));
    
    dialog.present();
}

pub fn show_add_time_based_dialog(state: Arc<Mutex<AppState>>) {
    let dialog = Dialog::with_buttons(
        Some("Add Time-Based Set"),
        crate::ui::util::parent_for_dialog().as_ref(),
        DialogFlags::MODAL,
        &[("Cancel", ResponseType::Cancel), ("Add", ResponseType::Accept)]
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
    
    // Exercise Search Widget (Default)
    let (search_expander, group_search_expander, ex_entry, label_entry, _alt_group_entry, _search_widget) = create_exercise_search_section(state.clone());
    content.append(&search_expander);
    content.append(&group_search_expander);

    // Manual Input Fields (Advanced option)
    let manual_expander = Expander::builder()
        .label("⚙️ Manual Entry (Advanced)")
        .expanded(false)
        .build();
    
    let manual_content = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .build();

    let ex_label = Label::new(Some("Exercise Code:"));
    ex_entry.set_text("PLANK.BW");
    
    let label_label = Label::new(Some("Exercise Name:"));
    label_entry.set_text("Plank Hold");
    
    let sets_label = Label::new(Some("Sets:"));
    let sets_entry = SpinButton::with_range(1.0, 10.0, 1.0);
    sets_entry.set_value(3.0);
    
    let time_label = Label::new(Some("Duration (seconds):"));
    let time_entry = SpinButton::with_range(10.0, 300.0, 5.0);
    time_entry.set_value(45.0);
    
    let rest_label = Label::new(Some("Rest (seconds):"));
    let rest_entry = SpinButton::with_range(30.0, 300.0, 15.0);
    rest_entry.set_value(60.0);
    
    let interval_check = CheckButton::with_label("Use intervals");
    
    let work_label = Label::new(Some("Work interval (seconds):"));
    let work_entry = SpinButton::with_range(5.0, 60.0, 5.0);
    work_entry.set_value(15.0);
    work_entry.set_sensitive(false);
    
    let interval_rest_label = Label::new(Some("Rest interval (seconds):"));
    let interval_rest_entry = SpinButton::with_range(5.0, 60.0, 5.0);
    interval_rest_entry.set_value(5.0);
    interval_rest_entry.set_sensitive(false);
    
    let repeats_label = Label::new(Some("Interval repeats:"));
    let repeats_entry = SpinButton::with_range(2.0, 20.0, 1.0);
    repeats_entry.set_value(6.0);
    repeats_entry.set_sensitive(false);
    
    interval_check.connect_toggled(clone!(@strong work_entry, @strong interval_rest_entry, @strong repeats_entry => move |check| {
        let active = check.is_active();
        work_entry.set_sensitive(active);
        interval_rest_entry.set_sensitive(active);
        repeats_entry.set_sensitive(active);
    }));
    
    manual_content.append(&ex_label);
    manual_content.append(&ex_entry);
    manual_content.append(&label_label);
    manual_content.append(&label_entry);
    
    manual_expander.set_child(Some(&manual_content));
    content.append(&manual_expander);
    
    content.append(&sets_label);
    content.append(&sets_entry);
    content.append(&time_label);
    content.append(&time_entry);
    content.append(&rest_label);
    content.append(&rest_entry);
    content.append(&interval_check);
    content.append(&work_label);
    content.append(&work_entry);
    content.append(&interval_rest_label);
    content.append(&interval_rest_entry);
    content.append(&repeats_label);
    content.append(&repeats_entry);
    
    dialog.content_area().append(&content);
    
    dialog.connect_response(clone!(@strong state, @strong ex_entry, @strong label_entry, @strong sets_entry, @strong time_entry, @strong rest_entry, @strong interval_check, @strong work_entry, @strong interval_rest_entry, @strong repeats_entry => move |dialog, response| {
        if response == ResponseType::Accept {
            let ex_code = ex_entry.text().to_string();
            let ex_label = label_entry.text().to_string();
            let sets = sets_entry.value() as u32;
            let time_sec = time_entry.value() as u32;
            let rest_sec = rest_entry.value() as u32;
            
            let interval = if interval_check.is_active() {
                Some((
                    work_entry.value() as u32,
                    interval_rest_entry.value() as u32,
                    repeats_entry.value() as u32
                ))
            } else {
                None
            };
            
            add_time_based_to_plan(state.clone(), ex_code, ex_label, sets, time_sec, rest_sec, interval);
        }
        dialog.close();
    }));
    
    dialog.present();
}
