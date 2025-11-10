// Group search widget for selecting exercise groups
// Supports real-time search through group names and member exercises

use crate::state::AppState;
use gtk4::prelude::*;
use gtk4::{Box, Entry, GestureClick, Label, ListBox, ListBoxRow, Orientation, ScrolledWindow};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct GroupSearchWidget {
    pub container: Box,
    search_entry: Entry,
    results_list: ListBox,
    state: Arc<Mutex<AppState>>,
    current_group: Option<String>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct GroupResult {
    pub name: String,
    pub exercise_count: usize,
    pub exercises: Vec<String>,
}

impl GroupSearchWidget {
    pub fn new(state: Arc<Mutex<AppState>>, current_group: Option<String>) -> Self {
        let search_entry = Entry::new();
        search_entry.set_placeholder_text(Some("Search exercise groups..."));

        let results_list = ListBox::new();
        results_list.set_selection_mode(gtk4::SelectionMode::Single);

        // Wrap results list in a scrolled window to limit height
        let scrolled_window = ScrolledWindow::builder()
            .hexpand(true)
            .vexpand(false)
            .min_content_height(200)
            .max_content_height(250)
            .build();
        scrolled_window.set_child(Some(&results_list));

        let container = Box::new(Orientation::Vertical, 5);
        container.append(&search_entry);
        container.append(&scrolled_window);

        let widget = Self {
            container,
            search_entry,
            results_list,
            state: state.clone(),
            current_group,
        };

        widget.setup_signals();
        widget.populate_initial_results();
        widget
    }

    fn clear_results_list(list_box: &ListBox) {
        while let Some(child) = list_box.first_child() {
            list_box.remove(&child);
        }
    }

    fn setup_signals(&self) {
        let search_entry = self.search_entry.clone();
        let results_list = self.results_list.clone();
        let state = self.state.clone();
        let current_group = self.current_group.clone();

        search_entry.connect_changed(move |entry| {
            let query = entry.text().to_string().to_lowercase();
            Self::clear_results_list(&results_list);

            if query.is_empty() {
                // Show all groups when no query
                Self::populate_all_groups(&results_list, state.clone(), current_group.as_ref());
            } else {
                // Search through groups
                Self::search_groups(&results_list, state.clone(), &query, current_group.as_ref());
            }
        });
    }

    fn populate_initial_results(&self) {
        Self::populate_all_groups(
            &self.results_list,
            self.state.clone(),
            self.current_group.as_ref(),
        );
    }

    fn populate_all_groups(
        results_list: &ListBox,
        state: Arc<Mutex<AppState>>,
        current_group: Option<&String>,
    ) {
        let state_lock = state.lock().unwrap();
        if let Some(plan) = &state_lock.current_plan {
            // Collect all groups into a vector for sorting
            let mut groups: Vec<(String, Vec<String>)> = plan
                .groups
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();

            // Sort groups: current group first, then alphabetically
            groups.sort_by(|a, b| {
                let a_is_current = current_group.map_or(false, |cg| &a.0 == cg);
                let b_is_current = current_group.map_or(false, |cg| &b.0 == cg);

                if a_is_current && !b_is_current {
                    std::cmp::Ordering::Less
                } else if !a_is_current && b_is_current {
                    std::cmp::Ordering::Greater
                } else {
                    a.0.cmp(&b.0)
                }
            });

            // Create rows for all groups
            for (group_name, exercises) in groups {
                let result = GroupResult {
                    name: group_name.clone(),
                    exercise_count: exercises.len(),
                    exercises: exercises.clone(),
                };
                let names: Vec<String> = exercises
                    .iter()
                    .take(4)
                    .map(|code| {
                        plan.dictionary
                            .get(code)
                            .cloned()
                            .unwrap_or_else(|| code.clone())
                    })
                    .collect();
                let mut preview = names.join(", ");
                if exercises.len() > 4 {
                    preview.push_str(", …");
                }

                let is_current = current_group.map_or(false, |cg| &group_name == cg);
                let row = Self::create_result_row(&result, Some(preview), is_current);
                results_list.append(&row);
            }
        }
    }

    fn search_groups(
        results_list: &ListBox,
        state: Arc<Mutex<AppState>>,
        query: &str,
        current_group: Option<&String>,
    ) {
        let state_lock = state.lock().unwrap();
        if let Some(plan) = &state_lock.current_plan {
            // Collect matching groups first
            let mut matching_groups: Vec<(String, Vec<String>)> = Vec::new();

            for (group_name, exercises) in &plan.groups {
                let mut matches = false;

                // Search in group name
                if group_name.to_lowercase().contains(query) {
                    matches = true;
                }

                // Search in exercise codes
                if !matches {
                    for exercise_code in exercises {
                        if exercise_code.to_lowercase().contains(query) {
                            matches = true;
                            break;
                        }
                    }
                }

                // Search in exercise names from dictionary
                if !matches {
                    for exercise_code in exercises {
                        if let Some(exercise_name) = plan.dictionary.get(exercise_code) {
                            if exercise_name.to_lowercase().contains(query) {
                                matches = true;
                                break;
                            }
                        }
                    }
                }

                if matches {
                    matching_groups.push((group_name.clone(), exercises.clone()));
                }
            }

            // Sort matching groups: current group first, then alphabetically
            matching_groups.sort_by(|a, b| {
                let a_is_current = current_group.map_or(false, |cg| &a.0 == cg);
                let b_is_current = current_group.map_or(false, |cg| &b.0 == cg);

                if a_is_current && !b_is_current {
                    std::cmp::Ordering::Less
                } else if !a_is_current && b_is_current {
                    std::cmp::Ordering::Greater
                } else {
                    a.0.cmp(&b.0)
                }
            });

            // Create rows for matching groups
            for (group_name, exercises) in matching_groups {
                let result = GroupResult {
                    name: group_name.clone(),
                    exercise_count: exercises.len(),
                    exercises: exercises.clone(),
                };
                let names: Vec<String> = exercises
                    .iter()
                    .take(4)
                    .map(|code| {
                        plan.dictionary
                            .get(code)
                            .cloned()
                            .unwrap_or_else(|| code.clone())
                    })
                    .collect();
                let mut preview = names.join(", ");
                if exercises.len() > 4 {
                    preview.push_str(", …");
                }

                let is_current = current_group.map_or(false, |cg| &group_name == cg);
                let row = Self::create_result_row(&result, Some(preview), is_current);
                results_list.append(&row);
            }
        }
    }

    fn create_result_row(
        result: &GroupResult,
        preview: Option<String>,
        is_current: bool,
    ) -> ListBoxRow {
        let row = ListBoxRow::new();
        let box_ = Box::new(Orientation::Vertical, 5);
        box_.set_margin_start(10);
        box_.set_margin_end(10);
        box_.set_margin_top(8);
        box_.set_margin_bottom(8);

        let name_label = Label::new(Some(&result.name));
        name_label.add_css_class("heading");
        name_label.set_halign(gtk4::Align::Start);

        let count_label = Label::new(Some(&format!("{} exercises", result.exercise_count)));
        count_label.add_css_class("dim-label");
        count_label.set_halign(gtk4::Align::Start);

        box_.append(&name_label);
        box_.append(&count_label);
        if let Some(text) = preview {
            let members_label = Label::new(Some(&text));
            members_label.add_css_class("dim-label");
            members_label.set_wrap(true);
            members_label.set_wrap_mode(gtk4::pango::WrapMode::WordChar);
            members_label.set_halign(gtk4::Align::Start);
            box_.append(&members_label);
        }
        row.set_child(Some(&box_));

        // Highlight the current group with yellow background
        if is_current {
            row.add_css_class("selected-alt-group");
        }

        // Store the group name in the row for later retrieval
        unsafe {
            row.set_data("group_name", result.name.clone());
        }

        row
    }

    pub fn connect_row_activated<F: Fn(&GroupResult) + 'static>(&self, f: F) {
        let results_list = self.results_list.clone();
        let state = self.state.clone();

        results_list.connect_row_activated(move |_, row| {
            if let Some(box_) = row.child() {
                if let Some(box_) = box_.downcast_ref::<Box>() {
                    if let Some(name_label) =
                        box_.first_child().and_then(|c| c.downcast::<Label>().ok())
                    {
                        let group_name = name_label.text().to_string();

                        // Get group details from state
                        let state_lock = state.lock().unwrap();
                        if let Some(plan) = &state_lock.current_plan {
                            if let Some(exercises) = plan.groups.get(&group_name) {
                                let result = GroupResult {
                                    name: group_name,
                                    exercise_count: exercises.len(),
                                    exercises: exercises.clone(),
                                };
                                f(&result);
                            }
                        }
                    }
                }
            }
        });
    }

    #[allow(dead_code)]
    pub fn selected_group(&self) -> Option<GroupResult> {
        if let Some(row) = self.results_list.selected_row() {
            if let Some(box_) = row.child() {
                if let Some(box_) = box_.downcast_ref::<Box>() {
                    let name_label = box_
                        .first_child()
                        .and_then(|c| c.downcast::<Label>().ok())?;
                    let group_name = name_label.text().to_string();

                    // Get group details from state
                    let state_lock = self.state.lock().unwrap();
                    if let Some(plan) = &state_lock.current_plan {
                        if let Some(exercises) = plan.groups.get(&group_name) {
                            return Some(GroupResult {
                                name: group_name,
                                exercise_count: exercises.len(),
                                exercises: exercises.clone(),
                            });
                        }
                    }
                }
            }
        }
        None
    }

    #[allow(dead_code)]
    pub fn refresh(&self) {
        let query = self.search_entry.text().to_string().to_lowercase();
        Self::clear_results_list(&self.results_list);

        if query.is_empty() {
            Self::populate_all_groups(
                &self.results_list,
                self.state.clone(),
                self.current_group.as_ref(),
            );
        } else {
            Self::search_groups(
                &self.results_list,
                self.state.clone(),
                &query,
                self.current_group.as_ref(),
            );
        }
    }

    #[allow(dead_code)]
    pub fn clear_search(&self) {
        self.search_entry.set_text("");
        Self::clear_results_list(&self.results_list);
        Self::populate_all_groups(
            &self.results_list,
            self.state.clone(),
            self.current_group.as_ref(),
        );
    }

    /// Enable right-click on group rows to open the group editor dialog
    pub fn enable_right_click_edit(&self) {
        let results_list_for_gesture = self.results_list.clone();
        let results_list_for_controller = self.results_list.clone();
        let state = self.state.clone();
        let widget_self = self.clone();

        // Add a right-click gesture to the list box
        let gesture = GestureClick::new();
        gesture.set_button(3); // Right mouse button

        gesture.connect_pressed(move |_, _, x, y| {
            // Find which row was clicked
            if let Some(row) = results_list_for_gesture.pick(x, y, gtk4::PickFlags::DEFAULT) {
                if let Some(list_row) = row.ancestor(ListBoxRow::static_type()) {
                    if let Ok(list_box_row) = list_row.downcast::<ListBoxRow>() {
                        // Get the group name from the row data
                        unsafe {
                            if let Some(group_name) = list_box_row.data::<String>("group_name") {
                                let group_name = group_name.as_ref().clone();

                                // Open the group editor dialog
                                crate::dialogs::exercise_groups::show_edit_exercise_group_dialog_simple(
                                    state.clone(),
                                    group_name
                                );

                                // Refresh the widget after the dialog closes (it will refresh when focus returns)
                                let widget_clone = widget_self.clone();
                                glib::idle_add_local_once(move || {
                                    widget_clone.refresh();
                                });
                            }
                        }
                    }
                }
            }
        });

        results_list_for_controller.add_controller(gesture);
    }
}
