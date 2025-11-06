// Circuit creation dialog extracted from complex_dialogs.rs

use crate::state::AppState;
use glib::clone;
use gtk4::prelude::*;
use gtk4::{Orientation, Label, StringList, ListView, ScrolledWindow, NoSelection, SignalListItemFactory, StringObject, Box as GtkBox, SpinButton, Button, Dialog, DialogFlags, ResponseType};
use std::sync::{Arc, Mutex};

use crate::ui::exercise_menus::exercise_data::CircuitExerciseData;
use crate::operations::plan_ops::add_circuit_to_plan;

pub fn show_add_circuit_dialog(state: Arc<Mutex<AppState>>) {
    let dialog = Dialog::with_buttons(
        Some("Create Circuit"),
        crate::ui::util::parent_for_dialog().as_ref(),
        DialogFlags::MODAL,
        &[("Cancel", ResponseType::Cancel), ("Create", ResponseType::Accept)]
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

    // Circuit parameters
    let params_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .build();

    let rounds_label = Label::new(Some("Rounds:"));
    let rounds_entry = SpinButton::with_range(1.0, 10.0, 1.0);
    rounds_entry.set_value(3.0);

    let rest_label = Label::new(Some("Rest between exercises (sec):"));
    let rest_entry = SpinButton::with_range(0.0, 60.0, 5.0);
    rest_entry.set_value(15.0);

    let rest_rounds_label = Label::new(Some("Rest between rounds (sec):"));
    let rest_rounds_entry = SpinButton::with_range(60.0, 600.0, 30.0);
    rest_rounds_entry.set_value(120.0);

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

    // Exercise list
    let exercises_label = Label::new(Some("Exercises in Circuit:"));
    content.append(&exercises_label);

    let exercises_model = StringList::new(&[]);
    let selection_model = NoSelection::new(Some(exercises_model.clone()));

    // Create a factory to display string items
    let factory = SignalListItemFactory::new();
    factory.connect_setup(move |_, list_item| {
        let label = Label::new(None);
        list_item.set_child(Some(&label));
    });
    factory.connect_bind(move |_, list_item| {
        let string_object = list_item.item().unwrap().downcast::<StringObject>().unwrap();
        let label = list_item.child().unwrap().downcast::<Label>().unwrap();
        label.set_text(&string_object.string());
    });

    let exercises_view = ListView::new(Some(selection_model.clone()), Some(factory));

    let scrolled_exercises = ScrolledWindow::builder()
        .child(&exercises_view)
        .min_content_height(150)
        .build();

    content.append(&scrolled_exercises);

    // Exercise management buttons
    let exercise_buttons_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .build();

    let add_exercise_btn = Button::with_label("+ Add Exercise");
    let remove_exercise_btn = Button::with_label("- Remove Selected");
    let move_up_btn = Button::with_label("↑ Move Up");
    let move_down_btn = Button::with_label("↓ Move Down");

    exercise_buttons_box.append(&add_exercise_btn);
    exercise_buttons_box.append(&remove_exercise_btn);
    exercise_buttons_box.append(&move_up_btn);
    exercise_buttons_box.append(&move_down_btn);

    content.append(&exercise_buttons_box);

    // Store exercises data for the circuit
    let exercises_data = std::rc::Rc::new(std::cell::RefCell::new(Vec::<CircuitExerciseData>::new()));

    // Add exercise button handler
    add_exercise_btn.connect_clicked(clone!(@strong exercises_model, @strong exercises_data => move |_| {
        show_add_circuit_exercise_dialog(exercises_model.clone(), exercises_data.clone());
    }));

    // Remove exercise button handler (removes last added exercise)
    remove_exercise_btn.connect_clicked(clone!(@strong exercises_model, @strong exercises_data => move |_| {
        let count = exercises_data.borrow().len();
        if count > 0 {
            exercises_model.remove(count as u32 - 1);
            exercises_data.borrow_mut().remove(count - 1);
        }
    }));

    // Move up button handler (moves last exercise up)
    move_up_btn.connect_clicked(clone!(@strong exercises_model, @strong exercises_data => move |_| {
        let count = exercises_data.borrow().len();
        if count > 1 {
            let last_idx = count - 1;
            let prev_idx = count - 2;

            // Swap in data
            exercises_data.borrow_mut().swap(last_idx, prev_idx);
            // Rebuild model
            exercises_model.splice(0, exercises_model.n_items(), &[] as &[&str]);
            let data = exercises_data.borrow();
            for exercise in data.iter() {
                let display_text = if let Some(time) = exercise.time_sec {
                    format!("{}: {}s", exercise.ex_name, time)
                } else if exercise.reps_min == exercise.reps_max {
                    format!("{}: {} reps", exercise.ex_name, exercise.reps_min)
                } else {
                    format!("{}: {}-{} reps", exercise.ex_name, exercise.reps_min, exercise.reps_max)
                };
                exercises_model.append(&display_text);
            }
        }
    }));

    // Move down button handler (moves second-to-last exercise down)
    move_down_btn.connect_clicked(clone!(@strong exercises_model, @strong exercises_data => move |_| {
        let count = exercises_data.borrow().len();
        if count > 1 {
            let second_last_idx = count - 2;
            let last_idx = count - 1;

            // Swap in data
            exercises_data.borrow_mut().swap(second_last_idx, last_idx);
            // Rebuild model
            exercises_model.splice(0, exercises_model.n_items(), &[] as &[&str]);
            let data = exercises_data.borrow();
            for exercise in data.iter() {
                let display_text = if let Some(time) = exercise.time_sec {
                    format!("{}: {}s", exercise.ex_name, time)
                } else if exercise.reps_min == exercise.reps_max {
                    format!("{}: {} reps", exercise.ex_name, exercise.reps_min)
                } else {
                    format!("{}: {}-{} reps", exercise.ex_name, exercise.reps_min, exercise.reps_max)
                };
                exercises_model.append(&display_text);
            }
        }
    }));

    dialog.content_area().append(&content);

    dialog.connect_response(clone!(@strong state, @strong rounds_entry, @strong rest_entry, @strong rest_rounds_entry, @strong exercises_data => move |dialog, response| {
        if response == ResponseType::Accept {
            let rounds = rounds_entry.value() as u32;
            let rest_sec = rest_entry.value() as u32;
            let rest_between_rounds_sec = rest_rounds_entry.value() as u32;

            let exercises = exercises_data.borrow().clone();

            if !exercises.is_empty() {
                add_circuit_to_plan(state.clone(), rounds, rest_sec, rest_between_rounds_sec, exercises);
            } else {
                println!("Cannot create circuit with no exercises");
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
        &[("Cancel", ResponseType::Cancel), ("Add", ResponseType::Accept)]
    );

    let content = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .margin_start(20)
        .margin_end(20)
        .margin_top(20)
        .margin_bottom(20)
        .spacing(12)
        .build();

    // Basic inputs for circuit items
    let name_label = Label::new(Some("Exercise Name:"));
    let name_entry = gtk4::Entry::new();
    name_entry.set_text("Push-up");

    let reps_label = Label::new(Some("Reps (min-max) or Time (sec):"));
    let reps_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .build();

    let min_reps = SpinButton::with_range(1.0, 200.0, 1.0);
    min_reps.set_value(10.0);
    let max_reps = SpinButton::with_range(1.0, 200.0, 1.0);
    max_reps.set_value(15.0);

    let time_entry = SpinButton::with_range(0.0, 600.0, 5.0);
    time_entry.set_value(0.0);

    reps_box.append(&min_reps);
    reps_box.append(&Label::new(Some("-")));
    reps_box.append(&max_reps);
    reps_box.append(&Label::new(Some("or")));
    reps_box.append(&time_entry);

    content.append(&name_label);
    content.append(&name_entry);
    content.append(&reps_label);
    content.append(&reps_box);

    dialog.content_area().append(&content);

    dialog.connect_response(clone!(@strong exercises_model, @strong exercises_data, @strong name_entry, @strong min_reps, @strong max_reps, @strong time_entry => move |dialog, response| {
        if response == ResponseType::Accept {
            let ex_name = name_entry.text().to_string();
            let reps_min = min_reps.value() as u32;
            let reps_max = max_reps.value() as u32;
            let time_sec = if time_entry.value() > 0.0 { Some(time_entry.value() as u32) } else { None };

            let item = CircuitExerciseData {
                ex_code: ex_name.clone(), // For now, use name as code placeholder if not selected from DB
                ex_name: ex_name.clone(),
                reps_min,
                reps_max,
                time_sec,
            };

            exercises_data.borrow_mut().push(item);

            let display_text = if let Some(time) = time_sec {
                format!("{}: {}s", ex_name, time)
            } else if reps_min == reps_max {
                format!("{}: {} reps", ex_name, reps_min)
            } else {
                format!("{}: {}-{} reps", ex_name, reps_min, reps_max)
            };
            exercises_model.append(&display_text);
        }
        dialog.close();
    }));

    dialog.present();
}
