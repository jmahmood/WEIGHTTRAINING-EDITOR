use gtk4::{Dialog, DialogFlags, ResponseType, Box as GtkBox, Orientation, Label, Button, Entry, ComboBoxText};
use gtk4::prelude::*;
use glib::clone;
use std::sync::{Arc, Mutex};
use weightlifting_core::AppPaths;
use weightlifting_validate::PlanValidator;

use crate::state::AppState;
use crate::operations::fixes::{replace_exercise_references, rename_exercise_code, ensure_dictionary_entry};

pub fn show_validation_dialog(state: Arc<Mutex<AppState>>, _paths: Arc<AppPaths>) {
    let has_plan = {
        let s = state.lock().unwrap();
        s.current_plan.is_some()
    };
    if !has_plan {
        crate::ui::plan::show_no_plan_error_dialog("validate");
        return;
    }

    let dialog = Dialog::with_buttons(
        Some("Validate Plan"),
        crate::ui::util::parent_for_dialog().as_ref(),
        DialogFlags::MODAL,
        &[("Close", ResponseType::Close)]
    );
    crate::ui::util::standardize_dialog(&dialog);

    let content = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .margin_start(20).margin_end(20).margin_top(20).margin_bottom(20)
        .spacing(10)
        .build();

    let header = Label::builder().label("Issues found. Click Fix to resolve common problems before saving.").build();
    content.append(&header);

    let list = GtkBox::builder().orientation(Orientation::Vertical).spacing(6).build();
    repopulate_validation(&list, &header, state.clone());
    content.append(&list);

    dialog.content_area().append(&content);
    dialog.connect_response(|d, _| d.close());
    dialog.present();
}

fn repopulate_validation(list: &GtkBox, header: &Label, state: Arc<Mutex<AppState>>) {
    // Clear existing rows
    while let Some(ch) = list.first_child() { list.remove(&ch); }
    let validator = PlanValidator::new().expect("validator");
    let res = {
        let s = state.lock().unwrap();
        s.current_plan.as_ref().map(|p| validator.validate(p))
    };
    if let Some(res) = res {
        if res.errors.is_empty() {
            header.set_label("No errors found. You can close this dialog.");
            list.append(&Label::new(Some("✓ Clean: no validation errors.")));
            return;
        }
        for err in res.errors {
            let row = GtkBox::builder().orientation(Orientation::Horizontal).spacing(8).build();
            let msg = Label::builder().label(format!("{} @ {}", err.message, err.path)).halign(gtk4::Align::Start).hexpand(true).build();
            row.append(&msg);
            match err.code.as_str() {
                "E102" => {
                    let bad_code = err.hint.clone();
                    let fix_btn = Button::with_label("Fix…");
                    let st = state.clone();
                    let list_c = list.clone();
                    let header_c = header.clone();
                    fix_btn.connect_clicked(move |_| {
                        show_fix_unknown_ex_dialog_refresh(st.clone(), None, bad_code.clone(), &list_c, &header_c);
                    });
                    row.append(&fix_btn);
                }
                "E103" => {
                    let group_name = err.field.clone();
                    let bad_code = err.hint.clone();
                    let fix_btn = Button::with_label("Fix…");
                    let st = state.clone();
                    let list_c = list.clone();
                    let header_c = header.clone();
                    fix_btn.connect_clicked(move |_| {
                        show_fix_unknown_ex_dialog_refresh(st.clone(), group_name.clone(), bad_code.clone(), &list_c, &header_c);
                    });
                    row.append(&fix_btn);
                }
                "E105" => {
                    let bad_code = err.field.clone().unwrap_or_default();
                    let fix_btn = Button::with_label("Rename…");
                    let st = state.clone();
                    let list_c = list.clone();
                    let header_c = header.clone();
                    fix_btn.connect_clicked(move |_| {
                        show_rename_code_dialog_refresh(st.clone(), &bad_code, &list_c, &header_c);
                    });
                    row.append(&fix_btn);
                }
                _ => {}
            }
            list.append(&row);
        }
    }
}

fn show_fix_unknown_ex_dialog_refresh(state: Arc<Mutex<AppState>>, group_name: Option<String>, bad_code: Option<String>, list: &GtkBox, header: &Label) {
    // Small dialog offering replace-with-existing or add-to-dictionary
    let dlg = Dialog::with_buttons(
        Some("Fix Unknown Exercise"),
        crate::ui::util::parent_for_dialog().as_ref(),
        DialogFlags::MODAL,
        &[("Cancel", ResponseType::Cancel), ("Apply", ResponseType::Accept)]
    );
    crate::ui::util::standardize_dialog(&dlg);

    let content = GtkBox::builder().orientation(Orientation::Vertical).spacing(8).margin_start(16).margin_end(16).margin_top(16).margin_bottom(16).build();

    let replace_row = GtkBox::builder().orientation(Orientation::Horizontal).spacing(6).build();
    replace_row.append(&Label::new(Some("Replace with existing:")));
    let dd = ComboBoxText::new();
    {
        let s = state.lock().unwrap();
        if let Some(plan) = &s.current_plan {
            let mut keys: Vec<_> = plan.dictionary.keys().cloned().collect();
            keys.sort();
            for k in keys { dd.append_text(&k); }
        }
    }
    dd.set_hexpand(true);
    replace_row.append(&dd);
    content.append(&replace_row);

    let add_row = GtkBox::builder().orientation(Orientation::Horizontal).spacing(6).build();
    add_row.append(&Label::new(Some("Or add code + name:")));
    let code_entry = Entry::new(); code_entry.set_placeholder_text(Some("CODE.SUBCODE[.VARIANT]")); code_entry.set_width_chars(24);
    if let Some(b) = &bad_code { code_entry.set_text(b); }
    let name_entry = Entry::new(); name_entry.set_placeholder_text(Some("Display Name")); name_entry.set_hexpand(true);
    add_row.append(&code_entry); add_row.append(&name_entry);
    content.append(&add_row);

    dlg.content_area().append(&content);
    crate::ui::util::bind_ctrl_enter_to_accept(&dlg);

    let list_c = list.clone();
    let header_c = header.clone();
    dlg.connect_response(clone!(@strong state, @strong dd, @strong code_entry, @strong name_entry, @strong group_name, @strong bad_code, @strong list_c, @strong header_c => move |d, resp| {
        if resp == ResponseType::Accept {
            if let Some(sel) = dd.active_text() {
                // If we know the bad code, replace its references with selected code
                if let Some(bad) = &bad_code { replace_exercise_references(state.clone(), bad, &sel); }
                // If scoped to a group without a specific code (fallback), replace unknowns with selected
                else if let Some(gname) = &group_name {
                    let mut s = state.lock().unwrap();
                    if let Some(plan) = &mut s.current_plan {
                        if let Some(members) = plan.groups.get_mut(gname) {
                            for mem in members.iter_mut() { if !plan.dictionary.contains_key(mem) { *mem = sel.to_string(); } }
                            s.mark_modified();
                        }
                    }
                }
            } else {
                let code = code_entry.text().to_string();
                let name = name_entry.text().to_string();
                if !code.is_empty() && !name.is_empty() {
                    ensure_dictionary_entry(state.clone(), &code, &name);
                }
            }
            repopulate_validation(&list_c, &header_c, state.clone());
        }
        d.close();
    }));
    dlg.present();
}

fn show_rename_code_dialog_refresh(state: Arc<Mutex<AppState>>, bad_code: &str, list: &GtkBox, header: &Label) {
    let dlg = Dialog::with_buttons(
        Some("Rename Exercise Code"),
        crate::ui::util::parent_for_dialog().as_ref(),
        DialogFlags::MODAL,
        &[("Cancel", ResponseType::Cancel), ("Rename", ResponseType::Accept)]
    );
    crate::ui::util::standardize_dialog(&dlg);
    let row = GtkBox::builder().orientation(Orientation::Horizontal).spacing(8).margin_start(16).margin_end(16).margin_top(16).margin_bottom(16).build();
    row.append(&Label::new(Some(&format!("{} →", bad_code))));
    let entry = Entry::new(); entry.set_placeholder_text(Some("NEW.CODE")); entry.set_width_chars(24);
    row.append(&entry);
    dlg.content_area().append(&row);
    crate::ui::util::bind_ctrl_enter_to_accept(&dlg);
    let bad = bad_code.to_string();
    let list_c = list.clone();
    let header_c = header.clone();
    dlg.connect_response(move |d, resp| {
        if resp == ResponseType::Accept {
            let new_code = entry.text().to_string();
            if !new_code.is_empty() {
                rename_exercise_code(state.clone(), &bad, &new_code);
                repopulate_validation(&list_c, &header_c, state.clone());
            }
        }
        d.close();
    });
    dlg.present();
}
