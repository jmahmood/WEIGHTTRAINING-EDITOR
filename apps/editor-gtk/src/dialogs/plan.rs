use crate::state::AppState;
use crate::canvas::update_canvas_content;
use gtk4::{Dialog, DialogFlags, ResponseType, Box as GtkBox, Orientation, Label, Entry, ComboBoxText};
use gtk4::prelude::*;
use glib::clone;
use std::sync::{Arc, Mutex};
use weightlifting_core::Unit;

pub fn show_edit_plan_metadata_dialog(state: Arc<Mutex<AppState>>) {
    let (name, author, source_url, license_note, unit) = {
        let app_state = state.lock().unwrap();
        if let Some(plan) = &app_state.current_plan {
            (
                plan.name.clone(),
                plan.author.clone().unwrap_or_default(),
                plan.source_url.clone().unwrap_or_default(),
                plan.license_note.clone().unwrap_or_default(),
                match plan.unit { Unit::Kg => "kg", Unit::Lb => "lb", Unit::Bw => "bw" }.to_string(),
            )
        } else {
            ("".to_string(), "".to_string(), "".to_string(), "".to_string(), "kg".to_string())
        }
    };
    
    let dialog = Dialog::with_buttons(
        Some("Edit Plan Metadata"),
        crate::ui::util::parent_for_dialog().as_ref(),
        DialogFlags::MODAL,
        &[("Cancel", ResponseType::Cancel), ("Save", ResponseType::Accept)]
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
    
    let name_label = Label::new(Some("Name:"));
    let name_entry = Entry::builder().text(&name).build();
    content.append(&name_label);
    content.append(&name_entry);
    
    let author_label = Label::new(Some("Author:"));
    let author_entry = Entry::builder().text(&author).build();
    content.append(&author_label);
    content.append(&author_entry);
    
    let source_label = Label::new(Some("Source URL:"));
    let source_entry = Entry::builder().text(&source_url).build();
    content.append(&source_label);
    content.append(&source_entry);
    
    let license_label = Label::new(Some("License Note:"));
    let license_entry = Entry::builder().text(&license_note).build();
    content.append(&license_label);
    content.append(&license_entry);
    
    let unit_label = Label::new(Some("Unit:"));
    let unit_combo = ComboBoxText::new();
    unit_combo.append_text("kg");
    unit_combo.append_text("lb");
    unit_combo.append_text("bw");
    unit_combo.set_active_id(Some(&unit));
    content.append(&unit_label);
    content.append(&unit_combo);
    
    dialog.content_area().append(&content);
    // Ctrl+Enter triggers Save
    crate::ui::util::bind_ctrl_enter_to_accept(&dialog);
    
    dialog.connect_response(clone!(@strong state => move |dialog, response| {
        if response == ResponseType::Accept {
            let mut app_state = state.lock().unwrap();
            if let Some(plan) = &mut app_state.current_plan {
                let new_name = name_entry.text().to_string();
                let new_author = author_entry.text().to_string();
                let new_source = source_entry.text().to_string();
                let new_license = license_entry.text().to_string();
                let new_unit = unit_combo.active_text().map(|s| s.to_string()).unwrap_or("kg".to_string());
                
                plan.name = if new_name.trim().is_empty() { plan.name.clone() } else { new_name };
                plan.author = if new_author.trim().is_empty() { None } else { Some(new_author) };
                plan.source_url = if new_source.trim().is_empty() { None } else { Some(new_source) };
                plan.license_note = if new_license.trim().is_empty() { None } else { Some(new_license) };
                plan.unit = match new_unit.as_str() { "kg" => Unit::Kg, "lb" => Unit::Lb, "bw" => Unit::Bw, _ => Unit::Kg };
                
                app_state.mark_modified();
            }
            drop(app_state);
            update_canvas_content(state.clone());
        }
        dialog.close();
    }));
    
    dialog.present();
}
