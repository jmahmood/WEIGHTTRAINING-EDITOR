// Superset edit dialog functionality

use crate::state::AppState;
use crate::ui::exercise_menus::exercise_data::SupersetExerciseData;
use crate::operations::plan_ops::update_superset_in_plan;
use super::dialog_components::{
    create_exercise_list_view, create_exercise_buttons,
    remove_last_exercise, move_exercise_up, move_exercise_down, rebuild_exercise_model
};
use glib::clone;
use gtk4::prelude::*;
use gtk4::{
    Orientation, Label, StringList, Box as GtkBox, Entry, 
    SpinButton, Dialog, DialogFlags, ResponseType, ListItem
};
use std::sync::{Arc, Mutex};
use weightlifting_core::RepsOrRange;
use crate::ui::widgets::{ExerciseSearchWidget, GroupSearchWidget};

pub fn show_edit_superset_dialog(state: Arc<Mutex<AppState>>, superset: weightlifting_core::SupersetSegment, day_index: usize, segment_index: usize) {
    let dialog = Dialog::with_buttons(
        Some("Edit Superset"),
        crate::ui::util::parent_for_dialog().as_ref(),
        DialogFlags::MODAL,
        &[("Cancel", ResponseType::Cancel), ("Update", ResponseType::Accept)]
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
        .text(superset.label.unwrap_or("Upper Body Superset".to_string()))
        .build();
    
    content.append(&label_label);
    content.append(&label_entry);
    
    // Superset parameters - populate with existing values
    let params_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .build();
    
    let rounds_label = Label::new(Some("Rounds:"));
    let rounds_entry = SpinButton::with_range(1.0, 10.0, 1.0);
    rounds_entry.set_value(superset.rounds as f64);
    
    let rest_label = Label::new(Some("Rest between exercises (sec):"));
    let rest_entry = SpinButton::with_range(0.0, 180.0, 15.0);
    rest_entry.set_value(superset.rest_sec as f64);
    
    let rest_rounds_label = Label::new(Some("Rest between rounds (sec):"));
    let rest_rounds_entry = SpinButton::with_range(60.0, 600.0, 30.0);
    rest_rounds_entry.set_value(superset.rest_between_rounds_sec as f64);
    
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
    let exercises_label = Label::new(Some("Exercises in Superset:"));
    content.append(&exercises_label);
    
    let exercises_model = StringList::new(&[]);
    let (_exercises_view, scrolled_exercises, factory) = create_exercise_list_view(exercises_model.clone());
    content.append(&scrolled_exercises);
    
    // Exercise management buttons
    let (exercise_buttons_box, add_exercise_btn, remove_exercise_btn, move_up_btn, move_down_btn) = create_exercise_buttons();
    content.append(&exercise_buttons_box);
    
    // Convert existing exercises to our data structure and populate model
    let exercises_data = std::rc::Rc::new(std::cell::RefCell::new(Vec::<SupersetExerciseData>::new()));
    
    // Load existing exercises
    for item in &superset.items {
        let (reps_min, reps_max) = if let Some(reps) = &item.reps {
            match reps {
                RepsOrRange::Range(range) => (range.min, range.max),
            }
        } else {
            (1, 1)
        };
        
        let exercise_data = SupersetExerciseData {
            ex_code: item.ex.clone(),
            ex_name: item.ex.clone(), // We'll use the code as name for editing
            alt_group: item.alt_group.clone(),
            sets: item.sets,
            reps_min,
            reps_max,
            rpe: item.rpe,
        };
        
        let display_text = format_superset_exercise_display(&exercise_data);
        
        exercises_data.borrow_mut().push(exercise_data);
        exercises_model.append(&display_text);
    }
    // Enable right-click to show context menu with Edit for an exercise
    {
        let exercises_data_for_click = exercises_data.clone();
        let exercises_model_for_click = exercises_model.clone();
        factory.connect_setup(clone!(@strong exercises_data_for_click, @strong exercises_model_for_click => move |_, list_item: &ListItem| {
            if let Some(child) = list_item.child() {
                let gesture = gtk4::GestureClick::new();
                gesture.set_button(3); // Right-click
                let li = list_item.clone();
                let child_for_menu = child.clone();
                gesture.connect_pressed(clone!(@strong exercises_data_for_click, @strong exercises_model_for_click => move |_, n_press, x, y| {
                    if n_press == 1 {
                        crate::ui::util::show_edit_context_menu(
                            &child_for_menu,
                            x,
                            y,
                            Box::new(clone!(@strong exercises_data_for_click, @strong exercises_model_for_click, @strong li => move || {
                                let idx = li.position() as usize;
                                if let Some(current) = exercises_data_for_click.borrow().get(idx).cloned() {
                                    show_edit_superset_exercise_dialog(idx, current, exercises_model_for_click.clone(), exercises_data_for_click.clone());
                                }
                            }))
                        );
                    }
                }));
                child.add_controller(gesture);
            }
        }));
    }

    // Exercise management button handlers
    add_exercise_btn.connect_clicked(clone!(@strong state, @strong exercises_model, @strong exercises_data => move |_| {
        show_add_superset_exercise_dialog(state.clone(), exercises_model.clone(), exercises_data.clone());
    }));
    
    remove_exercise_btn.connect_clicked(clone!(@strong exercises_model, @strong exercises_data => move |_| {
        remove_last_exercise(&exercises_model, &exercises_data);
    }));

    move_up_btn.connect_clicked(clone!(@strong exercises_model, @strong exercises_data => move |_| {
        move_exercise_up(&exercises_model, &exercises_data, format_superset_exercise_display);
    }));

    move_down_btn.connect_clicked(clone!(@strong exercises_model, @strong exercises_data => move |_| {
        move_exercise_down(&exercises_model, &exercises_data, format_superset_exercise_display);
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
                update_superset_in_plan(state.clone(), day_index, segment_index, label, rounds, rest_sec, rest_between_rounds_sec, exercises);
            }
        }
        dialog.close();
    }));
    
    dialog.present();
}

fn show_edit_superset_exercise_dialog(
    index: usize,
    current: SupersetExerciseData,
    exercises_model: StringList,
    exercises_data: std::rc::Rc<std::cell::RefCell<Vec<SupersetExerciseData>>>,
) {
    let dialog = Dialog::with_buttons(
        Some("Edit Superset Exercise"),
        crate::ui::util::parent_for_dialog().as_ref(),
        DialogFlags::MODAL,
        &[("Cancel", ResponseType::Cancel), ("Save", ResponseType::Accept)]
    );

    let content = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .margin_start(20)
        .margin_end(20)
        .margin_top(20)
        .margin_bottom(20)
        .spacing(12)
        .build();

    // Exercise Search Widget (optional, to help pick names)
    let search_widget = ExerciseSearchWidget::new();
    if let Err(e) = search_widget.set_database_path("/home/jawaad/weightlifting-desktop/exercises.db") {
        println!("Failed to connect to exercise database: {}", e);
    }
    let search_expander = gtk4::Expander::builder()
        .label("🔍 Search Exercise Database")
        .child(&search_widget.container)
        .expanded(false)
        .build();
    content.append(&search_expander);

    // Fields populated from current
    let ex_label = Label::new(Some("Exercise Code:"));
    let ex_entry = Entry::new();
    ex_entry.set_text(&current.ex_code);
    let name_label = Label::new(Some("Exercise Name:"));
    let name_entry = Entry::new();
    name_entry.set_text(&current.ex_name);
    content.append(&ex_label);
    content.append(&ex_entry);
    content.append(&name_label);
    content.append(&name_entry);

    let sets_label = Label::new(Some("Sets:"));
    let sets_entry = SpinButton::with_range(1.0, 20.0, 1.0);
    sets_entry.set_value(current.sets as f64);
    content.append(&sets_label);
    content.append(&sets_entry);

    let reps_label = Label::new(Some("Reps (min-max):"));
    let reps_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .build();
    let min_reps = SpinButton::with_range(1.0, 50.0, 1.0);
    min_reps.set_value(current.reps_min as f64);
    let max_reps = SpinButton::with_range(1.0, 50.0, 1.0);
    max_reps.set_value(current.reps_max as f64);
    reps_box.append(&min_reps);
    reps_box.append(&Label::new(Some("-")));
    reps_box.append(&max_reps);
    content.append(&reps_label);
    content.append(&reps_box);

    let rpe_label = Label::new(Some("RPE (optional):"));
    let rpe_entry = SpinButton::with_range(1.0, 10.0, 0.5);
    rpe_entry.set_value(current.rpe.unwrap_or(8.0));
    content.append(&rpe_label);
    content.append(&rpe_entry);

    let alt_group_label = Label::new(Some("Alternative Group (optional):"));
    let alt_group_entry = Entry::new();
    if let Some(ag) = current.alt_group.clone() { alt_group_entry.set_text(&ag); }
    content.append(&alt_group_label);
    content.append(&alt_group_entry);

    // Hook search selections to inputs
    let ex_entry_clone = ex_entry.clone();
    let name_entry_clone = name_entry.clone();
    search_widget.connect_row_activated(move |result| {
        ex_entry_clone.set_text(&result.code);
        name_entry_clone.set_text(&result.name);
    });
    let ex_entry_clone2 = ex_entry.clone();
    let name_entry_clone2 = name_entry.clone();
    search_widget.connect_row_selected(move |result| {
        ex_entry_clone2.set_text(&result.code);
        name_entry_clone2.set_text(&result.name);
    });

    dialog.content_area().append(&content);

    dialog.connect_response(clone!(@strong exercises_model, @strong exercises_data, @strong ex_entry, @strong name_entry, @strong sets_entry, @strong min_reps, @strong max_reps, @strong rpe_entry, @strong alt_group_entry => move |dialog, response| {
        if response == ResponseType::Accept {
            let mut vec = exercises_data.borrow_mut();
            if index < vec.len() {
                let rpe_val = rpe_entry.value();
                vec[index] = SupersetExerciseData {
                    ex_code: ex_entry.text().to_string(),
                    ex_name: name_entry.text().to_string(),
                    sets: sets_entry.value() as u32,
                    reps_min: min_reps.value() as u32,
                    reps_max: max_reps.value() as u32,
                    rpe: if rpe_val >= 6.0 { Some(rpe_val) } else { None },
                    alt_group: if alt_group_entry.text().trim().is_empty() { None } else { Some(alt_group_entry.text().to_string()) },
                };
                drop(vec);
                rebuild_exercise_model(&exercises_model, &exercises_data, format_superset_exercise_display);
            }
        }
        dialog.close();
    }));

    dialog.present();
}

fn show_add_superset_exercise_dialog(state: Arc<Mutex<AppState>>, exercises_model: StringList, exercises_data: std::rc::Rc<std::cell::RefCell<Vec<SupersetExerciseData>>>) {
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
    
    // Exercise Search Widget (search-first)
    let search_widget = ExerciseSearchWidget::new();
    if let Err(e) = search_widget.set_database_path("/home/jawaad/weightlifting-desktop/exercises.db") {
        println!("Failed to connect to exercise database: {}", e);
    }
    let search_expander = gtk4::Expander::builder()
        .label("🔍 Search Exercise Database")
        .child(&search_widget.container)
        .expanded(true)
        .build();
    content.append(&search_expander);

    // Group Search Widget (optional) - Note: supersets have per-item alt_groups, so we pass None here
    // Users can set alt_group per exercise item in the manual entry fields
    let group_search_widget = GroupSearchWidget::new(state.clone(), None);
    let group_search_expander = gtk4::Expander::builder()
        .label("🔗 Select Alternative Group (right-click to edit)")
        .child(&group_search_widget.container)
        .expanded(false)
        .build();

    // Enable right-click to edit groups
    group_search_widget.enable_right_click_edit();

    content.append(&group_search_expander);

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
    let ex_entry = Entry::new();
    ex_entry.set_text("BENCH.BB.FLAT");
    let name_label = Label::new(Some("Exercise Name:"));
    let name_entry = Entry::new();
    name_entry.set_text("Bench Press");
    manual_box.append(&ex_label);
    manual_box.append(&ex_entry);
    manual_box.append(&name_label);
    manual_box.append(&name_entry);
    manual_expander.set_child(Some(&manual_box));
    content.append(&manual_expander);
    
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
    
    content.append(&sets_label);
    content.append(&sets_entry);
    content.append(&reps_label);
    content.append(&reps_box);
    content.append(&rpe_label);
    content.append(&rpe_entry);

    // Alternative Group (text, filled via group search)
    let alt_group_label = Label::new(Some("Alternative Group (optional):"));
    let alt_group_entry = Entry::new();
    alt_group_entry.set_placeholder_text(Some("e.g., GROUP_VERTICAL_PULL_2GRIP"));
    content.append(&alt_group_label);
    content.append(&alt_group_entry);

    // Connect widgets
    let ex_entry_clone = ex_entry.clone();
    let name_entry_clone = name_entry.clone();
    let search_widget_clone = search_widget.clone();
    search_widget.connect_row_activated(move |result| {
        ex_entry_clone.set_text(&result.code);
        name_entry_clone.set_text(&result.name);
        search_widget_clone.set_selected_exercise(result);
    });
    let ex_entry_clone2 = ex_entry.clone();
    let name_entry_clone2 = name_entry.clone();
    search_widget.connect_row_selected(move |result| {
        ex_entry_clone2.set_text(&result.code);
        name_entry_clone2.set_text(&result.name);
    });
    let alt_group_entry_clone = alt_group_entry.clone();
    group_search_widget.connect_row_activated(move |result| {
        alt_group_entry_clone.set_text(&result.name);
    });

    dialog.content_area().append(&content);
    
    // Keyboard: Enter to accept, focus search on show
    let dialog_clone = dialog.clone();
    let search_widget_for_keys = search_widget.clone();
    let ex_entry_for_keys = ex_entry.clone();
    let name_entry_for_keys = name_entry.clone();
    let key_controller = gtk4::EventControllerKey::new();
    key_controller.connect_key_pressed(move |_, key, _, modifiers| {
        match key {
            gtk4::gdk::Key::Return | gtk4::gdk::Key::KP_Enter => {
                if modifiers.contains(gtk4::gdk::ModifierType::CONTROL_MASK) {
                    dialog_clone.response(ResponseType::Accept);
                    return glib::Propagation::Stop;
                }
                if let Some(sel) = search_widget_for_keys.selected_exercise() {
                    ex_entry_for_keys.set_text(&sel.code);
                    name_entry_for_keys.set_text(&sel.name);
                    dialog_clone.response(ResponseType::Accept);
                    return glib::Propagation::Stop;
                }
                glib::Propagation::Proceed
            }
            gtk4::gdk::Key::Escape => { dialog_clone.response(ResponseType::Cancel); glib::Propagation::Stop }
            _ => glib::Propagation::Proceed,
        }
    });
    dialog.add_controller(key_controller);
    dialog.connect_show(move |_| {
        if let Some(search_container) = search_widget.container.first_child() {
            if let Some(entry) = search_container.downcast_ref::<Entry>() { entry.grab_focus(); }
        }
    });

    dialog.connect_response(clone!(@strong exercises_model, @strong exercises_data, @strong ex_entry, @strong name_entry, @strong sets_entry, @strong min_reps, @strong max_reps, @strong rpe_entry, @strong alt_group_entry => move |dialog, response| {
        if response == ResponseType::Accept {
            let ex_code = ex_entry.text().to_string();
            let ex_name = name_entry.text().to_string();
            let sets = sets_entry.value() as u32;
            let reps_min = min_reps.value() as u32;
            let reps_max = max_reps.value() as u32;
            let rpe = if rpe_entry.value() >= 6.0 { Some(rpe_entry.value()) } else { None };
            let alt_group = if alt_group_entry.text().trim().is_empty() { None } else { Some(alt_group_entry.text().to_string()) };

            let exercise_data = SupersetExerciseData {
                ex_code: ex_code.clone(),
                ex_name: ex_name.clone(),
                sets,
                reps_min,
                reps_max,
                rpe,
                alt_group,
            };
            
            exercises_data.borrow_mut().push(exercise_data.clone());
            
            let display_text = format_superset_exercise_display(&exercise_data);
            exercises_model.append(&display_text);
        }
        dialog.close();
    }));
    
    dialog.present();
}

/// Format superset exercise data for display in the list
fn format_superset_exercise_display(exercise: &SupersetExerciseData) -> String {
    if exercise.reps_min == exercise.reps_max {
        format!("{}: {}x{} @ RPE {:?}", exercise.ex_name, exercise.sets, exercise.reps_min, exercise.rpe.unwrap_or(0.0))
    } else {
        format!("{}: {}x{}-{} @ RPE {:?}", exercise.ex_name, exercise.sets, exercise.reps_min, exercise.reps_max, exercise.rpe.unwrap_or(0.0))
    }
}
