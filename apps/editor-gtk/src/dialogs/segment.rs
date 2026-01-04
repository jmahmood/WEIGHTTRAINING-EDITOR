use crate::dialogs::comment::show_add_comment_dialog;
use crate::dialogs::exercise::show_add_exercise_dialog;
use crate::operations::segment::{add_custom_comment_to_day, set_target_day_for_next_segment};
use crate::state::AppState;
use crate::ui::exercise_menus::edit_dialogs::dialog_components::{
    create_base_segment_section, create_sets_reps_section, create_training_params_section,
};
use crate::ui::exercise_menus::{
    show_add_amrap_dialog, show_add_circuit_dialog, show_add_complex_dialog,
    show_add_group_choose_dialog, show_add_group_optional_dialog, show_add_group_rotate_dialog,
    show_add_group_superset_dialog, show_add_percentage_set_dialog, show_add_rpe_set_dialog,
    show_add_scheme_dialog, show_add_superset_dialog, show_add_time_based_dialog,
};
use crate::ui::widgets::GroupSearchWidget;
use glib::clone;
use gtk4::prelude::*;
use gtk4::{
    Box as GtkBox, Button, Dialog, DialogFlags, DropDown, Expander, Label, Orientation,
    ResponseType, ScrolledWindow, SpinButton, StringList, TextView,
};
use std::sync::{Arc, Mutex};

pub fn show_add_segment_to_day_dialog(state: Arc<Mutex<AppState>>, day_index: usize) {
    let dialog = Dialog::with_buttons(
        Some(&format!("Add Segment to Day {}", day_index + 1)),
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

    let type_label = Label::new(Some("Segment Type:"));
    content.append(&type_label);

    let segment_types = StringList::new(&[
        "Basic Exercise",
        "RPE Set",
        "Percentage Set",
        "AMRAP",
        "Superset",
        "Circuit",
        "Scheme",
        "Complex",
        "Comment",
        "Group (Choose)",
        "Group (Rotate)",
        "Group (Optional)",
        "Group (Superset)",
        "Time-based",
    ]);

    let dropdown = DropDown::new(Some(segment_types), None::<gtk4::Expression>);
    content.append(&dropdown);

    dialog.content_area().append(&content);
    crate::ui::util::bind_ctrl_enter_to_accept(&dialog);

    dialog.connect_response(
        clone!(@strong state, @strong dropdown => move |dialog, response| {
            if response == ResponseType::Accept {
                let selected = dropdown.selected();

                // Set the target day for subsequent segment creation
                set_target_day_for_next_segment(state.clone(), day_index);

                match selected {
                    0 => show_add_exercise_dialog(state.clone()), // Basic Exercise
                    1 => {
                        // Import from ui module
                        use crate::ui::exercise_menus::show_add_rpe_set_dialog;
                        show_add_rpe_set_dialog(state.clone());
                    },
                    2 => {
                        use crate::ui::exercise_menus::show_add_percentage_set_dialog;
                        show_add_percentage_set_dialog(state.clone());
                    },
                    3 => {
                        use crate::ui::exercise_menus::show_add_amrap_dialog;
                        show_add_amrap_dialog(state.clone());
                    },
                    4 => {
                        use crate::ui::exercise_menus::show_add_superset_dialog;
                        show_add_superset_dialog(state.clone());
                    },
                    5 => {
                        use crate::ui::exercise_menus::show_add_circuit_dialog;
                        show_add_circuit_dialog(state.clone());
                    },
                    6 => {
                        use crate::ui::exercise_menus::show_add_scheme_dialog;
                        show_add_scheme_dialog(state.clone());
                    },
                    7 => {
                        use crate::ui::exercise_menus::show_add_complex_dialog;
                        show_add_complex_dialog(state.clone());
                    },
                    8 => show_add_comment_to_day_dialog(state.clone(), day_index), // Comment
                    9 => {
                        use crate::ui::exercise_menus::show_add_group_choose_dialog;
                        show_add_group_choose_dialog(state.clone());
                    },
                    10 => {
                        use crate::ui::exercise_menus::show_add_group_rotate_dialog;
                        show_add_group_rotate_dialog(state.clone());
                    },
                    11 => {
                        use crate::ui::exercise_menus::show_add_group_optional_dialog;
                        show_add_group_optional_dialog(state.clone());
                    },
                    12 => {
                        use crate::ui::exercise_menus::show_add_group_superset_dialog;
                        show_add_group_superset_dialog(state.clone());
                    },
                    13 => {
                        use crate::ui::exercise_menus::show_add_time_based_dialog;
                        show_add_time_based_dialog(state.clone());
                    },
                    _ => {
                        println!("Unknown segment type selected: {}", selected);
                    }
                }
            }
            dialog.close();
        }),
    );

    dialog.present();
}

pub fn show_add_comment_to_day_dialog(state: Arc<Mutex<AppState>>, day_index: usize) {
    let dialog = Dialog::with_buttons(
        Some(&format!("Add Comment to Day {}", day_index + 1)),
        crate::ui::util::parent_for_dialog().as_ref(),
        DialogFlags::MODAL,
        &[
            ("Cancel", ResponseType::Cancel),
            ("Add", ResponseType::Accept),
        ],
    );
    crate::ui::util::standardize_dialog(&dialog);
    dialog.set_default_size(400, 300);

    let content = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .margin_start(20)
        .margin_end(20)
        .margin_top(20)
        .margin_bottom(20)
        .spacing(12)
        .build();

    // Icon dropdown
    let icon_label = Label::new(Some("Icon (optional):"));
    let exercise_icons = StringList::new(&[
        "🏋️", "💪", "🔥", "⚡", "🎯", "📝", "⏱️", "🕐", "⏰", "⏲️", "💥", "🚀", "⭐", "🔔", "⚠️",
        "✅", "🔴", "🟡", "🟢", "🔵", "⚫", "📊", "📈", "📉", "🎖️", "🏆", "💯", "🔄", "⏳",
    ]);

    let icon_dropdown = DropDown::new(Some(exercise_icons), None::<gtk4::Expression>);
    icon_dropdown.set_selected(0); // Default to "🏋️"

    content.append(&icon_label);
    content.append(&icon_dropdown);

    // Comment text
    let text_label = Label::new(Some("Comment Text:"));
    content.append(&text_label);

    let text_view = TextView::builder().wrap_mode(gtk4::WrapMode::Word).build();

    let text_buffer = text_view.buffer();
    text_buffer.set_text("Rest between exercises");

    let scrolled = ScrolledWindow::builder()
        .child(&text_view)
        .min_content_height(100)
        .build();

    content.append(&scrolled);

    dialog.content_area().append(&content);
    crate::ui::util::bind_ctrl_enter_to_accept(&dialog);

    dialog.connect_response(clone!(@strong state, @strong icon_dropdown => move |dialog, response| {
        if response == ResponseType::Accept {
            let selected_index = icon_dropdown.selected();
            let icon_options = ["🏋️", "💪", "🔥", "⚡", "🎯", "📝", "⏱️", "🕐", "⏰", "⏲️", "💥", "🚀", "⭐", "🔔", "⚠️",
                               "✅", "🔴", "🟡", "🟢", "🔵", "⚫", "📊", "📈", "📉", "🎖️", "🏆", "💯", "🔄", "⏳"];
            let selected_icon = icon_options[selected_index as usize];
            let icon = Some(selected_icon.to_string());

            let text_buffer = text_view.buffer();
            let (start, end) = text_buffer.bounds();
            let text = text_buffer.text(&start, &end, false).to_string();

            add_custom_comment_to_day(state.clone(), day_index, text, icon);
        }
        dialog.close();
    }));

    dialog.present();
}

pub fn show_edit_comment_dialog(
    state: Arc<Mutex<AppState>>,
    comment: weightlifting_core::CommentSegment,
    day_index: usize,
    segment_index: usize,
) {
    let dialog = Dialog::with_buttons(
        Some("Edit Comment"),
        crate::ui::util::parent_for_dialog().as_ref(),
        DialogFlags::MODAL,
        &[
            ("Cancel", ResponseType::Cancel),
            ("Update", ResponseType::Accept),
        ],
    );
    crate::ui::util::standardize_dialog(&dialog);
    dialog.set_default_size(400, 300);
    crate::ui::util::standardize_dialog(&dialog);
    let content = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .margin_start(20)
        .margin_end(20)
        .margin_top(20)
        .margin_bottom(20)
        .spacing(12)
        .build();

    // Icon dropdown
    let icon_label = Label::new(Some("Icon (optional):"));
    let exercise_icons = StringList::new(&[
        "🏋️", "💪", "🔥", "⚡", "🎯", "📝", "⏱️", "🕐", "⏰", "⏲️", "💥", "🚀", "⭐", "🔔", "⚠️",
        "✅", "🔴", "🟡", "🟢", "🔵", "⚫", "📊", "📈", "📉", "🎖️", "🏆", "💯", "🔄", "⏳",
    ]);

    let icon_dropdown = DropDown::new(Some(exercise_icons), None::<gtk4::Expression>);

    // Set the current icon selection
    if let Some(ref current_icon) = comment.icon {
        let icon_options = [
            "🏋️", "💪", "🔥", "⚡", "🎯", "📝", "⏱️", "🕐", "⏰", "⏲️", "💥", "🚀", "⭐", "🔔",
            "⚠️", "✅", "🔴", "🟡", "🟢", "🔵", "⚫", "📊", "📈", "📉", "🎖️", "🏆", "💯", "🔄",
            "⏳",
        ];
        if let Some(index) = icon_options.iter().position(|&x| x == current_icon) {
            icon_dropdown.set_selected(index as u32);
        }
    } else {
        icon_dropdown.set_selected(0); // Default to first icon
    }

    content.append(&icon_label);
    content.append(&icon_dropdown);

    // Comment text
    let text_label = Label::new(Some("Comment Text:"));
    content.append(&text_label);

    let text_view = TextView::builder().wrap_mode(gtk4::WrapMode::Word).build();

    let text_buffer = text_view.buffer();
    text_buffer.set_text(&comment.text);

    let scrolled = ScrolledWindow::builder()
        .child(&text_view)
        .min_content_height(100)
        .build();

    content.append(&scrolled);

    dialog.content_area().append(&content);
    crate::ui::util::bind_ctrl_enter_to_accept(&dialog);

    dialog.connect_response(clone!(@strong state, @strong icon_dropdown => move |dialog, response| {
        if response == ResponseType::Accept {
            let selected_index = icon_dropdown.selected();
            let icon_options = ["🏋️", "💪", "🔥", "⚡", "🎯", "📝", "⏱️", "🕐", "⏰", "⏲️", "💥", "🚀", "⭐", "🔔", "⚠️",
                               "✅", "🔴", "🟡", "🟢", "🔵", "⚫", "📊", "📈", "📉", "🎖️", "🏆", "💯", "🔄", "⏳"];
            let selected_icon = icon_options[selected_index as usize];
            let icon = Some(selected_icon.to_string());

            let text_buffer = text_view.buffer();
            let (start, end) = text_buffer.bounds();
            let text = text_buffer.text(&start, &end, false).to_string();

            use crate::operations::segment::update_comment_segment;
            update_comment_segment(state.clone(), day_index, segment_index, text, icon);
        }
        dialog.close();
    }));

    dialog.present();
}

/// Generic edit dialog for simple segment types (Straight, RPE, AMRAP, Time)
#[allow(dead_code)]
pub fn show_generic_segment_edit_dialog(
    state: Arc<Mutex<AppState>>,
    segment: &weightlifting_core::Segment,
    day_index: usize,
    segment_index: usize,
    title: &str,
) {
    // Clone the segment to avoid lifetime issues
    let segment_clone = segment.clone();
    let dialog = Dialog::with_buttons(
        Some(title),
        crate::ui::util::parent_for_dialog().as_ref(),
        DialogFlags::MODAL,
        &[
            ("Cancel", ResponseType::Cancel),
            ("Update", ResponseType::Accept),
        ],
    );

    dialog.set_default_size(400, 500);

    let content = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .margin_start(20)
        .margin_end(20)
        .margin_top(20)
        .margin_bottom(20)
        .spacing(12)
        .build();

    // Extract values based on segment type
    let (ex, label, alt_group, sets, min_reps, max_reps, rpe, rest_sec) = match segment {
        weightlifting_core::Segment::Straight(s) => {
            let (min_r, max_r) =
                if let Some(weightlifting_core::RepsOrRange::Range(range)) = &s.reps {
                    (Some(range.min), Some(range.max))
                } else {
                    (Some(8), Some(10))
                };
            let rest = if let Some(weightlifting_core::RestOrRange::Fixed(r)) = &s.rest_sec {
                Some(*r)
            } else {
                None
            };
            (
                s.base.ex.clone(),
                s.base.label.as_deref(),
                s.base.alt_group.as_deref(),
                s.sets,
                min_r,
                max_r,
                s.rpe,
                rest,
            )
        }
        weightlifting_core::Segment::Rpe(r) => {
            let (min_r, max_r) =
                if let Some(weightlifting_core::RepsOrRange::Range(range)) = &r.reps {
                    (Some(range.min), Some(range.max))
                } else {
                    (Some(8), Some(10))
                };
            let rest = if let Some(weightlifting_core::RestOrRange::Fixed(rest_val)) = &r.rest_sec {
                Some(*rest_val)
            } else {
                None
            };
            (
                r.base.ex.clone(),
                r.base.label.as_deref(),
                r.base.alt_group.as_deref(),
                Some(r.sets),
                min_r,
                max_r,
                Some(r.rpe),
                rest,
            )
        }
        weightlifting_core::Segment::Amrap(a) => (
            a.base.ex.clone(),
            a.base.label.as_deref(),
            a.base.alt_group.as_deref(),
            None,
            Some(a.base_reps),
            Some(a.cap_reps),
            None,
            None,
        ),
        weightlifting_core::Segment::Time(t) => (
            t.base.ex.clone(),
            t.base.label.as_deref(),
            t.base.alt_group.as_deref(),
            None,
            None,
            None,
            t.rpe,
            None,
        ),
        _ => return, // Should not happen
    };

    let (group_role, per_week_json, load_axis_target_json) = match segment {
        weightlifting_core::Segment::Straight(s) => (
            s.base.group_role.as_deref(),
            s.base
                .per_week
                .as_ref()
                .and_then(|pw| serde_json::to_string(pw).ok()),
            s.base
                .load_axis_target
                .as_ref()
                .and_then(|lat| serde_json::to_string(lat).ok()),
        ),
        weightlifting_core::Segment::Rpe(r) => (
            r.base.group_role.as_deref(),
            r.base
                .per_week
                .as_ref()
                .and_then(|pw| serde_json::to_string(pw).ok()),
            r.base
                .load_axis_target
                .as_ref()
                .and_then(|lat| serde_json::to_string(lat).ok()),
        ),
        weightlifting_core::Segment::Amrap(a) => (
            a.base.group_role.as_deref(),
            a.base
                .per_week
                .as_ref()
                .and_then(|pw| serde_json::to_string(pw).ok()),
            a.base
                .load_axis_target
                .as_ref()
                .and_then(|lat| serde_json::to_string(lat).ok()),
        ),
        weightlifting_core::Segment::Time(t) => (
            t.base.group_role.as_deref(),
            t.base
                .per_week
                .as_ref()
                .and_then(|pw| serde_json::to_string(pw).ok()),
            t.base
                .load_axis_target
                .as_ref()
                .and_then(|lat| serde_json::to_string(lat).ok()),
        ),
        _ => (None, None, None),
    };

    // Create UI sections using shared components
    let (base_section, ex_entry, label_entry, alt_entry, group_role_entry, per_week_entry, load_axis_target_entry) =
        create_base_segment_section(
            &ex,
            label,
            alt_group,
            group_role,
            per_week_json.as_deref(),
            load_axis_target_json.as_deref(),
        );
    content.append(&base_section);
    alt_entry.set_sensitive(false);

    // Use GroupSearchWidget with current alternative group
    let current_alt_group = alt_group.map(|s| s.to_string());
    let group_search_widget = GroupSearchWidget::new(state.clone(), current_alt_group);
    let group_search_expander = Expander::builder()
        .label("🔗 Select Alternative Group (right-click to edit)")
        .child(&group_search_widget.container)
        .expanded(false)
        .build();

    // Connect group search widget to populate alt_entry field
    let alt_entry_for_group = alt_entry.clone();
    group_search_widget.connect_row_activated(move |result| {
        alt_entry_for_group.set_text(&result.name);
    });

    // Enable right-click to edit groups
    group_search_widget.enable_right_click_edit();

    content.append(&group_search_expander);

    // Add sets/reps section for most types
    let sets_reps_widgets = if !matches!(segment, weightlifting_core::Segment::Time(_)) {
        let (sets_section, sets_entry, min_reps_entry, max_reps_entry) =
            create_sets_reps_section(sets, min_reps, max_reps);
        content.append(&sets_section);
        Some((sets_entry, min_reps_entry, max_reps_entry))
    } else {
        None
    };

    // Add training parameters section
    let (params_section, rpe_entry, rest_entry) = create_training_params_section(rpe, rest_sec);
    content.append(&params_section);

    dialog.content_area().append(&content);
    crate::ui::util::bind_ctrl_enter_to_accept(&dialog);

    dialog.connect_response(clone!(@strong state, @strong segment_clone => move |dialog, response| {
        if response == ResponseType::Accept {
            let ex_text = ex_entry.text().to_string();
            let label_text = if label_entry.text().is_empty() {
                None
            } else {
                Some(label_entry.text().to_string())
            };
            let alt_text = if alt_entry.text().is_empty() {
                None
            } else {
                Some(alt_entry.text().to_string())
            };
            let group_role_text = if group_role_entry.text().is_empty() {
                None
            } else {
                Some(group_role_entry.text().to_string())
            };
            let per_week_text = if per_week_entry.text().is_empty() {
                None
            } else {
                match serde_json::from_str::<std::collections::HashMap<String, serde_json::Value>>(
                    &per_week_entry.text(),
                ) {
                    Ok(map) => Some(map),
                    Err(e) => {
                        println!("Invalid per_week JSON: {}", e);
                        None
                    }
                }
            };
            let load_axis_target_text = if load_axis_target_entry.text().is_empty() {
                None
            } else {
                match serde_json::from_str::<weightlifting_core::LoadAxisTarget>(
                    &load_axis_target_entry.text(),
                ) {
                    Ok(t) => Some(t),
                    Err(e) => {
                        println!("Invalid load_axis_target JSON: {}", e);
                        None
                    }
                }
            };
            let rpe_val = if rpe_entry.value() > 0.0 { Some(rpe_entry.value()) } else { None };
            let rest_val = if rest_entry.value() > 0.0 { Some(rest_entry.value() as u32) } else { None };

            // Call appropriate update operation based on segment type
            match &segment_clone {
                weightlifting_core::Segment::Straight(_) => {
                    if let Some((sets_entry, min_reps_entry, max_reps_entry)) = &sets_reps_widgets {
                        let sets_val = Some(sets_entry.value() as u32);
                        let min_reps_val = Some(min_reps_entry.value() as u32);
                        let max_reps_val = Some(max_reps_entry.value() as u32);
                        use crate::operations::segment::update_straight_segment;
                        update_straight_segment(state.clone(), day_index, segment_index,
                            ex_text, label_text, alt_text, group_role_text.clone(), per_week_text.clone(), load_axis_target_text.clone(), sets_val, min_reps_val, max_reps_val, rpe_val, rest_val);
                    }
                },
                weightlifting_core::Segment::Rpe(_) => {
                    if let Some((sets_entry, min_reps_entry, max_reps_entry)) = &sets_reps_widgets {
                        let sets_val = sets_entry.value() as u32;
                        let min_reps_val = Some(min_reps_entry.value() as u32);
                        let max_reps_val = Some(max_reps_entry.value() as u32);
                        let rpe_val = rpe_entry.value(); // RPE is required for RPE segments
                        use crate::operations::segment::update_rpe_segment;
                        update_rpe_segment(state.clone(), day_index, segment_index,
                            ex_text, label_text, alt_text, group_role_text.clone(), per_week_text.clone(), load_axis_target_text.clone(), sets_val, min_reps_val, max_reps_val, rpe_val, rest_val);
                    }
                },
                weightlifting_core::Segment::Amrap(_) => {
                    if let Some((_, min_reps_entry, max_reps_entry)) = &sets_reps_widgets {
                        let base_reps = min_reps_entry.value() as u32;
                        let cap_reps = max_reps_entry.value() as u32;
                        use crate::operations::segment::update_amrap_segment;
                        update_amrap_segment(state.clone(), day_index, segment_index,
                            ex_text, label_text, alt_text, group_role_text.clone(), per_week_text.clone(), load_axis_target_text.clone(), base_reps, cap_reps);
                    }
                },
                weightlifting_core::Segment::Time(_) => {
                    use crate::operations::segment::update_time_segment;
                    update_time_segment(state.clone(), day_index, segment_index,
                        ex_text, label_text, alt_text, group_role_text.clone(), per_week_text.clone(), load_axis_target_text.clone(), rpe_val, rest_val);
                },
                _ => {}
            }
        }
        dialog.close();
    }));

    dialog.present();
}

/// Specialized edit dialog for Percentage segments
pub fn show_edit_percentage_dialog(
    state: Arc<Mutex<AppState>>,
    segment: &weightlifting_core::Segment,
    day_index: usize,
    segment_index: usize,
) {
    if let weightlifting_core::Segment::Percentage(p) = segment {
        let dialog = Dialog::with_buttons(
            Some("Edit Percentage Set"),
            crate::ui::util::parent_for_dialog().as_ref(),
            DialogFlags::MODAL,
            &[
                ("Cancel", ResponseType::Cancel),
                ("Update", ResponseType::Accept),
            ],
        );
        crate::ui::util::standardize_dialog(&dialog);

        dialog.set_default_size(500, 400);

        let content = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .margin_start(20)
            .margin_end(20)
            .margin_top(20)
            .margin_bottom(20)
            .spacing(12)
            .build();

        let group_role = p.base.group_role.as_deref();
        let per_week_json = p
            .base
            .per_week
            .as_ref()
            .and_then(|pw| serde_json::to_string(pw).ok());
        let load_axis_target_json = p
            .base
            .load_axis_target
            .as_ref()
            .and_then(|lat| serde_json::to_string(lat).ok());

        // Base segment section
        let (base_section, ex_entry, label_entry, alt_entry, group_role_entry, per_week_entry, load_axis_target_entry) =
            create_base_segment_section(
                &p.base.ex,
                p.base.label.as_deref(),
                p.base.alt_group.as_deref(),
                group_role,
                per_week_json.as_deref(),
                load_axis_target_json.as_deref(),
            );
        content.append(&base_section);
        alt_entry.set_sensitive(false);
        let alt_group_label_dd = Label::new(Some("Alternative Group:"));
        let groups_list = StringList::new(&["None"]);
        {
            let state_lock = state.lock().unwrap();
            if let Some(plan) = &state_lock.current_plan {
                let mut names: Vec<String> = plan.groups.keys().cloned().collect();
                names.sort();
                for name in &names {
                    groups_list.append(name);
                }
                let mut sel = 0u32;
                if let Some(ag) = p.base.alt_group.as_deref() {
                    if let Some(pos) = names.iter().position(|n| n == ag) {
                        sel = (pos + 1) as u32;
                    }
                }
                let alt_dropdown =
                    DropDown::new(Some(groups_list.clone()), None::<gtk4::Expression>);
                alt_dropdown.set_selected(sel);
                let names_clone = names.clone();
                alt_dropdown.connect_selected_notify(clone!(@strong alt_entry => move |dd| {
                    let idx = dd.selected() as usize;
                    if idx == 0 { alt_entry.set_text(""); } else { alt_entry.set_text(&names_clone[idx - 1]); }
                }));
                content.append(&alt_group_label_dd);
                content.append(&alt_dropdown);
            }
        }

        // Prescriptions section
        let prescriptions_label = Label::new(Some("Prescriptions (Sets × Reps @ %1RM):"));
        content.append(&prescriptions_label);

        let scrolled = ScrolledWindow::builder().min_content_height(200).build();

        let prescriptions_box = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(4)
            .build();

        // Create prescription entries from existing data
        let mut prescription_widgets = Vec::new();
        for prescription in &p.prescriptions {
            let presc_row = GtkBox::builder()
                .orientation(Orientation::Horizontal)
                .spacing(8)
                .build();

            let sets_entry = SpinButton::with_range(1.0, 20.0, 1.0);
            sets_entry.set_value(prescription.sets as f64);
            let reps_entry = SpinButton::with_range(1.0, 50.0, 1.0);
            reps_entry.set_value(prescription.reps as f64);
            let pct_entry = SpinButton::with_range(10.0, 150.0, 5.0);
            pct_entry.set_value(prescription.pct_1rm);

            presc_row.append(&sets_entry);
            presc_row.append(&Label::new(Some("×")));
            presc_row.append(&reps_entry);
            presc_row.append(&Label::new(Some("@")));
            presc_row.append(&pct_entry);
            presc_row.append(&Label::new(Some("%")));

            let remove_btn = Button::with_label("Remove");
            let prescriptions_box_clone = prescriptions_box.clone();
            let presc_row_clone = presc_row.clone();
            remove_btn.connect_clicked(move |_| {
                prescriptions_box_clone.remove(&presc_row_clone);
            });
            presc_row.append(&remove_btn);

            prescriptions_box.append(&presc_row);
            prescription_widgets.push((sets_entry, reps_entry, pct_entry, presc_row));
        }

        scrolled.set_child(Some(&prescriptions_box));
        content.append(&scrolled);

        // Add prescription button
        let add_btn = Button::with_label("+ Add Prescription");
        let prescriptions_box_clone = prescriptions_box.clone();
        add_btn.connect_clicked(move |_| {
            let presc_row = GtkBox::builder()
                .orientation(Orientation::Horizontal)
                .spacing(8)
                .build();

            let sets_entry = SpinButton::with_range(1.0, 20.0, 1.0);
            sets_entry.set_value(3.0);
            let reps_entry = SpinButton::with_range(1.0, 50.0, 1.0);
            reps_entry.set_value(5.0);
            let pct_entry = SpinButton::with_range(10.0, 150.0, 5.0);
            pct_entry.set_value(80.0);

            presc_row.append(&sets_entry);
            presc_row.append(&Label::new(Some("×")));
            presc_row.append(&reps_entry);
            presc_row.append(&Label::new(Some("@")));
            presc_row.append(&pct_entry);
            presc_row.append(&Label::new(Some("%")));

            let remove_btn = Button::with_label("Remove");
            let prescriptions_box_clone2 = prescriptions_box_clone.clone();
            let presc_row_clone = presc_row.clone();
            remove_btn.connect_clicked(move |_| {
                prescriptions_box_clone2.remove(&presc_row_clone);
            });
            presc_row.append(&remove_btn);

            prescriptions_box_clone.append(&presc_row);
        });
        content.append(&add_btn);

        dialog.content_area().append(&content);

        dialog.connect_response(
            clone!(@strong state, @strong prescriptions_box, @strong group_role_entry, @strong per_week_entry, @strong load_axis_target_entry => move |dialog, response| {
                if response == ResponseType::Accept {
                    let ex_text = ex_entry.text().to_string();
                    let label_text = if label_entry.text().is_empty() {
                        None
                    } else {
                        Some(label_entry.text().to_string())
                    };
                    let alt_text = if alt_entry.text().is_empty() {
                        None
                    } else {
                        Some(alt_entry.text().to_string())
                    };
                    let group_role_text = if group_role_entry.text().is_empty() {
                        None
                    } else {
                        Some(group_role_entry.text().to_string())
                    };
                    let per_week_text = if per_week_entry.text().is_empty() {
                        None
                    } else {
                        match serde_json::from_str::<std::collections::HashMap<String, serde_json::Value>>(
                            &per_week_entry.text(),
                        ) {
                            Ok(map) => Some(map),
                            Err(e) => {
                                println!("Invalid per_week JSON: {}", e);
                                None
                            }
                        }
                    };
                    let load_axis_target_text = if load_axis_target_entry.text().is_empty() {
                        None
                    } else {
                        match serde_json::from_str::<weightlifting_core::LoadAxisTarget>(
                            &load_axis_target_entry.text(),
                        ) {
                            Ok(t) => Some(t),
                            Err(e) => {
                                println!("Invalid load_axis_target JSON: {}", e);
                                None
                            }
                        }
                    };

                    // Collect prescriptions from UI
                    let mut prescriptions = Vec::new();
                    let mut child = prescriptions_box.first_child();
                    while let Some(row) = child {
                        let row_clone = row.clone();
                        if let Ok(presc_row) = row.downcast::<GtkBox>() {
                            // Extract the spin buttons from the row
                            let mut child_widget = presc_row.first_child();
                            let mut widgets = Vec::new();
                            while let Some(widget) = child_widget {
                                let widget_clone = widget.clone();
                                if let Ok(spin_btn) = widget.downcast::<SpinButton>() {
                                    widgets.push(spin_btn);
                                }
                                child_widget = widget_clone.next_sibling();
                            }

                            if widgets.len() >= 3 {
                                prescriptions.push(weightlifting_core::PercentagePrescription {
                                    sets: widgets[0].value() as u32,
                                    reps: widgets[1].value() as u32,
                                    pct_1rm: widgets[2].value(),
                                });
                            }
                        }
                        child = row_clone.next_sibling();
                    }

                    use crate::operations::segment::update_percentage_segment;
                    update_percentage_segment(state.clone(), day_index, segment_index,
                        ex_text, label_text, alt_text, group_role_text, per_week_text, load_axis_target_text, prescriptions);
                }
                dialog.close();
            }),
        );

        dialog.present();
    }
}

pub fn edit_segment_if_applicable(
    state: Arc<Mutex<AppState>>,
    segment: &weightlifting_core::Segment,
    day_index: usize,
    segment_index: usize,
) {
    match segment {
        weightlifting_core::Segment::Superset(superset) => {
            use crate::ui::exercise_menus::show_edit_superset_dialog;
            show_edit_superset_dialog(state.clone(), superset.clone(), day_index, segment_index);
        }
        weightlifting_core::Segment::Circuit(circuit) => {
            use crate::ui::exercise_menus::show_edit_circuit_dialog;
            show_edit_circuit_dialog(state.clone(), circuit.clone(), day_index, segment_index);
        }
        weightlifting_core::Segment::Scheme(scheme) => {
            use crate::ui::exercise_menus::show_edit_scheme_dialog;
            show_edit_scheme_dialog(state.clone(), scheme.clone(), day_index, segment_index);
        }
        weightlifting_core::Segment::Complex(complex) => {
            use crate::ui::exercise_menus::show_edit_complex_dialog;
            show_edit_complex_dialog(state.clone(), complex.clone(), day_index, segment_index);
        }
        weightlifting_core::Segment::Comment(comment) => {
            show_edit_comment_dialog(state.clone(), comment.clone(), day_index, segment_index);
        }
        weightlifting_core::Segment::Straight(_)
        | weightlifting_core::Segment::Rpe(_)
        | weightlifting_core::Segment::Amrap(_)
        | weightlifting_core::Segment::Time(_) => {
            use crate::dialogs::exercise::{show_exercise_editor_dialog, ExerciseEditorContext};
            show_exercise_editor_dialog(
                state.clone(),
                ExerciseEditorContext::Edit {
                    day_index,
                    segment_index,
                    segment: segment.clone(),
                },
            );
        }
        weightlifting_core::Segment::Percentage(_) => {
            show_edit_percentage_dialog(state.clone(), segment, day_index, segment_index);
        }
        _ => {
            // Group segments and other types not yet implemented
        }
    }
}

pub fn show_add_segment_dialog(state: Arc<Mutex<AppState>>) {
    let dialog = Dialog::with_buttons(
        Some("Add Segment"),
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

    let type_label = Label::new(Some("Segment Type:"));
    content.append(&type_label);

    let segment_types = StringList::new(&[
        "Basic Exercise",
        "RPE Set",
        "Percentage Set",
        "AMRAP",
        "Superset",
        "Circuit",
        "Scheme",
        "Complex",
        "Comment",
        "Group (Choose)",
        "Group (Rotate)",
        "Group (Optional)",
        "Group (Superset)",
        "Time-based",
    ]);

    let dropdown = DropDown::new(Some(segment_types), None::<gtk4::Expression>);
    content.append(&dropdown);

    dialog.content_area().append(&content);
    crate::ui::util::bind_ctrl_enter_to_accept(&dialog);

    dialog.connect_response(
        clone!(@strong state, @strong dropdown => move |dialog, response| {
            if response == ResponseType::Accept {
                let selected = dropdown.selected();
                match selected {
                    0 => show_add_exercise_dialog(state.clone()), // Basic Exercise
                    1 => show_add_rpe_set_dialog(state.clone()),   // RPE Set
                    2 => show_add_percentage_set_dialog(state.clone()), // Percentage Set
                    3 => show_add_amrap_dialog(state.clone()),     // AMRAP
                    4 => show_add_superset_dialog(state.clone()),  // Superset
                    5 => show_add_circuit_dialog(state.clone()),   // Circuit
                    6 => show_add_scheme_dialog(state.clone()),    // Scheme
                    7 => show_add_complex_dialog(state.clone()),   // Complex
                    8 => show_add_comment_dialog(state.clone()),   // Comment
                    9 => show_add_group_choose_dialog(state.clone()),    // Group (Choose)
                    10 => show_add_group_rotate_dialog(state.clone()),   // Group (Rotate)
                    11 => show_add_group_optional_dialog(state.clone()), // Group (Optional)
                    12 => show_add_group_superset_dialog(state.clone()), // Group (Superset)
                    13 => show_add_time_based_dialog(state.clone()),     // Time-based
                    _ => {
                        println!("Unknown segment type selected: {}", selected);
                    }
                }
            }
            dialog.close();
        }),
    );

    dialog.present();
}
