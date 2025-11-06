// Complex dialog extracted from complex_dialogs.rs

use crate::state::AppState;
use super::components::create_exercise_search_section_complex;
use glib::clone;
use gtk4::prelude::*;
use gtk4::prelude::EditableExt;
use gtk4::{Orientation, Label, ComboBoxText, Box as GtkBox, Entry, SpinButton, Dialog, DialogFlags, ResponseType, Expander};
use std::sync::{Arc, Mutex};

use crate::operations::plan_ops::add_complex_to_plan;

pub fn show_add_complex_dialog(state: Arc<Mutex<AppState>>) {
    let dialog = Dialog::with_buttons(
        Some("Add Complex"),
        crate::ui::util::parent_for_dialog().as_ref(),
        DialogFlags::MODAL,
        &[("Cancel", ResponseType::Cancel), ("Add", ResponseType::Accept)]
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

    // Anchor load configuration
    let anchor_label = Label::new(Some("Anchor Load Configuration:"));
    anchor_label.set_css_classes(&["heading"]);
    content.append(&anchor_label);

    let mode_label = Label::new(Some("Load Mode:"));
    let mode_combo = ComboBoxText::new();
    mode_combo.append_text("pct_1rm");
    mode_combo.append_text("fixed_kg");
    mode_combo.set_active(Some(0));

    // Anchor Exercise Search Widget (Default)
    let (anchor_search_expander, anchor_ex_entry, _, _anchor_search_widget) = create_exercise_search_section_complex();
    content.append(&anchor_search_expander);

    // Manual Input Fields (Advanced option)
    let anchor_manual_expander = Expander::builder()
        .label("⚙️ Manual Entry (Advanced)")
        .expanded(false)
        .build();

    let anchor_manual_content = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .build();

    let anchor_ex_label = Label::new(Some("Anchor Exercise:"));
    anchor_ex_entry.set_text("SQUAT.BB.BACK");

    let anchor_pct_label = Label::new(Some("Percentage:"));
    let anchor_pct_entry = SpinButton::with_range(50.0, 120.0, 5.0);
    anchor_pct_entry.set_value(85.0);

    let anchor_kg_label = Label::new(Some("Fixed Load (kg):"));
    let anchor_kg_entry = SpinButton::with_range(20.0, 500.0, 5.0);
    anchor_kg_entry.set_value(100.0);
    anchor_kg_entry.set_sensitive(false);

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
    sets_entry.set_value(3.0);

    // Sequence configuration (simplified - single exercise for now)
    let seq_label = Label::new(Some("Complex Sequence:"));
    seq_label.set_css_classes(&["heading"]);
    content.append(&seq_label);

    let seq_ex_label = Label::new(Some("Exercise:"));
    let seq_ex_entry = Entry::builder().text("DEADLIFT.BB.CON").build();

    let seq_reps_label = Label::new(Some("Reps:"));
    let seq_reps_entry = SpinButton::with_range(1.0, 20.0, 1.0);
    seq_reps_entry.set_value(3.0);

    // Build the UI
    anchor_manual_content.append(&anchor_ex_label);
    anchor_manual_content.append(&anchor_ex_entry);

    anchor_manual_expander.set_child(Some(&anchor_manual_content));
    content.append(&anchor_manual_expander);

    content.append(&mode_label);
    content.append(&mode_combo);
    content.append(&anchor_pct_label);
    content.append(&anchor_pct_entry);
    content.append(&anchor_kg_label);
    content.append(&anchor_kg_entry);
    content.append(&sets_label);
    content.append(&sets_entry);
    content.append(&seq_ex_label);
    content.append(&seq_ex_entry);
    content.append(&seq_reps_label);
    content.append(&seq_reps_entry);

    dialog.content_area().append(&content);

    dialog.connect_response(clone!(@strong state, @strong mode_combo, @strong anchor_ex_entry, @strong anchor_pct_entry, @strong anchor_kg_entry, @strong sets_entry, @strong seq_ex_entry, @strong seq_reps_entry => move |dialog, response| {
        if response == ResponseType::Accept {
            let mode = mode_combo.active_text().unwrap_or_default().to_string();
            let anchor_ex = anchor_ex_entry.text().to_string();
            let sets = sets_entry.value() as u32;
            let seq_ex = seq_ex_entry.text().to_string();
            let seq_reps = seq_reps_entry.value() as u32;

            let (pct, kg) = if mode == "pct_1rm" {
                (Some(anchor_pct_entry.value()), None)
            } else {
                (None, Some(anchor_kg_entry.value()))
            };

            add_complex_to_plan(
                state.clone(), 
                mode, 
                anchor_ex, 
                pct, 
                kg, 
                sets, 
                seq_ex, 
                seq_reps
            );
        }
        dialog.close();
    }));

    dialog.present();
}
