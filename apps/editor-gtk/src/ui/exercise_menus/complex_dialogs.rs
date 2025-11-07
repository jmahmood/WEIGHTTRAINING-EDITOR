// Group dialogs (Choose/Rotate/Optional/Superset) and helpers

use crate::operations::plan_ops::add_group_choose_to_plan;
use crate::state::AppState;
use crate::ui::widgets::ExerciseSearchWidget;
use glib::clone;
use gtk4::prelude::*;
use gtk4::{
    Box as GtkBox, Button, Dialog, DialogFlags, DropDown, Expander, Label, ListBox, ListBoxRow,
    Orientation, ResponseType, ScrolledWindow, SelectionMode, StringList,
};
use std::sync::{Arc, Mutex};

pub fn show_add_group_choose_dialog(state: Arc<Mutex<AppState>>) {
    let dialog = Dialog::with_buttons(
        Some("Add Group (Choose)"),
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

    // Pick count
    let pick_label = Label::new(Some("How many exercises to pick:"));
    let pick_entry = gtk4::SpinButton::with_range(1.0, 10.0, 1.0);
    pick_entry.set_value(1.0);
    content.append(&pick_label);
    content.append(&pick_entry);

    // Rotation type
    let rotation_label = Label::new(Some("Rotation type:"));
    let rotation_options = StringList::new(&["none", "weekly", "session", "random"]);
    let rotation_dropdown = DropDown::new(Some(rotation_options), None::<gtk4::Expression>);
    rotation_dropdown.set_selected(0);
    content.append(&rotation_label);
    content.append(&rotation_dropdown);

    // Exercise list
    let exercises_label = Label::new(Some("Exercises to choose from:"));
    content.append(&exercises_label);
    let exercises_scrolled = ScrolledWindow::builder()
        .hexpand(true)
        .vexpand(true)
        .min_content_height(300)
        .build();
    let exercises_list = ListBox::new();
    exercises_list.set_selection_mode(SelectionMode::None);
    exercises_scrolled.set_child(Some(&exercises_list));
    content.append(&exercises_scrolled);

    // Exercise search
    let search_widget = ExerciseSearchWidget::new();
    if let Err(e) =
        search_widget.set_database_path("/home/jawaad/weightlifting-desktop/exercises.db")
    {
        println!("Failed to connect to exercise database: {}", e);
    }
    let search_expander = Expander::builder()
        .label("Add Exercise")
        .child(&search_widget.container)
        .expanded(false)
        .build();
    content.append(&search_expander);

    search_widget.connect_row_activated(clone!(@strong exercises_list => move |result| {
        add_exercise_to_group_list(&exercises_list, &result.code, &result.name);
    }));

    dialog.content_area().append(&content);
    dialog.connect_response(clone!(@strong state => move |dialog, response| {
        if response == ResponseType::Accept {
            let pick = pick_entry.value() as u32;
            let rotation_index = rotation_dropdown.selected();
            let rotation = match rotation_index {
                1 => Some("weekly".to_string()),
                2 => Some("session".to_string()),
                3 => Some("random".to_string()),
                _ => None,
            };
            let exercises = collect_exercises_from_list(&exercises_list);
            if exercises.is_empty() {
                return;
            }
            add_group_choose_to_plan(state.clone(), pick, rotation, exercises);
        }
        dialog.close();
    }));
    dialog.present();
}

pub fn show_add_group_rotate_dialog(_state: Arc<Mutex<AppState>>) {
    println!("Group (Rotate) dialog - not yet implemented in Sprint 2");
}

pub fn show_add_group_optional_dialog(_state: Arc<Mutex<AppState>>) {
    println!("Group (Optional) dialog - not yet implemented in Sprint 2");
}

pub fn show_add_group_superset_dialog(_state: Arc<Mutex<AppState>>) {
    println!("Group (Superset) dialog - not yet implemented in Sprint 2");
}

fn add_exercise_to_group_list(exercises_list: &ListBox, code: &str, name: &str) {
    // Prevent duplicates
    let mut child = exercises_list.first_child();
    while let Some(row) = child {
        if let Some(list_row) = row.downcast_ref::<ListBoxRow>() {
            if let Some(box_) = list_row.child() {
                if let Some(box_) = box_.downcast_ref::<GtkBox>() {
                    if let Some(first_child) = box_.first_child() {
                        if let Some(label) = first_child.downcast_ref::<Label>() {
                            if label.text() == code {
                                return;
                            }
                        }
                    }
                }
            }
        }
        child = row.next_sibling();
    }

    let row = ListBoxRow::new();
    let box_ = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .margin_start(10)
        .margin_end(10)
        .margin_top(5)
        .margin_bottom(5)
        .spacing(10)
        .build();
    let code_label = Label::new(Some(code));
    code_label.set_css_classes(&["monospace"]);
    code_label.set_width_chars(15);
    let name_label = Label::new(Some(name));
    name_label.set_hexpand(true);
    let remove_btn = Button::from_icon_name("user-trash-symbolic");
    remove_btn.set_css_classes(&["flat"]);
    box_.append(&code_label);
    box_.append(&name_label);
    box_.append(&remove_btn);
    row.set_child(Some(&box_));

    remove_btn.connect_clicked(clone!(@strong row, @strong exercises_list => move |_| {
        exercises_list.remove(&row);
    }));
    exercises_list.append(&row);
}

fn collect_exercises_from_list(exercises_list: &ListBox) -> Vec<(String, String)> {
    let mut exercises = Vec::new();
    let mut child = exercises_list.first_child();
    while let Some(row) = child {
        if let Some(list_row) = row.downcast_ref::<ListBoxRow>() {
            if let Some(box_) = list_row.child() {
                if let Some(box_) = box_.downcast_ref::<GtkBox>() {
                    if let Some(code_label) =
                        box_.first_child().and_then(|c| c.downcast::<Label>().ok())
                    {
                        if let Some(name_label) = code_label
                            .next_sibling()
                            .and_then(|c| c.downcast::<Label>().ok())
                        {
                            exercises.push((
                                code_label.text().to_string(),
                                name_label.text().to_string(),
                            ));
                        }
                    }
                }
            }
        }
        child = row.next_sibling();
    }
    exercises
}
