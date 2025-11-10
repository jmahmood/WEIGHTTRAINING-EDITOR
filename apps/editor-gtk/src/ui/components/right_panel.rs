use crate::dialogs::exercise_groups::show_manage_exercise_groups_dialog_with_selection;
use crate::state::AppState;
use glib::clone;
use gtk4::prelude::*;
use gtk4::{
    Box as GtkBox, Button, DropDown, Entry, GestureClick, Label, ListBox, ListBoxRow, Notebook,
    Orientation, ScrolledWindow,
};
use std::sync::{Arc, Mutex};
use weightlifting_core::AppPaths;

pub fn create_right_panel(state: Arc<Mutex<AppState>>, _paths: Arc<AppPaths>) -> GtkBox {
    let container = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .margin_start(6)
        .margin_end(6)
        .margin_top(6)
        .margin_bottom(6)
        .build();

    let notebook = Notebook::new();
    notebook.set_hexpand(true);
    notebook.set_vexpand(true);

    // Exercises tab
    let (exercises_tab, exercises_list) = create_exercises_tab(state.clone());
    let exercises_label = Label::new(Some("Exercises"));
    notebook.append_page(&exercises_tab, Some(&exercises_label));

    // Exercise Groups tab
    let (groups_tab, groups_list) = create_groups_tab(state.clone());
    let groups_label = Label::new(Some("Exercise Groups"));
    notebook.append_page(&groups_tab, Some(&groups_label));

    container.append(&notebook);

    // Periodically refresh lists when plan data changes
    use std::cell::Cell;
    use std::rc::Rc;
    let last_counts = Rc::new(Cell::new((0usize, 0usize))); // (dict_len, groups_len)
    let state_for_timer = state.clone();
    let list_for_timer_ex = exercises_list.clone();
    let list_for_timer_grp = groups_list.clone();
    glib::timeout_add_seconds_local(1, move || {
        let (dict_len, groups_len) = {
            let s = state_for_timer.lock().unwrap();
            if let Some(plan) = &s.current_plan {
                (plan.dictionary.len(), plan.groups.len())
            } else {
                (0, 0)
            }
        };
        let prev = last_counts.get();
        if prev != (dict_len, groups_len) {
            last_counts.set((dict_len, groups_len));
            populate_exercises_list(state_for_timer.clone(), &list_for_timer_ex);
            populate_groups_list(&state_for_timer.clone(), &list_for_timer_grp);
        }
        glib::ControlFlow::Continue
    });
    container
}

fn create_exercises_tab(state: Arc<Mutex<AppState>>) -> (GtkBox, ListBox) {
    let panel = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .margin_start(8)
        .margin_end(8)
        .margin_top(8)
        .margin_bottom(8)
        .build();

    // Header
    let header = Label::new(Some("Exercises"));
    header.set_css_classes(&["heading"]);
    header.set_halign(gtk4::Align::Start);
    panel.append(&header);

    // Grammar inputs
    let grammar_box = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(6)
        .build();

    // PATTERN and IMPLEMENT dropdowns with a Custom option
    let patterns: Vec<&str> = vec![
        "SQ",
        "BP",
        "DL",
        "OHP",
        "ROW",
        "PULLUP",
        "DIP",
        "HINGE",
        "LUNGE",
        "CALF",
        "CORE",
        "CARRY",
        "CURL",
        "EXT",
        "RAISE",
        "Custom…",
    ];
    let implements: Vec<&str> = vec![
        "BB",
        "DB",
        "KB",
        "BW",
        "CBL",
        "MACH",
        "SM",
        "SWISS",
        "TB",
        "SSB",
        "Custom…",
    ];
    let pattern_dd = DropDown::from_strings(&patterns);
    let implement_dd = DropDown::from_strings(&implements);
    pattern_dd.set_selected(0);
    implement_dd.set_selected(0);
    let pattern_custom = Entry::new();
    pattern_custom.set_placeholder_text(Some("Custom PATTERN (uppercase)"));
    pattern_custom.set_visible(false);
    let implement_custom = Entry::new();
    implement_custom.set_placeholder_text(Some("Custom IMPLEMENT (uppercase)"));
    implement_custom.set_visible(false);
    let variant_entry = Entry::new();
    variant_entry.set_placeholder_text(Some(
        "VARIANT (short uppercase code, e.g., FLAT, INCLINE, NARROW)",
    ));

    let name_entry = Entry::new();
    name_entry.set_placeholder_text(Some("Exercise Name (e.g., Bench Press)"));

    // Arrange grammar row
    let code_row = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(6)
        .build();
    let pattern_box = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(4)
        .build();
    pattern_box.append(&pattern_dd);
    pattern_box.append(&pattern_custom);
    code_row.append(&pattern_box);
    code_row.append(&Label::new(Some(".")));
    let implement_box = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(4)
        .build();
    implement_box.append(&implement_dd);
    implement_box.append(&implement_custom);
    code_row.append(&implement_box);
    code_row.append(&Label::new(Some(".")));
    code_row.append(&variant_entry);

    grammar_box.append(&Label::new(Some(
        "New Exercise (PATTERN.IMPLEMENT.VARIANT)",
    )));
    grammar_box.append(&code_row);
    grammar_box.append(&name_entry);

    let add_btn = Button::with_label("+ Add Exercise");
    add_btn.set_css_classes(&["suggested-action"]);
    grammar_box.append(&add_btn);
    panel.append(&grammar_box);

    // Exercises list
    let scrolled = ScrolledWindow::builder()
        .hexpand(true)
        .vexpand(true)
        .build();
    let list = ListBox::new();
    list.set_selection_mode(gtk4::SelectionMode::None);
    scrolled.set_child(Some(&list));
    panel.append(&scrolled);

    populate_exercises_list(state.clone(), &list);

    // Toggle custom entry visibility when selecting "Custom…"
    {
        let pattern_custom = pattern_custom.clone();
        let patterns_len = patterns.len();
        pattern_dd.connect_selected_notify(move |dd| {
            let sel = dd.selected() as usize;
            let is_custom = sel + 1 == patterns_len; // last index
            pattern_custom.set_visible(is_custom);
            if is_custom {
                pattern_custom.grab_focus();
            }
        });
    }
    {
        let implement_custom = implement_custom.clone();
        let implements_len = implements.len();
        implement_dd.connect_selected_notify(move |dd| {
            let sel = dd.selected() as usize;
            let is_custom = sel + 1 == implements_len;
            implement_custom.set_visible(is_custom);
            if is_custom {
                implement_custom.grab_focus();
            }
        });
    }

    // Add handler for add button
    add_btn.connect_clicked(clone!(@strong state, @strong pattern_dd, @strong implement_dd, @strong pattern_custom, @strong implement_custom, @strong variant_entry, @strong name_entry, @strong list => move |_| {
        let sel_pat = pattern_dd.selected() as usize;
        let sel_impl = implement_dd.selected() as usize;
        let patterns_vec: Vec<String> = vec![
            "SQ","BP","DL","OHP","ROW","PULLUP","DIP","HINGE","LUNGE","CALF","CORE","CARRY","CURL","EXT","RAISE","Custom…"
        ].into_iter().map(|s| s.to_string()).collect();
        let implements_vec: Vec<String> = vec![
            "BB","DB","KB","BW","CBL","MACH","SM","SWISS","TB","SSB","Custom…"
        ].into_iter().map(|s| s.to_string()).collect();
        let pattern = if sel_pat + 1 == patterns_vec.len() {
            pattern_custom.text().to_string()
        } else { patterns_vec[sel_pat].clone() };
        let implement = if sel_impl + 1 == implements_vec.len() {
            implement_custom.text().to_string()
        } else { implements_vec[sel_impl].clone() };
        let pattern = pattern.trim().to_uppercase();
        let implement = implement.trim().to_uppercase();
        let variant = variant_entry.text().to_string().trim().to_uppercase();
        let name = name_entry.text().to_string().trim().to_string();

        if pattern.is_empty() || implement.is_empty() || variant.is_empty() {
            show_error_dialog("Please enter PATTERN, IMPLEMENT, and VARIANT.");
            return;
        }
        if name.is_empty() {
            show_error_dialog("Please enter an exercise name.");
            return;
        }

        let mut ex_code = format!("{}.{}.{}", pattern, implement, variant);

        // Validate against current plan
        let (exists_code, exists_name) = {
            let s = state.lock().unwrap();
            if let Some(plan) = &s.current_plan {
                (
                    plan.dictionary.contains_key(&ex_code),
                    plan.dictionary.values().any(|n| n == &name),
                )
            } else {
                show_error_dialog("Open or create a plan to add exercises.");
                return;
            }
        };

        if exists_name {
            show_error_dialog("An exercise with this name already exists in this plan.");
            return;
        }

        if exists_code {
            // warn and propose suffixing the variant
            let new_variant = generate_unique_variant(&state, &pattern, &implement, &variant);
            let new_code = format!("{}.{}.{}", pattern, implement, new_variant);
            let proceed = confirm_dialog(&format!(
                "Exercise code already exists ({}).\nSave as {} instead?",
                ex_code, new_code
            ));
            if !proceed {
                return;
            }
            ex_code = new_code;
        }

        // insert
        {
            let mut s = state.lock().unwrap();
            s.save_to_undo_history();
            if let Some(plan) = &mut s.current_plan {
                plan.dictionary.insert(ex_code.clone(), name.clone());
            }
            s.mark_modified();
        }

        // Clear inputs and refresh list
        variant_entry.set_text("");
        name_entry.set_text("");
        populate_exercises_list(state.clone(), &list);
    }));

    (panel, list)
}

fn populate_exercises_list(state: Arc<Mutex<AppState>>, list: &ListBox) {
    while let Some(child) = list.first_child() {
        list.remove(&child);
    }
    let s = state.lock().unwrap();
    if let Some(plan) = &s.current_plan {
        let mut items: Vec<(String, String)> = plan
            .dictionary
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        items.sort_by(|a, b| a.0.cmp(&b.0));
        for (code, name) in items {
            let row = ListBoxRow::new();
            let box_ = GtkBox::builder()
                .orientation(Orientation::Horizontal)
                .spacing(10)
                .margin_start(10)
                .margin_end(10)
                .margin_top(6)
                .margin_bottom(6)
                .build();
            let code_lbl = Label::new(Some(&code));
            code_lbl.add_css_class("monospace");
            code_lbl.set_width_chars(18);
            let name_lbl = Label::new(Some(&name));
            name_lbl.set_hexpand(true);
            box_.append(&code_lbl);
            box_.append(&name_lbl);
            row.set_child(Some(&box_));
            list.append(&row);
        }
    }
}

fn create_groups_tab(state: Arc<Mutex<AppState>>) -> (GtkBox, ListBox) {
    let panel = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .margin_start(8)
        .margin_end(8)
        .margin_top(8)
        .margin_bottom(8)
        .build();

    let header = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .build();
    let title = Label::new(Some("Exercise Groups"));
    title.set_css_classes(&["heading"]);
    title.set_halign(gtk4::Align::Start);
    title.set_hexpand(true);
    let manage_btn = Button::with_label("Manage...");
    header.append(&title);
    header.append(&manage_btn);
    panel.append(&header);

    let scrolled = ScrolledWindow::builder()
        .hexpand(true)
        .vexpand(true)
        .build();
    let list = ListBox::new();
    list.set_selection_mode(gtk4::SelectionMode::Single);
    scrolled.set_child(Some(&list));
    panel.append(&scrolled);

    populate_groups_list(&state, &list);

    // Right-click to show context menu with Edit
    let gesture = GestureClick::new();
    gesture.set_button(gtk4::gdk::BUTTON_SECONDARY);
    gesture.connect_pressed(clone!(@strong state, @strong list => move |_g, n_press, x, y| {
        if n_press == 1 {
            if let Some(row) = list.row_at_y(y as i32) {
                list.select_row(Some(&row));
                if let Some(container) = row.child() {
                    if let Some(container) = container.downcast_ref::<GtkBox>() {
                        if let Some(name_lbl) = container.first_child().and_then(|c| c.downcast::<Label>().ok()) {
                            let group_name = name_lbl.text().to_string();
                            crate::ui::util::show_edit_context_menu(
                                &list,
                                x,
                                y,
                                std::boxed::Box::new(clone!(@strong state, @strong group_name => move || {
                                    crate::dialogs::exercise_groups::show_edit_exercise_group_dialog_simple(state.clone(), group_name.clone());
                                }))
                            );
                        }
                    }
                }
            }
        }
    }));
    list.add_controller(gesture);

    // Manage button opens full dialog
    manage_btn.connect_clicked(clone!(@strong state => move |_| {
        show_manage_exercise_groups_dialog_with_selection(state.clone(), None);
    }));

    (panel, list)
}

fn populate_groups_list(state: &Arc<Mutex<AppState>>, list: &ListBox) {
    while let Some(child) = list.first_child() {
        list.remove(&child);
    }
    let s = state.lock().unwrap();
    if let Some(plan) = &s.current_plan {
        let mut items: Vec<(String, usize)> = plan
            .groups
            .iter()
            .map(|(k, v)| (k.clone(), v.len()))
            .collect();
        items.sort_by(|a, b| a.0.cmp(&b.0));
        for (name, count) in items {
            let row = ListBoxRow::new();
            let box_ = GtkBox::builder()
                .orientation(Orientation::Vertical)
                .spacing(4)
                .margin_start(10)
                .margin_end(10)
                .margin_top(6)
                .margin_bottom(6)
                .build();
            let name_lbl = Label::new(Some(&name));
            name_lbl.set_halign(gtk4::Align::Start);
            name_lbl.set_css_classes(&["heading"]);
            let count_lbl = Label::new(Some(&format!("{} exercises", count)));
            count_lbl.set_halign(gtk4::Align::Start);
            count_lbl.set_css_classes(&["dim-label"]);
            box_.append(&name_lbl);
            box_.append(&count_lbl);
            row.set_child(Some(&box_));
            list.append(&row);
        }
    }
}

fn show_error_dialog(message: &str) {
    use gtk4::{ButtonsType, DialogFlags, MessageDialog, MessageType};
    let dialog = MessageDialog::new(
        crate::ui::util::parent_for_dialog().as_ref(),
        DialogFlags::MODAL,
        MessageType::Error,
        ButtonsType::Ok,
        message,
    );
    crate::ui::util::standardize_dialog(&dialog);
    dialog.connect_response(|d, _| d.close());
    dialog.present();
}

fn confirm_dialog(message: &str) -> bool {
    use gtk4::{ButtonsType, DialogFlags, MessageDialog, MessageType, ResponseType};
    let dlg = MessageDialog::new(
        crate::ui::util::parent_for_dialog().as_ref(),
        DialogFlags::MODAL,
        MessageType::Question,
        ButtonsType::YesNo,
        message,
    );
    crate::ui::util::standardize_dialog(&dlg);
    let decided = std::rc::Rc::new(std::cell::Cell::new(false));
    let done = std::rc::Rc::new(std::cell::Cell::new(false));
    let decided_c = decided.clone();
    let done_c = done.clone();
    dlg.connect_response(move |d, resp| {
        decided_c.set(resp == ResponseType::Yes);
        done_c.set(true);
        d.close();
    });
    dlg.present();
    let ctx = glib::MainContext::default();
    while !done.get() {
        ctx.iteration(true);
    }
    decided.get()
}

fn generate_unique_variant(
    state: &Arc<Mutex<AppState>>,
    pattern: &str,
    implement: &str,
    variant: &str,
) -> String {
    let mut suffix = 2u32;
    loop {
        let candidate = format!("{}{}", variant, suffix);
        let code = format!("{}.{}.{}", pattern, implement, candidate);
        let exists = {
            let s = state.lock().unwrap();
            if let Some(plan) = &s.current_plan {
                plan.dictionary.contains_key(&code)
            } else {
                false
            }
        };
        if !exists {
            return candidate;
        }
        suffix += 1;
    }
}
