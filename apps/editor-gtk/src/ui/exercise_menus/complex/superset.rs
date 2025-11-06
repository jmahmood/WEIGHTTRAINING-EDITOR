// Superset creation dialog extracted from complex_dialogs.rs

use crate::state::AppState;
use super::components::create_exercise_search_section_complex;
use glib::clone;
use gtk4::prelude::*;
use gtk4::{Orientation, Label, StringList, ListView, ScrolledWindow, NoSelection, SignalListItemFactory, StringObject, Box as GtkBox, Entry, SpinButton, Button, Dialog, DialogFlags, ResponseType, Expander};
use std::sync::{Arc, Mutex};

use crate::ui::exercise_menus::exercise_data::SupersetExerciseData;
use crate::operations::plan_ops::add_superset_to_plan;

pub fn show_add_superset_dialog(state: Arc<Mutex<AppState>>) {
    let dialog = Dialog::with_buttons(
        Some("Create Superset"),
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

    // Superset label
    let label_label = Label::new(Some("Superset Label (optional):"));
    let label_entry = Entry::builder()
        .text("Upper Body Superset")
        .build();

    content.append(&label_label);
    content.append(&label_entry);

    // Superset parameters
    let params_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .build();

    let rounds_label = Label::new(Some("Rounds:"));
    let rounds_entry = SpinButton::with_range(1.0, 10.0, 1.0);
    rounds_entry.set_value(3.0);

    let rest_label = Label::new(Some("Rest between exercises (sec):"));
    let rest_entry = SpinButton::with_range(0.0, 180.0, 15.0);
    rest_entry.set_value(30.0);

    let rest_rounds_label = Label::new(Some("Rest between rounds (sec):"));
    let rest_rounds_entry = SpinButton::with_range(60.0, 600.0, 30.0);
    rest_rounds_entry.set_value(180.0);

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
    let exercises_label = Label::new(Some("Exercises in Superset:"));
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

    // Store exercises data for the superset
    let exercises_data = std::rc::Rc::new(std::cell::RefCell::new(Vec::<SupersetExerciseData>::new()));

    // Add exercise button handler
    add_exercise_btn.connect_clicked(clone!(@strong state, @strong exercises_model, @strong exercises_data => move |_| {
        show_add_superset_exercise_dialog(exercises_model.clone(), exercises_data.clone(), state.clone());
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
                let display_text = if exercise.reps_min == exercise.reps_max {
                    format!("{}: {}x{} @ RPE {:?}", exercise.ex_name, exercise.sets, exercise.reps_min, exercise.rpe.unwrap_or(0.0))
                } else {
                    format!("{}: {}x{}-{} @ RPE {:?}", exercise.ex_name, exercise.sets, exercise.reps_min, exercise.reps_max, exercise.rpe.unwrap_or(0.0))
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
                let display_text = if exercise.reps_min == exercise.reps_max {
                    format!("{}: {}x{} @ RPE {:?}", exercise.ex_name, exercise.sets, exercise.reps_min, exercise.rpe.unwrap_or(0.0))
                } else {
                    format!("{}: {}x{}-{} @ RPE {:?}", exercise.ex_name, exercise.sets, exercise.reps_min, exercise.reps_max, exercise.rpe.unwrap_or(0.0))
                };
                exercises_model.append(&display_text);
            }
        }
    }));

    dialog.content_area().append(&content);

    dialog.connect_response(clone!(@strong state, @strong label_entry, @strong rounds_entry, @strong rest_entry, @strong rest_rounds_entry, @strong exercises_data => move |dialog, response| {
        if response == ResponseType::Accept {
            let label = label_entry.text().to_string();
            let label = if label.trim().is_empty() { None } else { Some(label) };
            let rounds = rounds_entry.value() as u32;
            let rest_sec = rest_entry.value() as u32;
            let rest_between_rounds_sec = rest_rounds_entry.value() as u32;

            let exercises = exercises_data.borrow().clone();

            if !exercises.is_empty() {
                add_superset_to_plan(state.clone(), label, rounds, rest_sec, rest_between_rounds_sec, exercises);
            } else {
                println!("Cannot create superset with no exercises");
            }
        }
        dialog.close();
    }));

    dialog.present();
}

fn show_add_superset_exercise_dialog(
    exercises_model: StringList,
    exercises_data: std::rc::Rc<std::cell::RefCell<Vec<SupersetExerciseData>>>,
    _state: Arc<Mutex<AppState>>,
) {
    let dialog = Dialog::with_buttons(
        Some("Add Exercise to Superset"),
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

    // Exercise Search Widget (Default)
    let (search_expander, ex_entry, name_entry, _search_widget) = create_exercise_search_section_complex();
    content.append(&search_expander);

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
    ex_entry.set_text("BENCH.BB.FLAT");

    let name_label = Label::new(Some("Exercise Name:"));
    name_entry.set_text("Bench Press");

    let sets_label = Label::new(Some("Sets:"));
    let sets_entry = SpinButton::with_range(1.0, 20.0, 1.0);
    sets_entry.set_value(3.0);

    let reps_label = Label::new(Some("Reps (min-max):"));
    let reps_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .build();

    let min_reps = SpinButton::with_range(1.0, 50.0, 1.0);
    min_reps.set_value(8.0);
    let max_reps = SpinButton::with_range(1.0, 50.0, 1.0);
    max_reps.set_value(10.0);

    reps_box.append(&min_reps);
    reps_box.append(&Label::new(Some("-")));
    reps_box.append(&max_reps);

    let rpe_label = Label::new(Some("RPE (optional):"));
    let rpe_entry = SpinButton::with_range(6.0, 10.0, 0.5);
    rpe_entry.set_value(8.0);

    manual_content.append(&ex_label);
    manual_content.append(&ex_entry);
    manual_content.append(&name_label);
    manual_content.append(&name_entry);

    manual_expander.set_child(Some(&manual_content));
    content.append(&manual_expander);

    content.append(&sets_label);
    content.append(&sets_entry);
    content.append(&reps_label);
    content.append(&reps_box);
    content.append(&rpe_label);
    content.append(&rpe_entry);

    dialog.content_area().append(&content);

    dialog.connect_response(clone!(@strong exercises_model, @strong exercises_data, @strong ex_entry, @strong name_entry, @strong sets_entry, @strong min_reps, @strong max_reps, @strong rpe_entry => move |dialog, response| {
        if response == ResponseType::Accept {
            let ex_code = ex_entry.text().to_string();
            let ex_name = name_entry.text().to_string();
            let sets = sets_entry.value() as u32;
            let reps_min = min_reps.value() as u32;
            let reps_max = max_reps.value() as u32;
            let rpe = if rpe_entry.value() >= 6.0 { Some(rpe_entry.value()) } else { None };

            let exercise_data = SupersetExerciseData {
                ex_code: ex_code.clone(),
                ex_name: ex_name.clone(),
                sets,
                reps_min,
                reps_max,
                rpe,
                alt_group: None,
            };

            exercises_data.borrow_mut().push(exercise_data);

            let display_text = if reps_min == reps_max {
                format!("{}: {}x{} @ RPE {:?}", ex_name, sets, reps_min, rpe.unwrap_or(0.0))
            } else {
                format!("{}: {}x{}-{} @ RPE {:?}", ex_name, sets, reps_min, reps_max, rpe.unwrap_or(0.0))
            };

            exercises_model.append(&display_text);
        }
        dialog.close();
    }));

    dialog.present();
}
