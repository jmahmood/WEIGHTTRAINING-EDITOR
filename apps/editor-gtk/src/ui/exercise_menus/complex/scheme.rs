// Scheme dialog extracted from complex_dialogs.rs

use crate::state::AppState;
use super::components::create_exercise_search_section_complex;
use glib::clone;
use gtk4::prelude::*;
use gtk4::prelude::EditableExt;
use gtk4::{Orientation, Box as GtkBox, SpinButton, Dialog, DialogFlags, ResponseType, Expander};
use std::sync::{Arc, Mutex};

use crate::operations::plan_ops::add_scheme_to_plan;

pub fn show_add_scheme_dialog(state: Arc<Mutex<AppState>>) {
    use gtk4::Label as GtkLabel;

    let dialog = Dialog::with_buttons(
        Some("Add Scheme"),
        crate::ui::util::parent_for_dialog().as_ref(),
        DialogFlags::MODAL,
        &[("Cancel", ResponseType::Cancel), ("Add", ResponseType::Accept)]
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

    // Exercise Search Widget (Default)
    let (search_expander, ex_entry, label_entry, _search_widget) = create_exercise_search_section_complex();
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

    let ex_label = GtkLabel::new(Some("Exercise Code:"));
    ex_entry.set_text("SQ.BB.BACK");

    let label_label = GtkLabel::new(Some("Exercise Name:"));
    label_entry.set_text("Back Squat");

    // Scheme sets configuration
    let scheme_label = GtkLabel::new(Some("Scheme Configuration"));
    scheme_label.set_css_classes(&["heading"]);

    // Top set configuration
    let top_set_label = GtkLabel::new(Some("Top Set:"));
    top_set_label.set_css_classes(&["caption"]);

    let top_reps_label = GtkLabel::new(Some("Top Set Reps:"));
    let top_reps_entry = SpinButton::with_range(1.0, 10.0, 1.0);
    top_reps_entry.set_value(1.0);

    let top_rpe_label = GtkLabel::new(Some("Top Set RPE:"));
    let top_rpe_entry = SpinButton::with_range(1.0, 10.0, 0.5);
    top_rpe_entry.set_value(8.0);

    let top_rest_label = GtkLabel::new(Some("Top Set Rest (sec):"));
    let top_rest_entry = SpinButton::with_range(30.0, 300.0, 15.0);
    top_rest_entry.set_value(180.0);

    // Backoff sets configuration
    let backoff_label = GtkLabel::new(Some("Backoff Sets:"));
    backoff_label.set_css_classes(&["caption"]);

    let backoff_sets_label = GtkLabel::new(Some("Number of Backoff Sets:"));
    let backoff_sets_entry = SpinButton::with_range(1.0, 10.0, 1.0);
    backoff_sets_entry.set_value(5.0);

    let backoff_reps_label = GtkLabel::new(Some("Backoff Reps:"));
    let backoff_reps_entry = SpinButton::with_range(1.0, 20.0, 1.0);
    backoff_reps_entry.set_value(5.0);

    let backoff_percent_label = GtkLabel::new(Some("Backoff Percentage:"));
    let backoff_percent_entry = SpinButton::with_range(0.5, 1.0, 0.05);
    backoff_percent_entry.set_value(0.8);

    let backoff_rest_label = GtkLabel::new(Some("Backoff Rest (sec):"));
    let backoff_rest_entry = SpinButton::with_range(30.0, 300.0, 15.0);
    backoff_rest_entry.set_value(150.0);

    // Build the UI
    manual_content.append(&ex_label);
    manual_content.append(&ex_entry);
    manual_content.append(&label_label);
    manual_content.append(&label_entry);

    manual_expander.set_child(Some(&manual_content));
    content.append(&manual_expander);

    content.append(&scheme_label);
    content.append(&top_set_label);
    content.append(&top_reps_label);
    content.append(&top_reps_entry);
    content.append(&top_rpe_label);
    content.append(&top_rpe_entry);
    content.append(&top_rest_label);
    content.append(&top_rest_entry);
    content.append(&backoff_label);
    content.append(&backoff_sets_label);
    content.append(&backoff_sets_entry);
    content.append(&backoff_reps_label);
    content.append(&backoff_reps_entry);
    content.append(&backoff_percent_label);
    content.append(&backoff_percent_entry);
    content.append(&backoff_rest_label);
    content.append(&backoff_rest_entry);

    dialog.content_area().append(&content);

    dialog.connect_response(clone!(@strong state, @strong ex_entry, @strong label_entry, @strong top_reps_entry, @strong top_rpe_entry, @strong top_rest_entry, @strong backoff_sets_entry, @strong backoff_reps_entry, @strong backoff_percent_entry, @strong backoff_rest_entry => move |dialog, response| {
        if response == ResponseType::Accept {
            let ex_code = ex_entry.text().to_string();
            let ex_label = label_entry.text().to_string();
            let top_reps = top_reps_entry.value() as u32;
            let top_rpe = top_rpe_entry.value();
            let top_rest = top_rest_entry.value() as u32;
            let backoff_sets = backoff_sets_entry.value() as u32;
            let backoff_reps = backoff_reps_entry.value() as u32;
            let backoff_percent = backoff_percent_entry.value();
            let backoff_rest = backoff_rest_entry.value() as u32;

            add_scheme_to_plan(
                state.clone(), 
                ex_code, 
                ex_label, 
                top_reps, 
                top_rpe, 
                top_rest, 
                backoff_sets, 
                backoff_reps, 
                backoff_percent, 
                backoff_rest
            );
        }
        dialog.close();
    }));

    dialog.present();
}
