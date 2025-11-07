// Exercise search widget for SQLite database integration
// Supports real-time search by name, code, and bodypart

use crate::state::AppState;
use gtk4::prelude::*;
use gtk4::{
    Box, Button, Entry, EventControllerKey, GestureClick, Label, ListBox, ListBoxRow, Orientation,
};
use rusqlite::{Connection, Result};
use std::collections::HashSet;
use std::path::Path;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct ExerciseSearchWidget {
    pub container: Box,
    search_entry: Entry,
    selected_name_label: Label,
    results_list: ListBox,
    db_connection: Arc<Mutex<Option<Connection>>>,
    plan_state: Arc<Mutex<Option<Arc<Mutex<AppState>>>>>,
    excluded_codes: Arc<Mutex<HashSet<String>>>,
}

#[derive(Debug)]
pub struct ExerciseResult {
    pub code: String,
    pub name: String,
}

impl ExerciseSearchWidget {
    pub fn new() -> Self {
        let search_entry = Entry::new();
        search_entry.set_placeholder_text(Some("Search exercises by name, code, or bodypart..."));

        // Label shown directly under the search field with the exercise name
        let selected_name_label = Label::new(None);
        selected_name_label.add_css_class("dim-label");
        selected_name_label.set_halign(gtk4::Align::Start);
        selected_name_label.set_wrap(true);
        selected_name_label.set_wrap_mode(gtk4::pango::WrapMode::WordChar);

        let results_list = ListBox::new();
        results_list.set_selection_mode(gtk4::SelectionMode::Single);

        let container = Box::new(Orientation::Vertical, 5);
        container.append(&search_entry);
        container.append(&selected_name_label);
        container.append(&results_list);

        let widget = Self {
            container,
            search_entry,
            selected_name_label,
            results_list,
            db_connection: Arc::new(Mutex::new(None)),
            plan_state: Arc::new(Mutex::new(None)),
            excluded_codes: Arc::new(Mutex::new(HashSet::new())),
        };

        widget.setup_signals();
        widget.setup_keyboard_navigation();
        widget
    }

    pub fn set_database_path(&self, path: &str) -> Result<()> {
        let conn = Connection::open(Path::new(path))?;
        *self.db_connection.lock().unwrap() = Some(conn);
        Ok(())
    }

    fn clear_results_list(list_box: &ListBox) {
        while let Some(child) = list_box.first_child() {
            list_box.remove(&child);
        }
    }

    fn setup_signals(&self) {
        let search_entry = self.search_entry.clone();
        let results_list = self.results_list.clone();
        let selected_name_label = self.selected_name_label.clone();
        let db_connection = self.db_connection.clone();
        let plan_state = self.plan_state.clone();
        let excluded_codes = self.excluded_codes.clone();

        search_entry.connect_changed(move |entry| {
            let query = entry.text().to_string();
            if query.is_empty() {
                Self::clear_results_list(&results_list);
                selected_name_label.set_text("");
                return;
            }

            let mut combined: Vec<ExerciseResult> = Vec::new();
            let mut seen_codes: HashSet<String> = HashSet::new();
            let excluded = excluded_codes.lock().unwrap().clone();
            if let Some(conn) = &*db_connection.lock().unwrap() {
                if let Ok(results) = Self::search_exercises(conn, &query) {
                    for r in results {
                        if excluded.contains(&r.code) {
                            continue;
                        }
                        seen_codes.insert(r.code.clone());
                        combined.push(r);
                    }
                }
            }
            if let Some(state_arc) = &*plan_state.lock().unwrap() {
                let state_lock = state_arc.lock().unwrap();
                if let Some(plan) = &state_lock.current_plan {
                    let q = query.to_lowercase();
                    for (code, name) in &plan.dictionary {
                        if seen_codes.contains(code) {
                            continue;
                        }
                        if excluded.contains(code) {
                            continue;
                        }
                        if code.to_lowercase().contains(&q) || name.to_lowercase().contains(&q) {
                            combined.push(ExerciseResult {
                                code: code.clone(),
                                name: name.clone(),
                            });
                        }
                    }
                }
            }
            Self::clear_results_list(&results_list);
            for (index, result) in combined.iter().enumerate() {
                let row = Self::create_result_row(result);
                results_list.append(&row);

                // Auto-select the first result for easy Enter key usage
                if index == 0 {
                    results_list.select_row(Some(&row));
                }
            }
        });

        // Reflect the currently selected result's name directly under the search field
        let results_list_for_select = self.results_list.clone();
        let name_label_for_select = self.selected_name_label.clone();
        results_list_for_select.connect_row_selected(move |_, row_opt| {
            if let Some(row) = row_opt {
                if let Some(box_) = row.child() {
                    if let Some(box_) = box_.downcast_ref::<Box>() {
                        if let Some(name_label) =
                            box_.last_child().and_then(|c| c.downcast::<Label>().ok())
                        {
                            name_label_for_select.set_text(&name_label.text());
                        }
                    }
                }
            } else {
                name_label_for_select.set_text("");
            }
        });
    }

    fn setup_keyboard_navigation(&self) {
        let key_controller = EventControllerKey::new();
        key_controller.set_propagation_phase(gtk4::PropagationPhase::Capture);
        let results_list = self.results_list.clone();

        key_controller.connect_key_pressed(move |_, keyval, _, _| {
            match keyval {
                gtk4::gdk::Key::Down => {
                    // Move to next result or select first if none selected
                    if let Some(selected_row) = results_list.selected_row() {
                        if let Some(next_row) = selected_row.next_sibling() {
                            if let Some(next_row) = next_row.downcast_ref::<ListBoxRow>() {
                                results_list.select_row(Some(next_row));
                            }
                        }
                    } else if let Some(first_row) = results_list.row_at_index(0) {
                        results_list.select_row(Some(&first_row));
                    }
                    glib::Propagation::Stop
                }
                gtk4::gdk::Key::Up => {
                    // Move to previous result
                    if let Some(selected_row) = results_list.selected_row() {
                        if let Some(prev_row) = selected_row.prev_sibling() {
                            if let Some(prev_row) = prev_row.downcast_ref::<ListBoxRow>() {
                                results_list.select_row(Some(prev_row));
                            }
                        }
                    }
                    glib::Propagation::Stop
                }
                gtk4::gdk::Key::Return | gtk4::gdk::Key::KP_Enter => {
                    // Activate the selected row, or select and activate the first one
                    if let Some(selected_row) = results_list.selected_row() {
                        // Use emit_by_name to trigger row-activated signal
                        results_list.emit_by_name::<()>("row-activated", &[&selected_row]);
                        glib::Propagation::Stop
                    } else if let Some(first_row) = results_list.row_at_index(0) {
                        // If no row is selected, select and activate the first one
                        results_list.select_row(Some(&first_row));
                        results_list.emit_by_name::<()>("row-activated", &[&first_row]);
                        glib::Propagation::Stop
                    } else {
                        glib::Propagation::Proceed
                    }
                }
                _ => glib::Propagation::Proceed,
            }
        });

        self.search_entry.add_controller(key_controller);

        // Also handle Enter when the results list has focus
        let list_key_controller = EventControllerKey::new();
        list_key_controller.set_propagation_phase(gtk4::PropagationPhase::Capture);
        let results_list2 = self.results_list.clone();
        list_key_controller.connect_key_pressed(move |_, keyval, _, _| match keyval {
            gtk4::gdk::Key::Return | gtk4::gdk::Key::KP_Enter => {
                if let Some(selected_row) = results_list2.selected_row() {
                    results_list2.emit_by_name::<()>("row-activated", &[&selected_row]);
                    glib::Propagation::Stop
                } else {
                    glib::Propagation::Proceed
                }
            }
            _ => glib::Propagation::Proceed,
        });
        self.results_list.add_controller(list_key_controller);

        // Make the results list focusable (so arrows + Enter work in list)
        self.results_list.set_can_focus(true);
    }

    pub fn connect_row_selected<F: Fn(&ExerciseResult) + 'static>(&self, f: F) {
        let results_list = self.results_list.clone();
        results_list.connect_row_selected(move |_, row| {
            if let Some(row) = row {
                if let Some(box_) = row.child() {
                    if let Some(box_) = box_.downcast_ref::<Box>() {
                        if let Some(code_label) =
                            box_.first_child().and_then(|c| c.downcast::<Label>().ok())
                        {
                            if let Some(name_label) =
                                box_.last_child().and_then(|c| c.downcast::<Label>().ok())
                            {
                                let code = code_label.text();
                                let name = name_label.text();
                                f(&ExerciseResult {
                                    code: code.to_string(),
                                    name: name.to_string(),
                                });
                            }
                        }
                    }
                }
            }
        });
    }

    fn search_exercises(conn: &Connection, query: &str) -> Result<Vec<ExerciseResult>> {
        let mut results = Vec::new();

        // Search using FTS5 across name, code, and aliases
        let fts_query = format!("{}*", query);
        let mut stmt = conn.prepare(
            "SELECT e.code, e.name 
             FROM exercise_fts f
             JOIN exercise e ON f.rowid = e.id
             WHERE exercise_fts MATCH ?
             ORDER BY rank",
        )?;

        let mut rows = stmt.query([&fts_query])?;
        while let Some(row) = rows.next()? {
            results.push(ExerciseResult {
                code: row.get(0)?,
                name: row.get(1)?,
            });
        }

        // Also search by bodypart if no FTS results found
        if results.is_empty() {
            let mut stmt = conn.prepare(
                "SELECT DISTINCT e.code, e.name
                 FROM exercise e
                 JOIN exercise_body_part ebp ON e.id = ebp.exercise_id
                 JOIN body_part bp ON ebp.body_part_id = bp.id
                 WHERE bp.name LIKE ? OR bp.key LIKE ?
                 LIMIT 20",
            )?;

            let bodypart_query = format!("%{}%", query);
            let mut rows = stmt.query([&bodypart_query, &bodypart_query])?;
            while let Some(row) = rows.next()? {
                results.push(ExerciseResult {
                    code: row.get(0)?,
                    name: row.get(1)?,
                });
            }
        }

        Ok(results)
    }

    fn create_result_row(result: &ExerciseResult) -> ListBoxRow {
        let row = ListBoxRow::new();
        let box_ = Box::new(Orientation::Horizontal, 10);

        let code_label = Label::new(Some(&result.code));
        code_label.add_css_class("monospace");
        code_label.set_width_chars(15);

        let name_label = Label::new(Some(&result.name));
        name_label.set_hexpand(true);

        box_.append(&code_label);
        box_.append(&name_label);
        row.set_child(Some(&box_));

        row
    }

    pub fn connect_row_activated<F: Fn(&ExerciseResult) + 'static>(&self, f: F) {
        let results_list = self.results_list.clone();
        let selected_name_label = self.selected_name_label.clone();
        results_list.connect_row_activated(move |_, row| {
            if let Some(box_) = row.child() {
                if let Some(box_) = box_.downcast_ref::<Box>() {
                    if let Some(code_label) =
                        box_.first_child().and_then(|c| c.downcast::<Label>().ok())
                    {
                        if let Some(name_label) =
                            box_.last_child().and_then(|c| c.downcast::<Label>().ok())
                        {
                            let code = code_label.text();
                            let name = name_label.text();
                            selected_name_label.set_text(&name);
                            f(&ExerciseResult {
                                code: code.to_string(),
                                name: name.to_string(),
                            });
                        }
                    }
                }
            }
        });
    }

    pub fn selected_exercise(&self) -> Option<ExerciseResult> {
        if let Some(row) = self.results_list.selected_row() {
            if let Some(box_) = row.child() {
                if let Some(box_) = box_.downcast_ref::<Box>() {
                    let code_label = box_
                        .first_child()
                        .and_then(|c| c.downcast::<Label>().ok())?;
                    let name_label = box_.last_child().and_then(|c| c.downcast::<Label>().ok())?;

                    return Some(ExerciseResult {
                        code: code_label.text().to_string(),
                        name: name_label.text().to_string(),
                    });
                }
            }
        }
        None
    }

    pub fn clear_search(&self) {
        self.search_entry.set_text("");
        Self::clear_results_list(&self.results_list);
    }

    pub fn set_selected_exercise(&self, exercise: &ExerciseResult) {
        // Clear the search results
        self.search_entry.set_text("");
        Self::clear_results_list(&self.results_list);

        // Show the selected exercise in the search entry as a readonly display
        let display_text = format!("✓ {} - {}", exercise.code, exercise.name);
        self.search_entry.set_text(&display_text);
        self.search_entry.set_editable(false);
        self.search_entry.add_css_class("selected-exercise");
        self.search_entry
            .set_placeholder_text(Some("Click to change exercise selection"));
        // Show the exercise name directly beneath the search field
        self.selected_name_label.set_text(&exercise.name);

        // Add click handler to reset search when clicking on the selected entry
        let click_gesture = GestureClick::new();
        let search_widget_clone2 = self.clone();
        click_gesture.connect_pressed(move |_, _, _, _| {
            search_widget_clone2.reset_search();
        });
        self.search_entry.add_controller(click_gesture);

        // Add a change button to the results area
        let change_button = Button::with_label("🔄 Change Exercise");
        change_button.add_css_class("suggested-action");

        let search_widget_clone = self.clone();
        change_button.connect_clicked(move |_| {
            search_widget_clone.reset_search();
        });

        self.results_list.append(&change_button);
    }

    pub fn reset_search(&self) {
        // Reset to editable search state
        self.search_entry.set_text("");
        self.search_entry.set_editable(true);
        self.search_entry.remove_css_class("selected-exercise");
        Self::clear_results_list(&self.results_list);
        self.search_entry
            .set_placeholder_text(Some("Search exercises by name, code, or bodypart..."));
        self.selected_name_label.set_text("");
    }

    pub fn set_state(&self, state: Arc<Mutex<AppState>>) {
        *self.plan_state.lock().unwrap() = Some(state);
    }

    pub fn set_excluded_codes(&self, codes: &[String]) {
        let mut excl = self.excluded_codes.lock().unwrap();
        excl.clear();
        for c in codes {
            excl.insert(c.clone());
        }
    }

    pub fn add_excluded_code(&self, code: &str) {
        self.excluded_codes.lock().unwrap().insert(code.to_string());
    }

    pub fn remove_excluded_code(&self, code: &str) {
        self.excluded_codes.lock().unwrap().remove(code);
    }

    #[allow(dead_code)]
    pub fn reset_exclusions(&self) {
        self.excluded_codes.lock().unwrap().clear();
    }

    pub fn refresh_results(&self) {
        // Trigger an update based on current query
        self.search_entry.emit_by_name::<()>("changed", &[]);
    }

    fn ensure_in_db(&self, code: &str, name: &str) {
        if let Some(conn) = &*self.db_connection.lock().unwrap() {
            let _ = conn.execute(
                "INSERT OR IGNORE INTO exercise (code, name) VALUES (?, ?)",
                (&code, &name),
            );
            let _ = conn.execute(
                "INSERT INTO exercise_fts(exercise_fts) VALUES('rebuild')",
                (),
            );
        }
    }

    pub fn connect_row_activated_with_import<F: Fn(&ExerciseResult) + 'static>(
        &self,
        state: Arc<Mutex<AppState>>,
        f: F,
    ) {
        let results_list = self.results_list.clone();
        let dbw = self.clone();
        let selected_name_label = self.selected_name_label.clone();
        results_list.connect_row_activated(move |_, row| {
            if let Some(box_) = row.child() {
                if let Some(box_) = box_.downcast_ref::<Box>() {
                    if let Some(code_label) =
                        box_.first_child().and_then(|c| c.downcast::<Label>().ok())
                    {
                        if let Some(name_label) =
                            box_.last_child().and_then(|c| c.downcast::<Label>().ok())
                        {
                            let code = code_label.text().to_string();
                            let name = name_label.text().to_string();
                            selected_name_label.set_text(&name);
                            dbw.ensure_in_db(&code, &name);
                            let mut needs_insert = false;
                            {
                                let s = state.lock().unwrap();
                                if let Some(plan) = &s.current_plan {
                                    if !plan.dictionary.contains_key(&code) {
                                        needs_insert = true;
                                    }
                                }
                            }
                            if needs_insert {
                                let mut s = state.lock().unwrap();
                                s.save_to_undo_history();
                                if let Some(plan) = &mut s.current_plan {
                                    plan.dictionary.insert(code.clone(), name.clone());
                                }
                                s.mark_modified();
                            }
                            f(&ExerciseResult { code, name });
                        }
                    }
                }
            }
        });
    }
}
