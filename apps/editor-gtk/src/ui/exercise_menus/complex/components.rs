use crate::ui::widgets::ExerciseSearchWidget;
use gtk4::{Expander, Entry};
use gtk4::prelude::EditableExt;

// Shared helper to build the exercise search section used by complex dialogs
pub fn create_exercise_search_section_complex() -> (Expander, Entry, Entry, ExerciseSearchWidget) {
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

    (search_expander, ex_entry, label_entry, search_widget)
}
