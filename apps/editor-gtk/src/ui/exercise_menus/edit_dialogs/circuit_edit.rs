// Circuit edit dialog functionality

use super::dialog_components::{
    create_exercise_buttons, create_exercise_list_view, move_exercise_down, move_exercise_up,
    remove_last_exercise,
};
use crate::operations::plan_ops::update_circuit_in_plan;
use crate::state::AppState;
use crate::ui::exercise_menus::exercise_data::CircuitExerciseData;
use crate::ui::widgets::ExerciseSearchWidget;
use glib::clone;
use gtk4::prelude::*;
use gtk4::{
    Box as GtkBox, Dialog, DialogFlags, Label, Orientation, ResponseType, SpinButton, StringList,
};
use std::sync::{Arc, Mutex};
use weightlifting_core::{RepsOrRange, TimeOrRange};

pub fn show_edit_circuit_dialog(
    state: Arc<Mutex<AppState>>,
    circuit: weightlifting_core::CircuitSegment,
    day_index: usize,
    segment_index: usize,
) {
    let dialog = Dialog::with_buttons(
        Some("Edit Circuit"),
        crate::ui::util::parent_for_dialog().as_ref(),
        DialogFlags::MODAL,
        &[
            ("Cancel", ResponseType::Cancel),
            ("Update", ResponseType::Accept),
        ],
    );
    crate::ui::util::standardize_dialog(&dialog);
    dialog.set_default_size(500, 400);
    crate::ui::util::standardize_dialog(&dialog);
    let content = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .margin_start(20)
        .margin_end(20)
        .margin_top(20)
        .margin_bottom(20)
        .spacing(12)
        .build();

    // Circuit parameters - populate with existing values
    let params_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .build();

    let rounds_label = Label::new(Some("Rounds:"));
    let rounds_entry = SpinButton::with_range(1.0, 10.0, 1.0);
    rounds_entry.set_value(circuit.rounds as f64);

    let rest_label = Label::new(Some("Rest between exercises (sec):"));
    let rest_entry = SpinButton::with_range(0.0, 60.0, 5.0);
    rest_entry.set_value(circuit.rest_sec as f64);

    let rest_rounds_label = Label::new(Some("Rest between rounds (sec):"));
    let rest_rounds_entry = SpinButton::with_range(60.0, 600.0, 30.0);
    rest_rounds_entry.set_value(circuit.rest_between_rounds_sec as f64);

    params_box.append(&rounds_label);
    params_box.append(&rounds_entry);
    params_box.append(&rest_label);
    params_box.append(&rest_entry);

    content.append(&params_box);

    let rounds_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .build();

    rounds_box.append(&rest_rounds_label);
    rounds_box.append(&rest_rounds_entry);

    content.append(&rounds_box);

    // Exercise list - populate with existing exercises
    let exercises_label = Label::new(Some("Exercises in Circuit:"));
    content.append(&exercises_label);

    let exercises_model = StringList::new(&[]);
    let (_, scrolled_exercises, _) = create_exercise_list_view(exercises_model.clone());
    content.append(&scrolled_exercises);

    // Exercise management buttons
    let (exercise_buttons_box, add_exercise_btn, remove_exercise_btn, move_up_btn, move_down_btn) =
        create_exercise_buttons();
    content.append(&exercise_buttons_box);

    // Convert existing exercises to our data structure and populate model
    let exercises_data =
        std::rc::Rc::new(std::cell::RefCell::new(Vec::<CircuitExerciseData>::new()));

    // Load existing exercises
    for item in &circuit.items {
        let (reps_min, reps_max, time_sec) = match (&item.reps, &item.time_sec) {
            (Some(reps), None) => match reps {
                RepsOrRange::Range(range) => (range.min, range.max, None),
            },
            (None, Some(time)) => {
                match time {
                    TimeOrRange::Fixed(t) => (1, 1, Some(*t)),
                    TimeOrRange::Range(range) => (1, 1, Some(range.min)), // Use min for editing
                }
            }
            (None, None) => (1, 1, None),
            (Some(_), Some(_)) => (1, 1, None), // Shouldn't happen, prefer reps
        };

        let exercise_data = CircuitExerciseData {
            ex_code: item.ex.clone(),
            ex_name: item.ex.clone(), // We'll use the code as name for editing
            reps_min,
            reps_max,
            time_sec,
        };

        let display_text = format_circuit_exercise_display(&exercise_data);

        exercises_data.borrow_mut().push(exercise_data);
        exercises_model.append(&display_text);
    }

    // Exercise management button handlers
    add_exercise_btn.connect_clicked(
        clone!(@strong exercises_model, @strong exercises_data => move |_| {
            show_add_circuit_exercise_dialog(exercises_model.clone(), exercises_data.clone());
        }),
    );

    remove_exercise_btn.connect_clicked(
        clone!(@strong exercises_model, @strong exercises_data => move |_| {
            remove_last_exercise(&exercises_model, &exercises_data);
        }),
    );

    move_up_btn.connect_clicked(
        clone!(@strong exercises_model, @strong exercises_data => move |_| {
            move_exercise_up(&exercises_model, &exercises_data, format_circuit_exercise_display);
        }),
    );

    move_down_btn.connect_clicked(
        clone!(@strong exercises_model, @strong exercises_data => move |_| {
            move_exercise_down(&exercises_model, &exercises_data, format_circuit_exercise_display);
        }),
    );

    dialog.content_area().append(&content);

    dialog.connect_response(clone!(@strong state, @strong rounds_entry, @strong rest_entry, @strong rest_rounds_entry, @strong exercises_data => move |dialog, response| {
        if response == ResponseType::Accept {
            let rounds = rounds_entry.value() as u32;
            let rest_sec = rest_entry.value() as u32;
            let rest_between_rounds_sec = rest_rounds_entry.value() as u32;

            let exercises = exercises_data.borrow().clone();

            if !exercises.is_empty() {
                update_circuit_in_plan(state.clone(), day_index, segment_index, rounds, rest_sec, rest_between_rounds_sec, exercises);
            }
        }
        dialog.close();
    }));

    dialog.present();
}

fn show_add_circuit_exercise_dialog(
    exercises_model: StringList,
    exercises_data: std::rc::Rc<std::cell::RefCell<Vec<CircuitExerciseData>>>,
) {
    let dialog = Dialog::with_buttons(
        Some("Add Exercise to Circuit"),
        crate::ui::util::parent_for_dialog().as_ref(),
        DialogFlags::MODAL,
        &[
            ("Cancel", ResponseType::Cancel),
            ("Add", ResponseType::Accept),
        ],
    );

    let content = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .margin_start(20)
        .margin_end(20)
        .margin_top(20)
        .margin_bottom(20)
        .spacing(12)
        .build();

    // Exercise Search Widget (search-first)
    let search_widget = ExerciseSearchWidget::new();
    if let Err(e) =
        search_widget.set_database_path("/home/jawaad/weightlifting-desktop/exercises.db")
    {
        println!("Failed to connect to exercise database: {}", e);
    }
    let search_expander = gtk4::Expander::builder()
        .label("🔍 Search Exercise Database")
        .child(&search_widget.container)
        .expanded(true)
        .build();
    content.append(&search_expander);

    // Manual Input Fields (Advanced)
    let manual_expander = gtk4::Expander::builder()
        .label("⚙️ Manual Entry (Advanced)")
        .expanded(false)
        .build();
    let manual_box = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .build();
    let ex_label = Label::new(Some("Exercise Code:"));
    let ex_entry = gtk4::Entry::new();
    ex_entry.set_text("BURPEE.BW");
    let name_label = Label::new(Some("Exercise Name:"));
    let name_entry = gtk4::Entry::new();
    name_entry.set_text("Burpees");
    manual_box.append(&ex_label);
    manual_box.append(&ex_entry);
    manual_box.append(&name_label);
    manual_box.append(&name_entry);
    manual_expander.set_child(Some(&manual_box));
    content.append(&manual_expander);

    // Mode selection - Reps or Time
    let mode_check = gtk4::CheckButton::with_label("Time-based (instead of reps)");
    content.append(&mode_check);

    let reps_label = Label::new(Some("Reps (min-max):"));
    let reps_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .build();

    let min_reps = SpinButton::with_range(1.0, 100.0, 1.0);
    min_reps.set_value(10.0);
    let max_reps = SpinButton::with_range(1.0, 100.0, 1.0);
    max_reps.set_value(15.0);

    reps_box.append(&min_reps);
    reps_box.append(&Label::new(Some("-")));
    reps_box.append(&max_reps);

    let time_label = Label::new(Some("Time (seconds):"));
    let time_entry = SpinButton::with_range(10.0, 300.0, 5.0);
    time_entry.set_value(30.0);
    time_entry.set_sensitive(false);

    // Toggle between reps and time mode
    mode_check.connect_toggled(
        clone!(@strong min_reps, @strong max_reps, @strong time_entry => move |check| {
            let is_time_based = check.is_active();
            min_reps.set_sensitive(!is_time_based);
            max_reps.set_sensitive(!is_time_based);
            time_entry.set_sensitive(is_time_based);
        }),
    );

    // Wire search selection to fields
    let ex_entry_clone = ex_entry.clone();
    let name_entry_clone = name_entry.clone();
    let sw_for_select = search_widget.clone();
    sw_for_select.connect_row_activated(move |res| {
        ex_entry_clone.set_text(&res.code);
        name_entry_clone.set_text(&res.name);
    });
    let ex_entry_clone2 = ex_entry.clone();
    let name_entry_clone2 = name_entry.clone();
    search_widget.connect_row_selected(move |res| {
        ex_entry_clone2.set_text(&res.code);
        name_entry_clone2.set_text(&res.name);
    });
    content.append(&reps_label);
    content.append(&reps_box);
    content.append(&time_label);
    content.append(&time_entry);

    dialog.content_area().append(&content);
    // Keyboard: Enter accepts current selection; focus search on open
    let dialog_clone = dialog.clone();
    let sw_for_keys = search_widget.clone();
    let ex_entry_keys = ex_entry.clone();
    let name_entry_keys = name_entry.clone();
    let key_controller = gtk4::EventControllerKey::new();
    key_controller.connect_key_pressed(move |_, key, _, modifiers| match key {
        gtk4::gdk::Key::Return | gtk4::gdk::Key::KP_Enter => {
            if modifiers.contains(gtk4::gdk::ModifierType::CONTROL_MASK) {
                dialog_clone.response(ResponseType::Accept);
                return glib::Propagation::Stop;
            }
            if let Some(sel) = sw_for_keys.selected_exercise() {
                ex_entry_keys.set_text(&sel.code);
                name_entry_keys.set_text(&sel.name);
                dialog_clone.response(ResponseType::Accept);
                return glib::Propagation::Stop;
            }
            glib::Propagation::Proceed
        }
        gtk4::gdk::Key::Escape => {
            dialog_clone.response(ResponseType::Cancel);
            glib::Propagation::Stop
        }
        _ => glib::Propagation::Proceed,
    });
    dialog.add_controller(key_controller);
    let sw_for_show = search_widget.clone();
    dialog.connect_show(move |_| {
        if let Some(container) = sw_for_show.container.first_child() {
            if let Some(entry) = container.downcast_ref::<gtk4::Entry>() {
                entry.grab_focus();
            }
        }
    });

    dialog.connect_response(clone!(@strong exercises_model, @strong exercises_data, @strong ex_entry, @strong name_entry, @strong mode_check, @strong min_reps, @strong max_reps, @strong time_entry => move |dialog, response| {
        if response == ResponseType::Accept {
            let ex_code = ex_entry.text().to_string();
            let ex_name = name_entry.text().to_string();
            let is_time_based = mode_check.is_active();

            let (reps_min, reps_max, time_sec) = if is_time_based {
                (1, 1, Some(time_entry.value() as u32)) // Default to 1 rep when time-based
            } else {
                (min_reps.value() as u32, max_reps.value() as u32, None)
            };

            let exercise_data = CircuitExerciseData {
                ex_code: ex_code.clone(),
                ex_name: ex_name.clone(),
                reps_min,
                reps_max,
                time_sec,
            };

            exercises_data.borrow_mut().push(exercise_data.clone());

            let display_text = format_circuit_exercise_display(&exercise_data);
            exercises_model.append(&display_text);
        }
        dialog.close();
    }));

    dialog.present();
}

/// Format circuit exercise data for display in the list
fn format_circuit_exercise_display(exercise: &CircuitExerciseData) -> String {
    if let Some(time) = exercise.time_sec {
        format!("{}: {}s", exercise.ex_name, time)
    } else if exercise.reps_min == exercise.reps_max {
        format!("{}: {} reps", exercise.ex_name, exercise.reps_min)
    } else {
        format!(
            "{}: {}-{} reps",
            exercise.ex_name, exercise.reps_min, exercise.reps_max
        )
    }
}
