// Scheme edit dialog functionality

use crate::operations::plan_ops::update_scheme_full_in_plan;
use crate::state::AppState;
use crate::ui::exercise_menus::exercise_data::SchemeSetData;
use crate::ui::widgets::ExerciseSearchWidget;
use glib::clone;
use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Dialog, DialogFlags, Label, Orientation, ResponseType};
use std::sync::{Arc, Mutex};

pub fn show_edit_scheme_dialog(
    state: Arc<Mutex<AppState>>,
    scheme: weightlifting_core::SchemeSegment,
    day_index: usize,
    segment_index: usize,
) {
    let dialog = Dialog::with_buttons(
        Some("Edit Scheme"),
        crate::ui::util::parent_for_dialog().as_ref(),
        DialogFlags::MODAL,
        &[
            ("Cancel", ResponseType::Cancel),
            ("Update", ResponseType::Accept),
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

    // Exercise Search Widget (search-first, updates fields)
    let search_widget = ExerciseSearchWidget::new();
    if let Err(e) =
        search_widget.set_database_path("/home/jawaad/weightlifting-desktop/exercises.db")
    {
        println!("Failed to connect to exercise database: {}", e);
    }
    let search_expander = gtk4::Expander::builder()
        .label("🔍 Search Exercise Database")
        .child(&search_widget.container)
        .expanded(true)
        .build();
    content.append(&search_expander);

    // Selection status directly under the search expander (remains visible when collapsed)
    let selected_status = Label::new(Some(&format!(
        "Selected: {} ({})",
        scheme
            .base
            .label
            .clone()
            .unwrap_or_else(|| scheme.base.ex.clone()),
        scheme.base.ex
    )));
    selected_status.set_css_classes(&["dim-label"]);
    selected_status.set_halign(gtk4::Align::Start);
    content.append(&selected_status);

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
    let ex_entry = gtk4::Entry::new();
    ex_entry.set_text(&scheme.base.ex);

    let label_label = Label::new(Some("Exercise Label:"));
    let label_entry = gtk4::Entry::new();
    label_entry.set_text(
        &scheme
            .base
            .label
            .clone()
            .unwrap_or_else(|| scheme.base.ex.clone()),
    );

    // Alternative Group (optional) — compact dropdown selector like <select>
    use gtk4::{DropDown, StringList};
    let alt_group_label = Label::new(Some("Alternative Group (optional):"));
    let row = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .build();
    let groups_list = StringList::new(&["None"]);
    let mut names: Vec<String> = {
        let s = state.lock().unwrap();
        if let Some(plan) = &s.current_plan {
            plan.groups.keys().cloned().collect()
        } else {
            vec![]
        }
    };
    names.sort();
    for name in &names {
        groups_list.append(name);
    }
    let alt_dropdown = DropDown::new(Some(groups_list.clone()), None::<gtk4::Expression>);
    // Custom list item with preview of members
    let list_factory = gtk4::SignalListItemFactory::new();
    let state_for_factory = state.clone();
    list_factory.connect_setup(move |_, list_item| {
        let vbox = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(2)
            .build();
        let name = Label::new(None);
        name.set_halign(gtk4::Align::Start);
        let preview = Label::new(None);
        preview.add_css_class("dim-label");
        preview.set_halign(gtk4::Align::Start);
        preview.set_wrap(true);
        preview.set_wrap_mode(gtk4::pango::WrapMode::WordChar);
        vbox.append(&name);
        vbox.append(&preview);
        list_item.set_child(Some(&vbox));
    });
    let state_for_bind = state_for_factory.clone();
    list_factory.connect_bind(move |_, list_item| {
        let string_object = list_item
            .item()
            .unwrap()
            .downcast::<gtk4::StringObject>()
            .unwrap();
        let name_lbl = list_item
            .child()
            .unwrap()
            .downcast::<GtkBox>()
            .unwrap()
            .first_child()
            .unwrap()
            .downcast::<Label>()
            .unwrap();
        let preview_lbl = list_item
            .child()
            .unwrap()
            .downcast::<GtkBox>()
            .unwrap()
            .last_child()
            .unwrap()
            .downcast::<Label>()
            .unwrap();
        let gname = string_object.string();
        name_lbl.set_text(&gname);
        if gname == "None" {
            preview_lbl.set_text("");
        } else {
            let s = state_for_bind.lock().unwrap();
            if let Some(plan) = &s.current_plan {
                if let Some(exs) = plan.groups.get(gname.as_str()) {
                    let names: Vec<String> = exs
                        .iter()
                        .take(4)
                        .map(|code| {
                            plan.dictionary
                                .get(code)
                                .cloned()
                                .unwrap_or_else(|| code.clone())
                        })
                        .collect();
                    let mut pv = names.join(", ");
                    if exs.len() > 4 {
                        pv.push_str(", …");
                    }
                    preview_lbl.set_text(&pv);
                } else {
                    preview_lbl.set_text("");
                }
            } else {
                preview_lbl.set_text("");
            }
        }
    });
    alt_dropdown.set_list_factory(Some(&list_factory));
    // Set current selection based on scheme.base.alt_group
    let mut sel = 0u32;
    if let Some(ag) = &scheme.base.alt_group {
        if let Some(pos) = names.iter().position(|n| n == ag) {
            sel = (pos + 1) as u32;
        }
    }
    alt_dropdown.set_selected(sel);
    // Edit button opens group manager
    let edit_group_btn = gtk4::Button::with_label("Edit Group…");
    row.append(&alt_dropdown);
    row.append(&edit_group_btn);
    content.append(&alt_group_label);
    content.append(&row);

    // Scheme sets editor
    let sets_title = Label::new(Some("Scheme Sets:"));
    sets_title.set_css_classes(&["heading"]);
    content.append(&sets_title);
    let sets_container = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(6)
        .build();
    content.append(&sets_container);

    // Helper to create a row for a set
    let add_set_row = |container: &GtkBox, data: SchemeSetData| {
        let row = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .build();
        let label_entry = gtk4::Entry::new();
        if let Some(lbl) = &data.label {
            label_entry.set_text(lbl);
        }
        label_entry.set_placeholder_text(Some("Label (optional)"));
        label_entry.set_width_chars(12);
        let sets_entry = gtk4::SpinButton::with_range(1.0, 20.0, 1.0);
        sets_entry.set_value(data.sets.unwrap_or(1) as f64);
        let reps_min = gtk4::SpinButton::with_range(1.0, 50.0, 1.0);
        reps_min.set_value(data.reps_min.unwrap_or(8) as f64);
        let reps_max = gtk4::SpinButton::with_range(1.0, 50.0, 1.0);
        reps_max.set_value(data.reps_max.unwrap_or(12) as f64);
        let time_entry = gtk4::SpinButton::with_range(0.0, 600.0, 5.0);
        time_entry.set_value(data.time_sec.unwrap_or(0) as f64);
        let rpe_entry = gtk4::SpinButton::with_range(0.0, 10.0, 0.5);
        rpe_entry.set_value(data.rpe.unwrap_or(0.0));
        let rest_entry = gtk4::SpinButton::with_range(0.0, 600.0, 5.0);
        rest_entry.set_value(data.rest_sec.unwrap_or(0) as f64);
        let remove_btn = gtk4::Button::from_icon_name("user-trash-symbolic");
        remove_btn.set_css_classes(&["flat"]);

        // Simple mode toggle: time takes precedence if >0
        let reps_label = Label::new(Some("reps:"));
        let time_label = Label::new(Some("time:"));

        row.append(&label_entry);
        row.append(&sets_entry);
        row.append(&reps_label);
        row.append(&reps_min);
        row.append(&Label::new(Some("-")));
        row.append(&reps_max);
        row.append(&time_label);
        row.append(&time_entry);
        row.append(&Label::new(Some("RPE:")));
        row.append(&rpe_entry);
        row.append(&Label::new(Some("Rest:")));
        row.append(&rest_entry);
        row.append(&remove_btn);

        // Remove handler
        let row_clone = row.clone();
        let container_clone = container.clone();
        remove_btn.connect_clicked(move |_| {
            container_clone.remove(&row_clone);
        });

        container.append(&row);
    };

    // Populate existing sets
    for s in &scheme.sets {
        let data = SchemeSetData {
            label: s.label.clone(),
            sets: s.sets,
            reps_min: match &s.reps {
                Some(weightlifting_core::RepsOrRange::Range(r)) => Some(r.min),
                _ => None,
            },
            reps_max: match &s.reps {
                Some(weightlifting_core::RepsOrRange::Range(r)) => Some(r.max),
                _ => None,
            },
            time_sec: match &s.time_sec {
                Some(weightlifting_core::TimeOrRange::Fixed(t)) => Some(*t),
                _ => None,
            },
            rpe: match &s.rpe {
                Some(weightlifting_core::RpeOrRange::Fixed(v)) => Some(*v),
                _ => None,
            },
            rest_sec: match &s.rest_sec {
                Some(weightlifting_core::RestOrRange::Fixed(v)) => Some(*v),
                _ => None,
            },
        };
        add_set_row(&sets_container, data);
    }

    // Add set button
    let add_set_btn = gtk4::Button::with_label("+ Add Set");
    let container_for_add = sets_container.clone();
    add_set_btn.connect_clicked(move |_| {
        let data = SchemeSetData {
            label: None,
            sets: Some(1),
            reps_min: Some(8),
            reps_max: Some(12),
            time_sec: None,
            rpe: None,
            rest_sec: None,
        };
        let adder = |c: &GtkBox, d: SchemeSetData| {
            let row = GtkBox::builder()
                .orientation(Orientation::Horizontal)
                .spacing(8)
                .build();
            let label_entry = gtk4::Entry::new();
            label_entry.set_placeholder_text(Some("Label (optional)"));
            label_entry.set_width_chars(12);
            let sets_entry = gtk4::SpinButton::with_range(1.0, 20.0, 1.0);
            sets_entry.set_value(d.sets.unwrap_or(1) as f64);
            let reps_min = gtk4::SpinButton::with_range(1.0, 50.0, 1.0);
            reps_min.set_value(d.reps_min.unwrap_or(8) as f64);
            let reps_max = gtk4::SpinButton::with_range(1.0, 50.0, 1.0);
            reps_max.set_value(d.reps_max.unwrap_or(12) as f64);
            let time_entry = gtk4::SpinButton::with_range(0.0, 600.0, 5.0);
            let rpe_entry = gtk4::SpinButton::with_range(0.0, 10.0, 0.5);
            let rest_entry = gtk4::SpinButton::with_range(0.0, 600.0, 5.0);
            let remove_btn = gtk4::Button::from_icon_name("user-trash-symbolic");
            remove_btn.set_css_classes(&["flat"]);
            row.append(&label_entry);
            row.append(&sets_entry);
            row.append(&Label::new(Some("reps:")));
            row.append(&reps_min);
            row.append(&Label::new(Some("-")));
            row.append(&reps_max);
            row.append(&Label::new(Some("time:")));
            row.append(&time_entry);
            row.append(&Label::new(Some("RPE:")));
            row.append(&rpe_entry);
            row.append(&Label::new(Some("Rest:")));
            row.append(&rest_entry);
            row.append(&remove_btn);
            let row_clone = row.clone();
            let container_clone = c.clone();
            remove_btn.connect_clicked(move |_| {
                container_clone.remove(&row_clone);
            });
            c.append(&row);
        };
        adder(&container_for_add, data);
    });
    content.append(&add_set_btn);

    // For now, just allow editing the exercise info - full scheme editing would be complex
    // let note_label = Label::new(Some("Note: Full scheme set editing not yet implemented. You can only edit the exercise code and label."));
    // note_label.set_css_classes(&["dim-label"]);
    // content.append(&note_label);

    // Build the UI
    manual_box.append(&ex_label);
    manual_box.append(&ex_entry);
    manual_box.append(&label_label);
    manual_box.append(&label_entry);
    manual_expander.set_child(Some(&manual_box));
    content.append(&manual_expander);

    // Edit selected group in place
    {
        use crate::dialogs::exercise_groups::show_manage_exercise_groups_dialog_with_selection;
        let state_for_edit = state.clone();
        let names_for_edit = names.clone();
        let alt_dropdown_for_btn = alt_dropdown.clone();
        edit_group_btn.connect_clicked(move |_| {
            let idx = alt_dropdown_for_btn.selected() as usize;
            if idx == 0 {
                // None
                show_manage_exercise_groups_dialog_with_selection(state_for_edit.clone(), None);
            } else {
                show_manage_exercise_groups_dialog_with_selection(
                    state_for_edit.clone(),
                    Some(names_for_edit[idx - 1].clone()),
                );
            }
        });
    }

    // Wire search selection to update fields
    let ex_entry_clone = ex_entry.clone();
    let label_entry_clone = label_entry.clone();
    let sw1 = search_widget.clone();
    let status_for_select = selected_status.clone();
    sw1.connect_row_selected(move |res| {
        ex_entry_clone.set_text(&res.code);
        label_entry_clone.set_text(&res.name);
        status_for_select.set_text(&format!("Selected: {} ({})", res.name, res.code));
    });
    let expander_on_select2 = search_expander.clone();
    let status_for_activate = selected_status.clone();
    search_widget.connect_row_activated(move |res| {
        status_for_activate.set_text(&format!("Selected: {} ({})", res.name, res.code));
        // Collapse the search dropdown immediately after activation (Enter/double-click)
        expander_on_select2.set_expanded(false);
    });

    dialog.content_area().append(&content);

    dialog.connect_response(clone!(@strong state, @strong ex_entry, @strong label_entry, @strong alt_dropdown, @strong sets_container => move |dialog, response| {
        if response == ResponseType::Accept {
            let ex_code = ex_entry.text().to_string();
            let ex_label = label_entry.text().to_string();
            let ex_label = if ex_label.trim().is_empty() { None } else { Some(ex_label) };
            let alt_group = {
                let idx = alt_dropdown.selected() as usize;
                if idx == 0 { None } else { Some(names[idx - 1].clone()) }
            };
            // Collect sets from container rows
            let mut rows_data: Vec<SchemeSetData> = Vec::new();
            let mut child = sets_container.first_child();
            while let Some(row) = child.clone() {
                if let Ok(hbox) = row.clone().downcast::<GtkBox>() {
                    // Extract widgets in known order
                    let mut w = hbox.first_child();
                    let mut widgets: Vec<gtk4::Widget> = Vec::new();
                    while let Some(cur) = w.clone() {
                        widgets.push(cur.clone());
                        w = cur.next_sibling();
                    }
                    // Expect roughly: [label_entry, sets, "reps:", reps_min, "-", reps_max, "time:", time, "RPE:", rpe, "Rest:", rest, remove]
                    let get_spin = |idx: usize| -> Option<f64> { widgets.get(idx).and_then(|w| w.clone().downcast::<gtk4::SpinButton>().ok()).map(|s| s.value()) };
                    let get_entry = |idx: usize| -> Option<String> { widgets.get(idx).and_then(|w| w.clone().downcast::<gtk4::Entry>().ok()).map(|e| e.text().to_string()) };
                    let label = get_entry(0).and_then(|s| if s.trim().is_empty() { None } else { Some(s) });
                    let sets = get_spin(1).map(|v| v as u32);
                    let reps_min = get_spin(3).map(|v| v as u32);
                    let reps_max = get_spin(5).map(|v| v as u32);
                    let time_sec = get_spin(7).map(|v| v as u32).filter(|v| *v > 0);
                    let rpe = get_spin(9);
                    let rest_sec = get_spin(11).map(|v| v as u32).filter(|v| *v > 0);
                    rows_data.push(SchemeSetData { label, sets, reps_min, reps_max, time_sec, rpe, rest_sec });
                }
                child = row.next_sibling();
            }
            update_scheme_full_in_plan(state.clone(), day_index, segment_index, ex_code, ex_label, alt_group, rows_data);
        }
        dialog.close();
    }));

    dialog.present();
}
