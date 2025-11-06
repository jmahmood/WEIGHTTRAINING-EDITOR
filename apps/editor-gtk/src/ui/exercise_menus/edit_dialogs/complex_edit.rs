// Complex edit dialog functionality

use crate::state::AppState;
use crate::operations::plan_ops::update_complex_in_plan;
use super::dialog_components::create_exercise_search_section;
use glib::clone;
use gtk4::prelude::*;
use gtk4::{
    Orientation, Label, Box as GtkBox, SpinButton,
    Dialog, DialogFlags, ResponseType, ComboBoxText
};
use std::sync::{Arc, Mutex};
use weightlifting_core::RepsOrRange;

pub fn show_edit_complex_dialog(state: Arc<Mutex<AppState>>, complex: weightlifting_core::ComplexSegment, day_index: usize, segment_index: usize) {
    let dialog = Dialog::with_buttons(
        Some("Edit Complex"),
        crate::ui::util::parent_for_dialog().as_ref(),
        DialogFlags::MODAL,
        &[("Cancel", ResponseType::Cancel), ("Update", ResponseType::Accept)]
    );
    
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
    
    // Anchor load configuration - populate with existing values
    let anchor_label = Label::new(Some("Anchor Load Configuration:"));
    anchor_label.set_css_classes(&["heading"]);
    content.append(&anchor_label);
    
    let mode_label = Label::new(Some("Load Mode:"));
    let mode_combo = ComboBoxText::new();
    mode_combo.append_text("pct_1rm");
    mode_combo.append_text("fixed_kg");
    
    // Set current mode
    if complex.anchor_load.mode == "pct_1rm" {
        mode_combo.set_active(Some(0));
    } else {
        mode_combo.set_active(Some(1));
    }
    
    // Anchor Exercise Search Widget
    let (anchor_search_expander, anchor_ex_entry, _) = create_exercise_search_section();
    content.append(&anchor_search_expander);

    let anchor_manual_label = Label::new(Some("Or enter anchor exercise manually:"));
    anchor_manual_label.set_css_classes(&["dim-label"]);
    content.append(&anchor_manual_label);

    let anchor_ex_label = Label::new(Some("Anchor Exercise:"));
    anchor_ex_entry.set_text(&complex.anchor_load.ex.unwrap_or_else(|| "SQUAT.BB.BACK".to_string()));
    
    let anchor_pct_label = Label::new(Some("Percentage:"));
    let anchor_pct_entry = SpinButton::with_range(50.0, 120.0, 5.0);
    anchor_pct_entry.set_value(complex.anchor_load.pct.unwrap_or(85.0));
    
    let anchor_kg_label = Label::new(Some("Fixed Load (kg):"));
    let anchor_kg_entry = SpinButton::with_range(20.0, 500.0, 5.0);
    anchor_kg_entry.set_value(complex.anchor_load.kg.unwrap_or(100.0));
    
    // Set initial sensitivity based on current mode
    let is_pct_mode = complex.anchor_load.mode == "pct_1rm";
    anchor_pct_entry.set_sensitive(is_pct_mode);
    anchor_kg_entry.set_sensitive(!is_pct_mode);
    
    // Toggle between percentage and fixed kg mode
    mode_combo.connect_changed(clone!(@strong anchor_pct_entry, @strong anchor_kg_entry => move |combo| {
        if let Some(text) = combo.active_text() {
            let is_pct_mode = text.as_str() == "pct_1rm";
            anchor_pct_entry.set_sensitive(is_pct_mode);
            anchor_kg_entry.set_sensitive(!is_pct_mode);
        }
    }));
    
    // Sets configuration
    let sets_label = Label::new(Some("Sets:"));
    let sets_entry = SpinButton::with_range(1.0, 10.0, 1.0);
    sets_entry.set_value(complex.sets as f64);
    
    // Rest configuration
    let rest_label = Label::new(Some("Rest between sets (sec):"));
    let rest_entry = SpinButton::with_range(30.0, 600.0, 15.0);
    rest_entry.set_value(complex.rest_sec as f64);
    
    // Sequence display (simplified - show first exercise)
    let seq_label = Label::new(Some("Complex Sequence:"));
    seq_label.set_css_classes(&["heading"]);
    content.append(&seq_label);
    
    let seq_display = if let Some(first_item) = complex.sequence.first() {
        format!("First exercise: {} ({} reps)", first_item.ex, 
                match &first_item.reps {
                    RepsOrRange::Range(r) => {
                        if r.min == r.max { r.min.to_string() } else { format!("{}-{}", r.min, r.max) }
                    }
                })
    } else {
        "No sequence items".to_string()
    };
    
    let seq_info = Label::new(Some(&seq_display));
    content.append(&seq_info);
    
    let note_label = Label::new(Some("Note: Full sequence editing not yet implemented. You can only edit anchor load and set parameters."));
    note_label.set_css_classes(&["dim-label"]);
    content.append(&note_label);
    
    // Build the UI
    content.append(&mode_label);
    content.append(&mode_combo);
    content.append(&anchor_ex_label);
    content.append(&anchor_ex_entry);
    content.append(&anchor_pct_label);
    content.append(&anchor_pct_entry);
    content.append(&anchor_kg_label);
    content.append(&anchor_kg_entry);
    content.append(&sets_label);
    content.append(&sets_entry);
    content.append(&rest_label);
    content.append(&rest_entry);
    
    dialog.content_area().append(&content);
    
    dialog.connect_response(clone!(@strong state, @strong mode_combo, @strong anchor_ex_entry, @strong anchor_pct_entry, @strong anchor_kg_entry, @strong sets_entry, @strong rest_entry => move |dialog, response| {
        if response == ResponseType::Accept {
            let mode = mode_combo.active_text().unwrap_or_default().to_string();
            let anchor_ex = anchor_ex_entry.text().to_string();
            let sets = sets_entry.value() as u32;
            let rest_sec = rest_entry.value() as u32;
            
            let (pct, kg) = if mode == "pct_1rm" {
                (Some(anchor_pct_entry.value()), None)
            } else {
                (None, Some(anchor_kg_entry.value()))
            };
            
            update_complex_in_plan(
                state.clone(), 
                day_index,
                segment_index,
                mode, 
                anchor_ex, 
                pct, 
                kg, 
                sets, 
                rest_sec
            );
        }
        dialog.close();
    }));
    
    dialog.present();
}
