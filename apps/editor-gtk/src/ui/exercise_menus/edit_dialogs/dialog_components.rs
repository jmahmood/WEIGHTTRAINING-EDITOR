// Common UI components and utilities shared across edit dialogs

use crate::ui::widgets::ExerciseSearchWidget;
use gtk4::prelude::*;
use gtk4::{
    Orientation, Label, StringList, ListView, ScrolledWindow, NoSelection, 
    SignalListItemFactory, StringObject, Box as GtkBox, Entry, Button, Expander, SpinButton
};

/// Creates a reusable exercise search section with search widget and manual input fields
pub fn create_exercise_search_section() -> (Expander, Entry, Entry) {
    let search_widget = ExerciseSearchWidget::new();
    if let Err(e) = search_widget.set_database_path("/home/jawaad/weightlifting-desktop/exercises.db") {
        println!("Failed to connect to exercise database: {}", e);
    }
    
    let search_expander = Expander::builder()
        .label("🔍 Search Exercise Database")
        .child(&search_widget.container)
        .expanded(false)
        .build();
    
    // Manual input fields
    let ex_entry = Entry::new();
    let label_entry = Entry::new();
    
    // Connect search widget selection to populate manual fields
    let ex_entry_clone = ex_entry.clone();
    let label_entry_clone = label_entry.clone();
    let expander_on_select = search_expander.clone();
    search_widget.connect_row_activated(move |result| {
        ex_entry_clone.set_text(&result.code);
        label_entry_clone.set_text(&result.name);
        // Collapse the search dropdown immediately after selection
        expander_on_select.set_expanded(false);
    });
    
    (search_expander, ex_entry, label_entry)
}

/// Creates a standard exercise list view with scrolling
pub fn create_exercise_list_view(exercises_model: StringList) -> (ListView, ScrolledWindow, SignalListItemFactory) {
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

    let exercises_view = ListView::new(Some(selection_model.clone()), Some(factory.clone()));

    let scrolled_exercises = ScrolledWindow::builder()
        .child(&exercises_view)
        .min_content_height(150)
        .build();

    (exercises_view, scrolled_exercises, factory)
}

/// Creates standard exercise management buttons
pub fn create_exercise_buttons() -> (GtkBox, Button, Button, Button, Button) {
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
    
    (exercise_buttons_box, add_exercise_btn, remove_exercise_btn, move_up_btn, move_down_btn)
}

/// Generic function to remove the last exercise from a list
pub fn remove_last_exercise<T>(exercises_model: &StringList, exercises_data: &std::rc::Rc<std::cell::RefCell<Vec<T>>>) {
    let count = exercises_data.borrow().len();
    if count > 0 {
        exercises_model.remove(count as u32 - 1);
        exercises_data.borrow_mut().remove(count - 1);
    }
}

/// Generic function to rebuild exercise model display
pub fn rebuild_exercise_model<T, F>(exercises_model: &StringList, exercises_data: &std::rc::Rc<std::cell::RefCell<Vec<T>>>, format_fn: F)
where
    F: Fn(&T) -> String,
{
    exercises_model.splice(0, exercises_model.n_items(), &[] as &[&str]);
    let data = exercises_data.borrow();
    for exercise in data.iter() {
        let display_text = format_fn(exercise);
        exercises_model.append(&display_text);
    }
}

/// Generic function to move exercise up in list (swap with previous)
pub fn move_exercise_up<T, F>(
    exercises_model: &StringList, 
    exercises_data: &std::rc::Rc<std::cell::RefCell<Vec<T>>>,
    format_fn: F
)
where
    F: Fn(&T) -> String,
{
    let count = exercises_data.borrow().len();
    if count > 1 {
        let last_idx = count - 1;
        let prev_idx = count - 2;
        
        // Swap in data
        exercises_data.borrow_mut().swap(last_idx, prev_idx);
        // Rebuild model
        rebuild_exercise_model(exercises_model, exercises_data, format_fn);
    }
}

/// Generic function to move exercise down in list (swap with next)  
pub fn move_exercise_down<T, F>(
    exercises_model: &StringList,
    exercises_data: &std::rc::Rc<std::cell::RefCell<Vec<T>>>,
    format_fn: F
)
where
    F: Fn(&T) -> String,
{
    let count = exercises_data.borrow().len();
    if count > 1 {
        let second_last_idx = count - 2;
        let last_idx = count - 1;
        
        // Swap in data
        exercises_data.borrow_mut().swap(second_last_idx, last_idx);
        // Rebuild model
        rebuild_exercise_model(exercises_model, exercises_data, format_fn);
    }
}

/// Creates a reusable base segment section (exercise, label, alt_group)
pub fn create_base_segment_section(ex: &str, label: Option<&str>, alt_group: Option<&str>) -> (GtkBox, Entry, Entry, Entry) {
    let section = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .build();
    
    // Exercise code
    let ex_label = Label::new(Some("Exercise Code:"));
    let ex_entry = Entry::builder()
        .text(ex)
        .build();
    section.append(&ex_label);
    section.append(&ex_entry);
    
    // Exercise label
    let label_label = Label::new(Some("Exercise Name (optional):"));
    let label_entry = Entry::builder()
        .text(label.unwrap_or(""))
        .build();
    section.append(&label_label);
    section.append(&label_entry);
    
    // Alternative group
    let alt_label = Label::new(Some("Alternative Group (optional):"));
    let alt_entry = Entry::builder()
        .text(alt_group.unwrap_or(""))
        .build();
    section.append(&alt_label);
    section.append(&alt_entry);
    
    (section, ex_entry, label_entry, alt_entry)
}

/// Creates a sets/reps section for basic exercise parameters
#[allow(dead_code)]
pub fn create_sets_reps_section(sets: Option<u32>, min_reps: Option<u32>, max_reps: Option<u32>) -> (GtkBox, SpinButton, SpinButton, SpinButton) {
    let section = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .build();
    
    // Sets
    let sets_label = Label::new(Some("Sets:"));
    let sets_entry = SpinButton::with_range(1.0, 20.0, 1.0);
    sets_entry.set_value(sets.unwrap_or(3) as f64);
    section.append(&sets_label);
    section.append(&sets_entry);
    
    // Reps range
    let reps_label = Label::new(Some("Reps (min-max):"));
    let reps_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .build();
    
    let min_reps_entry = SpinButton::with_range(1.0, 50.0, 1.0);
    min_reps_entry.set_value(min_reps.unwrap_or(8) as f64);
    let max_reps_entry = SpinButton::with_range(1.0, 50.0, 1.0);
    max_reps_entry.set_value(max_reps.unwrap_or(10) as f64);
    
    reps_box.append(&min_reps_entry);
    reps_box.append(&Label::new(Some("-")));
    reps_box.append(&max_reps_entry);
    
    section.append(&reps_label);
    section.append(&reps_box);
    
    (section, sets_entry, min_reps_entry, max_reps_entry)
}

/// Creates training parameters section (RPE, rest)
#[allow(dead_code)]
pub fn create_training_params_section(rpe: Option<f64>, rest_sec: Option<u32>) -> (GtkBox, SpinButton, SpinButton) {
    let section = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .build();
    
    // RPE
    let rpe_label = Label::new(Some("RPE (optional):"));
    let rpe_entry = SpinButton::with_range(1.0, 10.0, 0.5);
    if let Some(rpe_val) = rpe {
        rpe_entry.set_value(rpe_val);
    } else {
        rpe_entry.set_value(8.0);
    }
    section.append(&rpe_label);
    section.append(&rpe_entry);
    
    // Rest
    let rest_label = Label::new(Some("Rest (seconds, optional):"));
    let rest_entry = SpinButton::with_range(0.0, 600.0, 15.0);
    if let Some(rest_val) = rest_sec {
        rest_entry.set_value(rest_val as f64);
    } else {
        rest_entry.set_value(90.0);
    }
    section.append(&rest_label);
    section.append(&rest_entry);
    
    (section, rpe_entry, rest_entry)
}
