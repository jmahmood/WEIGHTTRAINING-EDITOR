use crate::operations::segment::add_custom_exercise_to_plan;
use crate::state::AppState;
use crate::ui::widgets::{ExerciseSearchWidget, GroupSearchWidget};
use glib::clone;
use gtk4::prelude::*;
use gtk4::{
    Box as GtkBox, Dialog, DialogFlags, Entry, EventControllerKey, Expander, Label, Orientation,
    ResponseType, SpinButton,
};
use std::sync::{Arc, Mutex};

pub fn show_add_exercise_dialog(state: Arc<Mutex<AppState>>) {
    show_exercise_editor_dialog(
        state,
        ExerciseEditorContext::Create {
            day_index: None,
            initial: None,
        },
    );
}

#[allow(dead_code)]
pub fn show_add_exercise_dialog_with_day(state: Arc<Mutex<AppState>>, day_index: Option<usize>) {
    show_exercise_editor_dialog(
        state,
        ExerciseEditorContext::Create {
            day_index,
            initial: None,
        },
    );
}

#[derive(Clone)]
#[allow(dead_code)]
pub enum ExerciseEditorContext {
    Create {
        day_index: Option<usize>,
        initial: Option<weightlifting_core::Segment>,
    },
    Edit {
        day_index: usize,
        segment_index: usize,
        segment: weightlifting_core::Segment,
    },
}

pub fn show_exercise_editor_dialog(state: Arc<Mutex<AppState>>, ctx: ExerciseEditorContext) {
    let (dlg_title, _day_index) = match &ctx {
        ExerciseEditorContext::Create { day_index, .. } => (
            if let Some(idx) = day_index {
                format!("Add Exercise to Day {}", idx + 1)
            } else {
                "Add Exercise".to_string()
            },
            *day_index,
        ),
        ExerciseEditorContext::Edit { day_index, .. } => {
            ("Edit Exercise".to_string(), Some(*day_index))
        }
    };

    let dialog = Dialog::with_buttons(
        Some(&dlg_title),
        crate::ui::util::parent_for_dialog().as_ref(),
        DialogFlags::MODAL,
        &[
            ("Cancel", ResponseType::Cancel),
            ("Add", ResponseType::Accept),
        ],
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

    // Exercise Search Widget (Default) - extract current alt_group if editing
    let current_alt_group = match &ctx {
        ExerciseEditorContext::Edit { segment, .. } => {
            use weightlifting_core::Segment;
            match segment {
                Segment::Straight(s) => s.base.alt_group.clone(),
                Segment::Rpe(r) => r.base.alt_group.clone(),
                Segment::Amrap(a) => a.base.alt_group.clone(),
                Segment::Time(t) => t.base.alt_group.clone(),
                _ => None,
            }
        }
        ExerciseEditorContext::Create { .. } => None,
    };
    let (
        search_expander,
        group_search_expander,
        ex_entry,
        label_entry,
        alt_group_entry,
        search_widget,
        _group_search_widget,
    ) = create_exercise_search_section(state.clone(), current_alt_group);
    search_expander.set_expanded(true); // Make search the default
    content.append(&search_expander);
    content.append(&group_search_expander);

    // When editing, show current selection summary that updates with new selections
    let selected_summary = Label::new(None);
    if matches!(ctx, ExerciseEditorContext::Edit { .. }) {
        selected_summary.set_css_classes(&["dim-label"]);
        content.append(&selected_summary);
    }

    // Manual Input Fields (Advanced option)
    let manual_expander = Expander::builder()
        .label("⚙️ Manual Entry (Advanced)")
        .expanded(false)
        .build();

    let manual_content = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .build();

    // Sets entry (hidden for AMRAP/Time edits)
    let sets_label = Label::new(Some("Sets:"));
    let sets_entry = SpinButton::with_range(1.0, 20.0, 1.0);
    sets_entry.set_value(3.0);
    content.append(&sets_label);
    content.append(&sets_entry);

    // Reps range entries
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

    content.append(&reps_label);
    content.append(&reps_box);

    // RPE entry
    let rpe_label = Label::new(Some("RPE:"));
    let rpe_entry = SpinButton::with_range(1.0, 10.0, 0.5);
    rpe_entry.set_value(8.0);

    content.append(&rpe_label);
    content.append(&rpe_entry);

    // Manual Input Fields (in expander)
    let ex_label = Label::new(Some("Exercise Code:"));
    ex_entry.set_text("BENCH.BB.FLAT");
    manual_content.append(&ex_label);
    manual_content.append(&ex_entry);

    let label_label = Label::new(Some("Exercise Name:"));
    label_entry.set_text("Bench Press");
    manual_content.append(&label_label);
    manual_content.append(&label_entry);

    let alt_group_label = Label::new(Some("Alternative Group (optional):"));
    manual_content.append(&alt_group_label);
    manual_content.append(&alt_group_entry);

    manual_expander.set_child(Some(&manual_content));
    content.append(&manual_expander);

    // Pre-populate if editing and set initial summary
    match &ctx {
        ExerciseEditorContext::Edit { segment, .. } => {
            use weightlifting_core::{RepsOrRange, Segment};
            match segment {
                Segment::Straight(s) => {
                    ex_entry.set_text(&s.base.ex);
                    label_entry.set_text(&s.base.label.clone().unwrap_or_default());
                    alt_group_entry.set_text(&s.base.alt_group.clone().unwrap_or_default());
                    sets_entry.set_value(s.sets.unwrap_or(3) as f64);
                    if let Some(RepsOrRange::Range(r)) = &s.reps {
                        min_reps.set_value(r.min as f64);
                        max_reps.set_value(r.max as f64);
                    }
                    if let Some(r) = s.rpe {
                        rpe_entry.set_value(r);
                    }
                    selected_summary.set_text(&format!(
                        "Current: {} ({})",
                        if let Some(name) = s.base.label.clone() {
                            name
                        } else {
                            s.base.ex.clone()
                        },
                        s.base.ex
                    ));
                }
                Segment::Rpe(r) => {
                    ex_entry.set_text(&r.base.ex);
                    label_entry.set_text(&r.base.label.clone().unwrap_or_default());
                    alt_group_entry.set_text(&r.base.alt_group.clone().unwrap_or_default());
                    sets_entry.set_value(r.sets as f64);
                    if let Some(RepsOrRange::Range(rr)) = &r.reps {
                        min_reps.set_value(rr.min as f64);
                        max_reps.set_value(rr.max as f64);
                    }
                    rpe_entry.set_value(r.rpe);
                    selected_summary.set_text(&format!(
                        "Current: {} ({})",
                        if let Some(name) = r.base.label.clone() {
                            name
                        } else {
                            r.base.ex.clone()
                        },
                        r.base.ex
                    ));
                }
                Segment::Amrap(a) => {
                    ex_entry.set_text(&a.base.ex);
                    label_entry.set_text(&a.base.label.clone().unwrap_or_default());
                    alt_group_entry.set_text(&a.base.alt_group.clone().unwrap_or_default());
                    // For AMRAP, hide sets; map base/cap reps to min/max UI
                    sets_label.set_visible(false);
                    sets_entry.set_visible(false);
                    min_reps.set_value(a.base_reps as f64);
                    max_reps.set_value(a.cap_reps as f64);
                    selected_summary.set_text(&format!(
                        "Current: {} ({})",
                        if let Some(name) = a.base.label.clone() {
                            name
                        } else {
                            a.base.ex.clone()
                        },
                        a.base.ex
                    ));
                }
                Segment::Time(t) => {
                    ex_entry.set_text(&t.base.ex);
                    label_entry.set_text(&t.base.label.clone().unwrap_or_default());
                    alt_group_entry.set_text(&t.base.alt_group.clone().unwrap_or_default());
                    // Hide sets/reps for Time
                    sets_label.set_visible(false);
                    sets_entry.set_visible(false);
                    reps_label.set_visible(false);
                    reps_box.set_visible(false);
                    if let Some(r) = t.rpe {
                        rpe_entry.set_value(r);
                    }
                    selected_summary.set_text(&format!(
                        "Current: {} ({})",
                        if let Some(name) = t.base.label.clone() {
                            name
                        } else {
                            t.base.ex.clone()
                        },
                        t.base.ex
                    ));
                }
                _ => {}
            }
        }
        ExerciseEditorContext::Create { .. } => {}
    }

    dialog.content_area().append(&content);

    // Add keyboard handling
    let key_controller = EventControllerKey::new();
    let dialog_clone = dialog.clone();
    let search_widget_clone = search_widget.clone();
    let ex_entry_clone = ex_entry.clone();
    let label_entry_clone = label_entry.clone();
    key_controller.connect_key_pressed(move |_, keyval, _, modifiers| {
        match keyval {
            gtk4::gdk::Key::Return | gtk4::gdk::Key::KP_Enter => {
                // Ctrl+Enter always triggers Add button
                if modifiers.contains(gtk4::gdk::ModifierType::CONTROL_MASK) {
                    dialog_clone.response(ResponseType::Accept);
                    return glib::Propagation::Stop;
                }

                // Regular Enter: If search widget has a selection, populate fields and trigger Add button
                if let Some(selected) = search_widget_clone.selected_exercise() {
                    ex_entry_clone.set_text(&selected.code);
                    label_entry_clone.set_text(&selected.name);
                    dialog_clone.response(ResponseType::Accept);
                    return glib::Propagation::Stop;
                }
                glib::Propagation::Proceed
            }
            gtk4::gdk::Key::Escape => {
                dialog_clone.response(ResponseType::Cancel);
                glib::Propagation::Stop
            }
            _ => glib::Propagation::Proceed,
        }
    });
    dialog.add_controller(key_controller);

    // Focus the search entry when dialog opens
    let sw_for_show = search_widget.clone();
    dialog.connect_show(move |_| {
        // Give focus to the search entry for immediate typing
        if let Some(search_container) = sw_for_show.container.first_child() {
            if let Some(search_entry) = search_container.downcast_ref::<Entry>() {
                search_entry.grab_focus();
            }
        }
    });

    // Update summary when a new exercise is selected from search
    if matches!(ctx, ExerciseEditorContext::Edit { .. }) {
        let summary_clone1 = selected_summary.clone();
        let summary_clone2 = selected_summary.clone();
        let sw_for_summary1 = search_widget.clone();
        let sw_for_summary2 = search_widget.clone();
        sw_for_summary1.connect_row_selected(move |res| {
            summary_clone1.set_text(&format!("Selected: {} ({})", res.name, res.code));
        });
        sw_for_summary2.connect_row_activated(move |res| {
            summary_clone2.set_text(&format!("Selected: {} ({})", res.name, res.code));
        });
    }

    dialog.connect_response(clone!(@strong state, @strong alt_group_entry, @strong ctx => move |dialog, response| {
        if response == ResponseType::Accept {
            let ex_code = ex_entry.text().to_string();
            let ex_label = label_entry.text().to_string();
            let alt_group = alt_group_entry.text().to_string();
            let alt_group = if alt_group.trim().is_empty() { None } else { Some(alt_group) };
            let sets = sets_entry.value() as u32;
            let min_reps = min_reps.value() as u32;
            let max_reps = max_reps.value() as u32;
            let rpe = rpe_entry.value();

            match &ctx {
                ExerciseEditorContext::Create { day_index, .. } => {
                    if let Some(day_idx) = day_index {
                        use crate::operations::segment::add_custom_exercise_to_day;
                        add_custom_exercise_to_day(state.clone(), *day_idx, ex_code, ex_label, alt_group, sets, min_reps, max_reps, rpe);
                    } else {
                        add_custom_exercise_to_plan(state.clone(), ex_code, ex_label, alt_group, sets, min_reps, max_reps, rpe);
                    }
                }
                ExerciseEditorContext::Edit { day_index, segment_index, segment } => {
                    use crate::operations::segment::{update_straight_segment, update_rpe_segment, update_amrap_segment, update_time_segment};
                    match segment {
                        weightlifting_core::Segment::Straight(_) => update_straight_segment(
                            state.clone(), *day_index, *segment_index, ex_code, Some(ex_label), alt_group, None, None, None, Some(sets), Some(min_reps), Some(max_reps), Some(rpe), None
                        ),
                        weightlifting_core::Segment::Rpe(_) => update_rpe_segment(
                            state.clone(), *day_index, *segment_index, ex_code, Some(ex_label), alt_group, None, None, None, sets, Some(min_reps), Some(max_reps), rpe, None
                        ),
                        weightlifting_core::Segment::Amrap(_) => update_amrap_segment(
                            state.clone(), *day_index, *segment_index, ex_code, Some(ex_label), alt_group, None, None, None, min_reps, max_reps
                        ),
                        weightlifting_core::Segment::Time(_) => update_time_segment(
                            state.clone(), *day_index, *segment_index, ex_code, Some(ex_label), alt_group, None, None, None, Some(rpe), None
                        ),
                        _ => {}
                    }
                }
            }
        }
        dialog.close();
    }));

    dialog.present();
}

// Helper function to create exercise search section with group selection
fn create_exercise_search_section(
    state: Arc<Mutex<AppState>>,
    current_alt_group: Option<String>,
) -> (
    Expander,
    Expander,
    Entry,
    Entry,
    Entry,
    ExerciseSearchWidget,
    GroupSearchWidget,
) {
    let search_widget = ExerciseSearchWidget::new();
    search_widget.set_state(state.clone());
    if let Err(e) =
        search_widget.set_database_path("/home/jawaad/weightlifting-desktop/exercises.db")
    {
        println!("Failed to connect to exercise database: {}", e);
    }

    let search_expander = Expander::builder()
        .label("🔍 Search Exercise Database")
        .child(&search_widget.container)
        .expanded(false)
        .build();

    // Group search widget with current alternative group
    let group_search_widget = GroupSearchWidget::new(state, current_alt_group);
    let group_search_expander = Expander::builder()
        .label("🔗 Select Alternative Group (right-click to edit)")
        .child(&group_search_widget.container)
        .expanded(false)
        .build();

    // Enable right-click to edit groups
    group_search_widget.enable_right_click_edit();

    // Manual input fields
    let ex_entry = Entry::new();
    let label_entry = Entry::new();
    let alt_group_entry = Entry::new();
    alt_group_entry.set_placeholder_text(Some("e.g., GROUP_CHEST_PRESS"));

    // Connect search widget selection to populate manual fields (both selection and activation)
    let ex_entry_clone = ex_entry.clone();
    let label_entry_clone = label_entry.clone();
    let search_widget_clone = search_widget.clone();
    search_widget.connect_row_activated(move |result| {
        ex_entry_clone.set_text(&result.code);
        label_entry_clone.set_text(&result.name);
        search_widget_clone.set_selected_exercise(result);
    });

    let ex_entry_clone2 = ex_entry.clone();
    let label_entry_clone2 = label_entry.clone();
    search_widget.connect_row_selected(move |result| {
        ex_entry_clone2.set_text(&result.code);
        label_entry_clone2.set_text(&result.name);
    });

    // Connect group search widget to populate alt_group field
    let alt_group_entry_clone = alt_group_entry.clone();
    group_search_widget.connect_row_activated(move |result| {
        alt_group_entry_clone.set_text(&result.name);
    });

    (
        search_expander,
        group_search_expander,
        ex_entry,
        label_entry,
        alt_group_entry,
        search_widget,
        group_search_widget,
    )
}
