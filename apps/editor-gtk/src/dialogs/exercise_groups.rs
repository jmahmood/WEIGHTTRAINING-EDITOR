use crate::state::AppState;
use crate::ui::widgets::ExerciseSearchWidget;
use crate::operations::groups::{create_group, delete_group, validate_group_exercises, update_group, group_exists};
use gtk4::{Dialog, DialogFlags, ResponseType, Box as GtkBox, Orientation, Label, Entry, 
    Button, ListBox, ListBoxRow, ScrolledWindow, Paned, 
    SelectionMode, MessageDialog, MessageType, ButtonsType};
use gtk4::prelude::*;
use glib::clone;
use std::sync::{Arc, Mutex};

pub fn show_manage_exercise_groups_dialog(state: Arc<Mutex<AppState>>) {
    show_manage_exercise_groups_dialog_with_selection(state, None);
}

pub fn show_manage_exercise_groups_dialog_with_selection(state: Arc<Mutex<AppState>>, select_group: Option<String>) {
    let dialog = Dialog::builder()
        .title("Manage Exercise Groups")
        .modal(true)
        .default_width(800)
        .default_height(600)
        .build();
    if let Some(parent) = crate::ui::util::parent_for_dialog() {
        dialog.set_transient_for(Some(&parent));
        dialog.set_destroy_with_parent(true);
    }
    crate::ui::util::standardize_dialog(&dialog);
    // Create main layout - horizontal paned
    let paned = Paned::new(Orientation::Horizontal);
    paned.set_position(300);
    
    // Left panel - Groups list
    let (left_panel, groups_list) = create_groups_list_panel(state.clone());
    paned.set_start_child(Some(&left_panel));
    
    use std::rc::Rc;
    use std::cell::Cell;
    let dirty = Rc::new(Cell::new(false));
    let current_group = Rc::new(std::cell::RefCell::new(String::new()));
    let reverting_selection = Rc::new(Cell::new(false));
    let is_populating = Rc::new(Cell::new(false));

    // Right panel - Group editor
    let (right_panel, name_entry, members_list, search_widget, save_btn) = create_group_editor_panel(state.clone(), groups_list.clone(), dirty.clone());
    paned.set_end_child(Some(&right_panel));
    
    // When a group is selected in the left list, populate the editor
    // Mark dirty when editing name
    {
        let dirty = dirty.clone();
        let is_populating = is_populating.clone();
        name_entry.connect_changed(move |_| {
            if !is_populating.get() {
                dirty.set(true);
            }
        });
    }

    // Load group details on activation
    groups_list.connect_row_activated(clone!(@strong state, @strong groups_list, @strong name_entry, @strong members_list, @strong search_widget, @strong dirty, @strong current_group, @strong is_populating => move |_, row| {
        dirty.set(false);
        if let Some(box_) = row.child() {
            if let Some(box_) = box_.downcast_ref::<GtkBox>() {
                if let Some(name_label) = box_.first_child().and_then(|c| c.downcast::<Label>().ok()) {
                    let group_name = name_label.text().to_string();
                    // Update current group safely
                    {
                        let mut cg = current_group.borrow_mut();
                        *cg = group_name.clone();
                    }
                    // Populate without triggering dirty
                    is_populating.set(true);
                    name_entry.set_text(&group_name);
                    while let Some(child) = members_list.first_child() {
                        members_list.remove(&child);
                    }
                    let state_lock = state.lock().unwrap();
                    if let Some(plan) = &state_lock.current_plan {
                        if let Some(exs) = plan.groups.get(&group_name) {
                            // Fill members and set search exclusions to current members
                            for code in exs {
                                let display_name = plan.dictionary.get(code).map(|s| s.as_str()).unwrap_or(code.as_str());
                                add_exercise_to_members_list(&members_list, code, display_name, None);
                            }
                            search_widget.set_excluded_codes(&exs.clone());
                            search_widget.refresh_results();
                        }
                    }
                    search_widget.clear_search();
                    is_populating.set(false);
                }
            }
        }
    }));

    // Mouse/selection change guard with unsaved confirmation
    groups_list.connect_row_selected(clone!(@strong dialog, @strong groups_list, @strong dirty, @strong current_group, @strong reverting_selection => move |list, row_opt| {
        if reverting_selection.get() { return; }
        if let Some(row) = row_opt {
            // Determine selected group's name
            let mut target_name: Option<String> = None;
            if let Some(box_) = row.child() {
                if let Some(box_) = box_.downcast_ref::<GtkBox>() {
                    if let Some(name_label) = box_.first_child().and_then(|c| c.downcast::<Label>().ok()) {
                        target_name = Some(name_label.text().to_string());
                    }
                }
            }
            if let Some(tname) = target_name {
                let prev_name = { current_group.borrow().clone() };
                if dirty.get() && prev_name != tname {
                    if !confirm_discard_for(&dialog) {
                        // Revert selection to previous
                        reverting_selection.set(true);
                        select_and_activate_group(&groups_list, &prev_name);
                        reverting_selection.set(false);
                        return;
                    }
                    dirty.set(false);
                    // Activate the selected row to load details
                    list.emit_by_name::<()>("row-activated", &[row]);
                }
            }
        }
    }));
    
    dialog.content_area().append(&paned);
    
    // Add dialog buttons
    dialog.add_buttons(&[("Close", ResponseType::Close)]);
    crate::ui::util::standardize_dialog(&dialog);
    // Handle Close button respecting unsaved changes
    {
        let dirty = dirty.clone();
    dialog.connect_response(move |dialog, response| {
            if response == ResponseType::Close {
                if dirty.get() {
                    if confirm_discard_for(dialog) {
                        dirty.set(false);
                        dialog.close();
                        dialog.hide();
                    } else {
                        // Keep dialog open
                    }
                } else {
                    dialog.close();
                    dialog.hide();
                }
            } else {
                dialog.close();
                dialog.hide();
            }
        });
    }
    
    // Ensure initial keyboard focus on the groups list for accessibility
    {
        let gl = groups_list.clone();
        glib::idle_add_local_once(move || {
            gl.grab_focus();
        });
    }

    // Hook up Save button and Ctrl+Enter, with selection guard to avoid discard prompts during programmatic refresh
    {
        let dlg_ref = dialog.clone();
        let gl_ref = groups_list.clone();
        let dirty_ref = dirty.clone();
        let reverting_ref = reverting_selection.clone();
        let name_ref = name_entry.clone();
        let members_ref = members_list.clone();
        let state_ref = state.clone();
        save_btn.connect_clicked(clone!(@strong state_ref, @strong name_ref, @strong members_ref, @strong gl_ref, @strong dirty_ref, @strong reverting_ref, @strong current_group => move |_| {
            // Pre-compute exercise count and whether the row already exists
            let group_name = name_ref.text().to_string();
            let mut count = 0usize;
            let mut child = members_ref.first_child();
            while let Some(row) = child {
                count += 1;
                child = row.next_sibling();
            }
            let _existed_before = group_exists(state_ref.clone(), &group_name);

            if save_current_group(state_ref.clone(), &name_ref, &members_ref, &gl_ref) {
                // Mark clean
                dirty_ref.set(false);
                // Update current group tracker
                {
                    let mut cg = current_group.borrow_mut();
                    *cg = group_name.clone();
                }
                // Update or insert the row without repopulating the entire list
                reverting_ref.set(true);
                upsert_group_row(&gl_ref, &group_name, count);
                // Keep current selection as-is; do not emit row-activated here to avoid reentrancy
                reverting_ref.set(false);
            }
        }));
        // Global Ctrl+Enter to save group
        crate::ui::util::bind_ctrl_enter_to_button(&dlg_ref, &save_btn);
    }

    // Warn on close if unsaved changes
    // Ensure window manager close (Esc/titlebar X) respects unsaved-changes
    dialog.connect_close_request(clone!(@strong dialog, @strong dirty => @default-return glib::Propagation::Proceed, move |_| {
        if dirty.get() {
            if confirm_discard_for(&dialog) {
                dirty.set(false);
                glib::Propagation::Proceed
            } else {
                glib::Propagation::Stop
            }
        } else {
            glib::Propagation::Proceed
        }
    }));

    // Keyboard control for groups list with dirty guard
    {
        let gl_ref = groups_list.clone();
        let dirty = dirty.clone();
        let _current_group = current_group.clone();
        let parent_dialog = dialog.clone();
        let key = gtk4::EventControllerKey::new();
        key.set_propagation_phase(gtk4::PropagationPhase::Capture);
        key.connect_key_pressed(move |_, keyval, _, _| {
            let navigate = |next: bool| {
                let (sel, neighbor) = if let Some(sel) = gl_ref.selected_row() {
                    let neigh = if next { sel.next_sibling() } else { sel.prev_sibling() };
                    (Some(sel), neigh.and_then(|w| w.downcast::<ListBoxRow>().ok()))
                } else {
                    (None, gl_ref.row_at_index(0))
                };
                if let Some(target_row) = neighbor.or(sel) {
                    if dirty.get() {
                        if !confirm_discard_for(&parent_dialog) { return; }
                        dirty.set(false);
                    }
                    gl_ref.select_row(Some(&target_row));
                    gl_ref.emit_by_name::<()>("row-activated", &[&target_row]);
                }
            };
            match keyval {
                gtk4::gdk::Key::Down => { navigate(true); glib::Propagation::Stop },
                gtk4::gdk::Key::Up => { navigate(false); glib::Propagation::Stop },
                gtk4::gdk::Key::Return | gtk4::gdk::Key::KP_Enter => {
                    if dirty.get() && !confirm_discard_for(&parent_dialog) {
                        return glib::Propagation::Stop;
                    }
                    if let Some(sel) = gl_ref.selected_row() {
                        gl_ref.emit_by_name::<()>("row-activated", &[&sel]);
                        glib::Propagation::Stop
                    } else { glib::Propagation::Proceed }
                },
                _ => glib::Propagation::Proceed,
            }
        });
        groups_list.add_controller(key);
    }

    // If a target group is specified, select and activate it after the dialog shows
    if let Some(target) = select_group.clone() {
        let gl = groups_list.clone();
        glib::idle_add_local_once(move || {
            select_and_activate_group(&gl, &target);
        });
    }

    dialog.present();
}

/// Open a simplified editor for a single exercise group without listing all groups.
/// The dialog shows only the editor for `group_name` and allows saving or deleting.
pub fn show_edit_exercise_group_dialog_simple(state: Arc<Mutex<AppState>>, group_name: String) {
    let dialog = Dialog::builder()
        .title("Edit Exercise Group")
        .modal(true)
        .default_width(600)
        .default_height(500)
        .build();
    if let Some(parent) = crate::ui::util::parent_for_dialog() {
        dialog.set_transient_for(Some(&parent));
        dialog.set_destroy_with_parent(true);
    }
    crate::ui::util::standardize_dialog(&dialog);

    use std::cell::Cell;
    use std::rc::Rc;
    let dirty = Rc::new(Cell::new(false));

    // Build the right-side editor panel, passing a dummy list for API compatibility
    let dummy_list = ListBox::new();
    let (right_panel, name_entry, members_list, search_widget, save_btn) =
        create_group_editor_panel(state.clone(), dummy_list.clone(), dirty.clone());

    // Populate editor with the specified group
    {
        let s = state.lock().unwrap();
        if let Some(plan) = &s.current_plan {
            // Set name
            name_entry.set_text(&group_name);
            // Fill members and set search exclusions to current members
            if let Some(exs) = plan.groups.get(&group_name) {
                while let Some(child) = members_list.first_child() { members_list.remove(&child); }
                for code in exs {
                    let display_name = plan.dictionary.get(code).map(|s| s.as_str()).unwrap_or(code.as_str());
                    add_exercise_to_members_list(&members_list, code, display_name, None);
                }
                search_widget.set_excluded_codes(&exs.clone());
                search_widget.refresh_results();
            }
            search_widget.clear_search();
        }
    }

    dialog.content_area().append(&right_panel);
    dialog.add_buttons(&[("Close", ResponseType::Close)]);

    // Save handler: save current group and keep dialog open, clear dirty flag
    {
        let name_ref = name_entry.clone();
        let members_ref = members_list.clone();
        let state_ref = state.clone();
        let dirty_ref = dirty.clone();
        let dlg_ref = dialog.clone();
        save_btn.connect_clicked(clone!(@strong state_ref, @strong name_ref, @strong members_ref, @strong dirty_ref, @strong dlg_ref => move |_| {
            if save_current_group(state_ref.clone(), &name_ref, &members_ref, &dummy_list) {
                dirty_ref.set(false);
                dlg_ref.close();
            }
        }));
        crate::ui::util::bind_ctrl_enter_to_button(&dialog, &save_btn);
    }

    // Close handling: allow close; if future dirty handling is desired, use confirm_discard_for
    dialog.connect_response(|d, _| { d.close(); });
    dialog.connect_close_request(|_| glib::Propagation::Proceed);

    dialog.present();
}

/// Ask whether to discard unsaved changes, using `dialog` as the transient parent.
fn confirm_discard_for(parent: &Dialog) -> bool {
    use gtk4::{DialogFlags, MessageDialog, MessageType, ButtonsType, ResponseType};
    let dlg = MessageDialog::new(Some(parent), DialogFlags::MODAL, MessageType::Question, ButtonsType::YesNo, "Discard unsaved changes?");
    // Ensure it closes with parent
    dlg.set_destroy_with_parent(true);
    let decision = std::rc::Rc::new(std::cell::Cell::new(false));
    let done = std::rc::Rc::new(std::cell::Cell::new(false));
    let decision_clone = decision.clone();
    let done_clone = done.clone();
    dlg.connect_response(move |d, resp| {
        decision_clone.set(resp == ResponseType::Yes);
        done_clone.set(true);
        d.close();
    });
    dlg.present();
    let ctx = glib::MainContext::default();
    while !done.get() {
        ctx.iteration(true);
    }
    decision.get()
}

fn create_groups_list_panel(state: Arc<Mutex<AppState>>) -> (GtkBox, ListBox) {
    let panel = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .margin_start(10)
        .margin_end(5)
        .margin_top(10)
        .margin_bottom(10)
        .spacing(10)
        .build();
    
    // Header
    let header = Label::new(Some("Exercise Groups"));
    header.set_css_classes(&["heading"]);
    panel.append(&header);
    
    // New group button
    let new_group_btn = Button::with_label("+ New Group");
    new_group_btn.set_css_classes(&["suggested-action"]);
    panel.append(&new_group_btn);
    
    // Groups list
    let scrolled = ScrolledWindow::builder()
        .hexpand(true)
        .vexpand(true)
        .min_content_height(400)
        .build();
    
    let groups_list = ListBox::new();
    groups_list.set_selection_mode(SelectionMode::Single);
    groups_list.set_can_focus(true);
    scrolled.set_child(Some(&groups_list));
    panel.append(&scrolled);
    
    // Populate groups list
    populate_groups_list(state.clone(), &groups_list);
    
    // Connect new group button
    new_group_btn.connect_clicked(clone!(@strong state, @strong groups_list => move |_| {
        show_new_group_dialog(state.clone(), &groups_list);
    }));

    // Keyboard navigation with unsaved-changes guard
    // Handlers attached in the parent function to share state
    
    (panel, groups_list)
}

fn create_group_editor_panel(state: Arc<Mutex<AppState>>, groups_list: ListBox, dirty: std::rc::Rc<std::cell::Cell<bool>>) -> (GtkBox, Entry, ListBox, crate::ui::widgets::ExerciseSearchWidget, gtk4::Button) {
    let panel = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .margin_start(5)
        .margin_end(10)
        .margin_top(10)
        .margin_bottom(10)
        .spacing(10)
        .build();
    
    // Header
    let header = Label::new(Some("Group Details"));
    header.set_css_classes(&["heading"]);
    panel.append(&header);
    
    // Group name
    let name_label = Label::new(Some("Group Name:"));
    name_label.set_halign(gtk4::Align::Start);
    let name_entry = Entry::new();
    name_entry.set_placeholder_text(Some("e.g., GROUP_CHEST_PRESS"));
    
    panel.append(&name_label);
    panel.append(&name_entry);
    
    // Exercise search
    let search_label = Label::new(Some("Add Exercises:"));
    search_label.set_halign(gtk4::Align::Start);
    panel.append(&search_label);
    
    let search_widget = ExerciseSearchWidget::new();
    if let Err(e) = search_widget.set_database_path("/home/jawaad/weightlifting-desktop/exercises.db") {
        println!("Failed to connect to exercise database: {}", e);
    }
    search_widget.set_state(state.clone());
    panel.append(&search_widget.container);
    
    // Group members list
    let members_label = Label::new(Some("Group Members:"));
    members_label.set_halign(gtk4::Align::Start);
    panel.append(&members_label);
    
    let members_scrolled = ScrolledWindow::builder()
        .hexpand(true)
        .vexpand(true)
        .min_content_height(200)
        .build();
    
    let members_list = ListBox::new();
    members_list.set_selection_mode(SelectionMode::Single);
    members_list.set_can_focus(true);
    members_scrolled.set_child(Some(&members_list));
    panel.append(&members_scrolled);
    
    // Action buttons
    let button_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .halign(gtk4::Align::End)
        .build();
    
    let save_btn = Button::with_label("Save Group");
    save_btn.set_css_classes(&["suggested-action"]);
    
    let delete_btn = Button::with_label("Delete Group");
    delete_btn.set_css_classes(&["destructive-action"]);
    
    button_box.append(&delete_btn);
    button_box.append(&save_btn);
    panel.append(&button_box);
    
    // Connect search widget to add exercises
    let search_widget_clone = search_widget.clone();
    search_widget.connect_row_activated_with_import(state.clone(), clone!(@strong members_list, @strong search_widget_clone, @strong groups_list, @strong dirty => move |result| {
        add_exercise_to_members_list(&members_list, &result.code, &result.name, Some(search_widget_clone.clone()));
        dirty.set(true);
        // Mark as dirty on add
        // Note: dirty flag is managed in parent; we rely on Save to commit.
        // Exclude newly added code from search results
        search_widget_clone.add_excluded_code(&result.code);
        search_widget_clone.refresh_results();
    }));
    
    // Save button handler is connected in parent to coordinate selection guards
    
    // Connect delete button  
    delete_btn.connect_clicked(clone!(@strong state, @strong name_entry, @strong groups_list => move |_| {
        delete_current_group(state.clone(), &name_entry, &groups_list);
    }));
    
    // Keyboard: remove selected member with Delete/BackSpace
    {
        let ml = members_list.clone();
        let sw = search_widget.clone();
        let dirty = dirty.clone();
        let key = gtk4::EventControllerKey::new();
        key.set_propagation_phase(gtk4::PropagationPhase::Capture);
        key.connect_key_pressed(move |_, keyval, _, _| {
            match keyval {
                gtk4::gdk::Key::Delete | gtk4::gdk::Key::BackSpace | gtk4::gdk::Key::KP_Delete => {
                    if let Some(sel) = ml.selected_row() {
                        // Capture code label to re-include in search
                        if let Some(child) = sel.child() {
                            if let Some(hbox) = child.downcast_ref::<GtkBox>() {
                                if let Some(code_lbl) = hbox.first_child().and_then(|c| c.downcast::<Label>().ok()) {
                                    sw.remove_excluded_code(&code_lbl.text());
                                    sw.refresh_results();
                                }
                            }
                        }
                        ml.remove(&sel);
                        dirty.set(true);
                        glib::Propagation::Stop
                    } else {
                        glib::Propagation::Proceed
                    }
                },
                _ => glib::Propagation::Proceed,
            }
        });
        members_list.add_controller(key);
    }

    (panel, name_entry, members_list, search_widget, save_btn)
}

fn populate_groups_list(state: Arc<Mutex<AppState>>, groups_list: &ListBox) {
    // Clear existing items
    while let Some(child) = groups_list.first_child() {
        groups_list.remove(&child);
    }
    
    // Get groups from current plan
    if let Some(plan) = &state.lock().unwrap().current_plan {
        for (group_name, exercises) in &plan.groups {
            let row = create_group_list_row(group_name, exercises.len());
            groups_list.append(&row);
        }
    }
}

fn create_group_list_row(name: &str, exercise_count: usize) -> ListBoxRow {
    let row = ListBoxRow::new();
    
    let box_ = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .margin_start(10)
        .margin_end(10)
        .margin_top(8)
        .margin_bottom(8)
        .spacing(4)
        .build();
    
    let name_label = Label::new(Some(name));
    name_label.set_halign(gtk4::Align::Start);
    name_label.set_css_classes(&["heading"]);
    
    let count_label = Label::new(Some(&format!("{} exercises", exercise_count)));
    count_label.set_halign(gtk4::Align::Start);
    count_label.set_css_classes(&["dim-label"]);
    
    box_.append(&name_label);
    box_.append(&count_label);
    row.set_child(Some(&box_));
    
    row
}

fn add_exercise_to_members_list(members_list: &ListBox, code: &str, name: &str, search_widget: Option<crate::ui::widgets::ExerciseSearchWidget>) {
    // Check if exercise already exists in list
    let mut child = members_list.first_child();
    while let Some(row) = child {
        if let Some(list_row) = row.downcast_ref::<ListBoxRow>() {
            if let Some(box_) = list_row.child() {
                if let Some(box_) = box_.downcast_ref::<GtkBox>() {
                    if let Some(first_child) = box_.first_child() {
                        if let Some(label) = first_child.downcast_ref::<Label>() {
                            if label.text() == code {
                                // Exercise already exists
                                return;
                            }
                        }
                    }
                }
            }
        }
        child = row.next_sibling();
    }
    
    // Create new member row
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
    
    // Connect remove button
    let code_for_remove = code.to_string();
    remove_btn.connect_clicked(clone!(@strong row, @strong members_list, @strong search_widget => move |_| {
        members_list.remove(&row);
        if let Some(w) = &search_widget {
            w.remove_excluded_code(&code_for_remove);
            w.refresh_results();
        }
    }));
    
    members_list.append(&row);
}

fn save_current_group(state: Arc<Mutex<AppState>>, name_entry: &Entry, members_list: &ListBox, _groups_list: &ListBox) -> bool {
    let group_name = name_entry.text().to_string().trim().to_string();
    
    if group_name.is_empty() {
        show_error_dialog("Please enter a group name");
        return false;
    }
    
    // Collect member exercises
    let mut exercises = Vec::new();
    let mut child = members_list.first_child();
    while let Some(row) = child {
        if let Some(list_row) = row.downcast_ref::<ListBoxRow>() {
            if let Some(box_) = list_row.child() {
                if let Some(box_) = box_.downcast_ref::<GtkBox>() {
                    if let Some(first_child) = box_.first_child() {
                        if let Some(code_label) = first_child.downcast_ref::<Label>() {
                            exercises.push(code_label.text().to_string());
                        }
                    }
                }
            }
        }
        child = row.next_sibling();
    }
    
    if exercises.is_empty() {
        show_error_dialog("Please add at least one exercise to the group");
        return false;
    }
    
    // Validate exercises exist in dictionary
    if let Err(e) = validate_group_exercises(state.clone(), &exercises) {
        show_error_dialog(&format!("Validation error: {}", e));
        return false;
    }
    
    // Save or update the group
    if group_exists(state.clone(), &group_name) {
        if let Err(e) = update_group(state.clone(), group_name.clone(), exercises) {
            show_error_dialog(&format!("Failed to update group: {}", e));
            return false;
        }
        // Keep dialog open and reflect saved state immediately
    } else if let Err(e) = create_group(state.clone(), group_name.clone(), exercises) {
        show_error_dialog(&format!("Failed to save group: {}", e));
        return false;
    }
    
    // Do not refresh selection here; caller coordinates UI refresh to avoid discard prompts.
    true
}

fn delete_current_group(state: Arc<Mutex<AppState>>, name_entry: &Entry, groups_list: &ListBox) {
    let group_name = name_entry.text().to_string().trim().to_string();
    
    if group_name.is_empty() {
        show_error_dialog("Please enter a group name to delete");
        return;
    }
    
    // Confirm deletion
    let dialog = MessageDialog::new(
        crate::ui::util::parent_for_dialog().as_ref(),
        DialogFlags::MODAL,
        MessageType::Question,
        ButtonsType::YesNo,
        format!("Are you sure you want to delete the group '{}'? This action cannot be undone.", group_name)
    );
    crate::ui::util::standardize_dialog(&dialog);
    
    dialog.connect_response(clone!(@strong state, @strong group_name, @strong groups_list => move |dialog, response| {
        if response == ResponseType::Yes {
            if let Err(e) = delete_group(state.clone(), group_name.clone()) {
                show_error_dialog(&format!("Failed to delete group: {}", e));
            } else {
                show_success_dialog(&format!("Group '{}' deleted successfully", group_name));
                populate_groups_list(state.clone(), &groups_list);
            }
        }
        dialog.close();
    }));
    
    dialog.present();
}

fn show_new_group_dialog(state: Arc<Mutex<AppState>>, groups_list: &ListBox) {
    let dialog = Dialog::with_buttons(
        Some("New Exercise Group"),
        crate::ui::util::parent_for_dialog().as_ref(),
        DialogFlags::MODAL,
        &[("Cancel", ResponseType::Cancel), ("Create", ResponseType::Accept)]
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
    
    let name_label = Label::new(Some("Group Name:"));
    let name_entry = Entry::new();
    name_entry.set_placeholder_text(Some("e.g., GROUP_CHEST_PRESS"));
    
    content.append(&name_label);
    content.append(&name_entry);
    
    dialog.content_area().append(&content);
    // Ctrl+Enter saves (Create)
    if let Some(ok_btn) = dialog.widget_for_response(ResponseType::Accept) {
        if let Ok(btn) = ok_btn.downcast::<gtk4::Button>() {
            crate::ui::util::bind_ctrl_enter_to_button(&dialog, &btn);
        }
    }
    
    dialog.connect_response(clone!(@strong state, @strong groups_list => move |dialog, response| {
        if response == ResponseType::Accept {
            let group_name = name_entry.text().to_string().trim().to_string();
            if !group_name.is_empty() {
                // Create empty group
                if let Err(e) = create_group(state.clone(), group_name, Vec::new()) {
                    show_error_dialog(&format!("Failed to create group: {}", e));
                } else {
                    // Refresh and select new group so user sees it immediately
                    populate_groups_list(state.clone(), &groups_list);
                    select_and_activate_group(&groups_list, &name_entry.text());
                }
            }
        }
        dialog.close();
    }));
    
    dialog.present();
}

/// Helper: select a group by name in the list and trigger the row-activated handler
fn select_and_activate_group(groups_list: &ListBox, group_name: &str) {
    let mut child = groups_list.first_child();
    while let Some(row) = child {
        if let Some(list_row) = row.downcast_ref::<ListBoxRow>() {
            if let Some(container) = list_row.child() {
                if let Some(container) = container.downcast_ref::<GtkBox>() {
                    if let Some(name_label) = container.first_child().and_then(|c| c.downcast::<Label>().ok()) {
                        if name_label.text() == group_name {
                            groups_list.select_row(Some(list_row));
                            // Emit row-activated to repopulate editor panel
                            groups_list.emit_by_name::<()>("row-activated", &[list_row]);
                            break;
                        }
                    }
                }
            }
        }
        child = row.next_sibling();
    }
}

/// Update the row for `group_name` with `exercise_count`, or insert it if missing.
fn upsert_group_row(groups_list: &ListBox, group_name: &str, exercise_count: usize) {
    let mut child = groups_list.first_child();
    while let Some(row) = child {
        if let Some(list_row) = row.downcast_ref::<ListBoxRow>() {
            if let Some(container) = list_row.child() {
                if let Some(container) = container.downcast_ref::<GtkBox>() {
                    // First child is name, second is count
                    let maybe_name = container.first_child().and_then(|c| c.downcast::<Label>().ok());
                    let maybe_count = maybe_name.as_ref().and_then(|_| container.last_child()).and_then(|c| c.downcast::<Label>().ok());
                    if let Some(name_lbl) = maybe_name {
                        if name_lbl.text() == group_name {
                            if let Some(count_lbl) = maybe_count {
                                count_lbl.set_text(&format!("{} exercises", exercise_count));
                            }
                            return;
                        }
                    }
                }
            }
        }
        child = row.next_sibling();
    }
    // Not found: append
    let new_row = create_group_list_row(group_name, exercise_count);
    groups_list.append(&new_row);
}

fn show_error_dialog(message: &str) {
    let dialog = MessageDialog::new(
        crate::ui::util::parent_for_dialog().as_ref(),
        DialogFlags::MODAL,
        MessageType::Error,
        ButtonsType::Ok,
        message
    );
    crate::ui::util::standardize_dialog(&dialog);
    dialog.connect_response(|dialog, _| dialog.close());
    dialog.present();
}

fn show_success_dialog(message: &str) {
    let dialog = MessageDialog::new(
        crate::ui::util::parent_for_dialog().as_ref(),
        DialogFlags::MODAL,
        MessageType::Info,
        ButtonsType::Ok,
        message
    );
    crate::ui::util::standardize_dialog(&dialog);
    dialog.connect_response(|dialog, _| dialog.close());
    dialog.present();
}
