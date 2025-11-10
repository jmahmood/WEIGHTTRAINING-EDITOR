use crate::canvas::update_canvas_content;
use crate::dialogs::validation::show_validation_dialog;
use crate::state::AppState;
use glib::clone;
use gtk4::prelude::*;
use gtk4::{
    Box, ButtonsType, Dialog, DialogFlags, FileChooserAction, FileFilter, MessageDialog,
    MessageType, Orientation, RecentManager, ResponseType,
};
use std::sync::{Arc, Mutex};
use weightlifting_core::{AppPaths, Day, Plan};
use weightlifting_validate::PlanValidator;

fn mark_recent_visit(uri: &str, _mime: &str) {
    let recent_manager = RecentManager::default();

    // For now, use add_item which should still work better than the previous approach
    // TODO: Research the correct GTK4-rs API for add_full with RecentData
    let _ = recent_manager.add_item(uri);
}

fn show_error_dialog(message: &str) {
    let dlg = MessageDialog::new(
        crate::ui::util::parent_for_dialog().as_ref(),
        DialogFlags::MODAL,
        MessageType::Error,
        ButtonsType::Ok,
        message,
    );
    crate::ui::util::standardize_dialog(&dlg);
    dlg.connect_response(|d, _| d.close());
    dlg.present();
}

pub fn save_current_plan(state: Arc<Mutex<AppState>>, paths: Arc<AppPaths>) {
    // Ensure a plan exists
    if state.lock().unwrap().current_plan.is_some() {
        // Pre-save validation gate
        if !validate_before_user_save(state.clone(), paths.clone()) {
            return;
        }

        let mut app_state = state.lock().unwrap();
        // Use the current file path if it exists (from Save As), otherwise fall back to draft path
        let save_path = if let Some(current_path) = &app_state.current_file_path {
            current_path.clone()
        } else if let Some(plan_id) = &app_state.plan_id {
            // Ensure drafts dir exists
            if let Err(e) = paths.ensure_subdirs() {
                drop(app_state);
                show_error_dialog(&format!("Failed to create save directories: {}", e));
                return;
            }
            paths.drafts_dir().join(format!("{}.json", plan_id))
        } else {
            println!("No plan ID set. Use 'New Plan' to create a plan first.");
            return;
        };

        match serde_json::to_string_pretty(&app_state.current_plan.as_ref().unwrap()) {
            Ok(json) => {
                if let Err(e) = std::fs::write(&save_path, json) {
                    show_error_dialog(&format!("Failed to save plan: {}", e));
                } else {
                    app_state.mark_saved();
                    // Remember the directory for future dialogs
                    if let Some(parent_dir) = save_path.parent() {
                        app_state
                            .update_last_opened_directory(Some(parent_dir.to_path_buf()), &paths);
                    }

                    // Record visit to recent manager and update/persist app MRU
                    let file_uri = format!("file://{}", save_path.display());
                    mark_recent_visit(&file_uri, "application/json");
                    app_state.recent_files_service.mark_opened(&file_uri, 9);
                    let uris = app_state.recent_files_service.get_all_uris();
                    app_state.preferences.set_recent_mru(uris);
                    let _ = app_state.preferences.save(&paths);

                    println!("Plan saved to {}", save_path.display());
                }
            }
            Err(e) => show_error_dialog(&format!("Failed to serialize plan: {}", e)),
        }
    } else {
        show_error_dialog("No plan to save. Create a new plan first.");
    }
}

pub fn save_as_current_plan(state: Arc<Mutex<AppState>>, paths: Arc<AppPaths>) {
    // Extract all needed data from state first, then drop the lock
    let (plan_name, initial_dir) = {
        let app_state = state.lock().unwrap();
        if let Some(plan) = &app_state.current_plan {
            let name = plan.name.clone();
            let dir = app_state
                .last_opened_directory
                .clone()
                .or_else(|| paths.drafts_dir().canonicalize().ok());
            (Some(name), dir)
        } else {
            (None, None)
        }
    }; // Lock is dropped here

    if let Some(plan_name) = plan_name {
        let dialog = Dialog::with_buttons(
            Some("Save Plan As"),
            crate::ui::util::parent_for_dialog().as_ref(),
            DialogFlags::MODAL,
            &[
                ("Cancel", ResponseType::Cancel),
                ("Save", ResponseType::Accept),
            ],
        );
        crate::ui::util::standardize_dialog(&dialog);
        let file_chooser = gtk4::FileChooserWidget::new(FileChooserAction::Save);

        // Set filter for JSON files
        let filter = FileFilter::new();
        filter.add_pattern("*.json");
        filter.set_name(Some("Weightlifting Plans (*.json)"));
        file_chooser.add_filter(&filter);

        // Set initial directory - prefer last opened directory, fall back to drafts
        if let Some(dir) = initial_dir {
            let gfile = gtk4::gio::File::for_path(&dir);
            let _ = file_chooser.set_current_folder(Some(&gfile));
        }

        // Generate a unique filename based on plan name
        let base_name = generate_unique_filename(&plan_name, &paths.drafts_dir());
        file_chooser.set_current_name(&base_name);

        let content = Box::builder()
            .orientation(Orientation::Vertical)
            .margin_start(20)
            .margin_end(20)
            .margin_top(20)
            .margin_bottom(20)
            .spacing(12)
            .build();

        content.append(&file_chooser);
        dialog.content_area().append(&content);

        dialog.connect_response(clone!(@strong state, @strong paths, @strong file_chooser => move |dialog, response| {
            if response == ResponseType::Accept {
                if let Some(file) = file_chooser.file() {
                    if let Some(path) = file.path() {
                        // Remember the directory for future dialogs
                        if let Some(parent_dir) = path.parent() {
                            let mut app_state = state.lock().unwrap();
                            app_state.update_last_opened_directory(Some(parent_dir.to_path_buf()), &paths);
                        }
                        save_to_file(state.clone(), paths.clone(), path);
                    }
                }
            }
            dialog.close();
        }));

        dialog.present();
    } else {
        println!("No plan to save. Create a new plan first.");
    }
}

fn save_to_file(state: Arc<Mutex<AppState>>, _paths: Arc<AppPaths>, path: std::path::PathBuf) {
    // Gate first
    if !validate_before_user_save(state.clone(), _paths.clone()) {
        return;
    }
    let mut app_state = state.lock().unwrap();

    if let Some(plan) = &app_state.current_plan {
        match serde_json::to_string_pretty(plan) {
            Ok(json) => {
                if let Err(e) = std::fs::write(&path, json) {
                    println!("Failed to save plan: {}", e);
                } else {
                    // Update plan ID to the new filename (without extension)
                    if let Some(file_stem) = path.file_stem() {
                        app_state.plan_id = Some(file_stem.to_string_lossy().to_string());
                    }
                    // Store the full path for future saves
                    app_state.current_file_path = Some(path.clone());
                    app_state.mark_saved();
                    // Remember the directory for future dialogs
                    if let Some(parent_dir) = path.parent() {
                        app_state
                            .update_last_opened_directory(Some(parent_dir.to_path_buf()), &_paths);
                    }

                    // Record visit to recent manager and update/persist app MRU
                    let file_uri = format!("file://{}", path.display());
                    mark_recent_visit(&file_uri, "application/json");
                    app_state.recent_files_service.mark_opened(&file_uri, 9);
                    let uris = app_state.recent_files_service.get_all_uris();
                    app_state.preferences.set_recent_mru(uris);
                    let _ = app_state.preferences.save(&_paths);

                    println!("Plan saved to {}", path.display());
                }
            }
            Err(e) => show_error_dialog(&format!("Failed to serialize plan: {}", e)),
        }
    }
}

fn generate_unique_filename(plan_name: &str, directory: &std::path::Path) -> String {
    // Convert plan name to valid filename (similar to CLI)
    let base_id = plan_name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim_matches('_')
        .to_string();

    let mut filename = format!("{}.json", base_id);
    let mut counter = 1;

    // Check if file exists and increment counter until we find a unique name
    while directory.join(&filename).exists() {
        filename = format!("{}_({}).json", base_id, counter);
        counter += 1;
    }

    filename
}

pub fn promote_current_plan(state: Arc<Mutex<AppState>>, paths: Arc<AppPaths>) {
    use chrono::Local;
    let app_state = state.lock().unwrap();
    if let (Some(plan), Some(plan_id)) = (&app_state.current_plan, &app_state.plan_id) {
        // Version by timestamp: YYYYMMDD-HHMMSS
        let version = Local::now().format("%Y%m%d-%H%M%S").to_string();
        let out_path = paths.active_plan_path(plan_id, &version);
        if let Some(dir) = out_path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        match serde_json::to_string_pretty(plan) {
            Ok(json) => {
                if let Err(e) = std::fs::write(&out_path, json) {
                    println!("Failed to promote plan: {}", e);
                } else {
                    println!("Promoted to {}", out_path.display());
                }
            }
            Err(e) => println!("Failed to serialize plan: {}", e),
        }
    } else {
        println!("No plan open. Create or load a plan first.");
    }
}

pub fn copy_current_plan_to_device(state: Arc<Mutex<AppState>>, paths: Arc<AppPaths>) {
    let plan_clone = {
        let app_state = state.lock().unwrap();
        app_state.current_plan.clone()
    };
    if let Some(plan) = plan_clone {
        let plan_name = plan.name.clone();
        // Export to data_dir/device_exports/<sanitized_name>.json
        let mut sanitized = plan_name
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '_' })
            .collect::<String>();
        if sanitized.is_empty() {
            sanitized = "plan".to_string();
        }
        let export_dir = paths.data_dir.join("device_exports");
        let _ = std::fs::create_dir_all(&export_dir);
        let out_path = export_dir.join(format!("{}.json", sanitized));
        match serde_json::to_string_pretty(&plan) {
            Ok(json) => {
                if let Err(e) = std::fs::write(&out_path, json) {
                    println!("Failed to copy to device folder: {}", e);
                } else {
                    println!("Copied plan to {}", out_path.display());
                }
            }
            Err(e) => println!("Failed to serialize plan: {}", e),
        }
    } else {
        println!("No plan open. Cannot copy.");
    }
}

pub fn create_new_plan(state: Arc<Mutex<AppState>>) {
    let mut app_state = state.lock().unwrap();

    // Create a basic new plan
    let mut plan = Plan::new("New Plan".to_string());
    plan.dictionary.insert(
        "EXAMPLE.EXERCISE".to_string(),
        "Example Exercise".to_string(),
    );
    plan.groups.insert(
        "GROUP_EXAMPLE".to_string(),
        vec!["EXAMPLE.EXERCISE".to_string()],
    );

    // Automatically add the first day
    let first_day = Day {
        day: 1,
        label: "Training Day".to_string(),
        time_cap_min: None,
        goal: None,
        equipment_policy: None,
        segments: vec![],
    };
    plan.schedule.push(first_day);

    app_state.current_plan = Some(plan);
    app_state.plan_id = Some("new_plan".to_string());
    app_state.current_file_path = None; // Clear any previous save location
    app_state.mark_modified();
    app_state.clear_selection();

    println!("Created new plan with multi-select support");
    // Update UI on main thread to prevent crashes
    let state_clone = state.clone();
    glib::idle_add_local_once(move || {
        update_canvas_content(state_clone);
    });
}

/// Validate current plan; if there are errors, offer to Fix… (opens validation dialog) or Save Anyway.
/// Returns true if user chooses to proceed with save, false otherwise.
fn validate_before_user_save(state: Arc<Mutex<AppState>>, paths: Arc<AppPaths>) -> bool {
    let plan_clone = {
        let s = state.lock().unwrap();
        s.current_plan.clone()
    };
    if let Some(plan) = plan_clone {
        let validator = PlanValidator::new().expect("validator");
        let result = validator.validate(&plan);
        if result.errors.is_empty() {
            return true;
        }

        // Build modal choice dialog
        use gtk4::{Box as GtkBox, Dialog, DialogFlags, Label, ResponseType};
        let dlg = Dialog::with_buttons(
            Some("Validation Issues Detected"),
            crate::ui::util::parent_for_dialog().as_ref(),
            DialogFlags::MODAL,
            &[
                ("Cancel", ResponseType::Cancel),
                ("Fix…", ResponseType::Reject),
                ("Save Anyway", ResponseType::Accept),
            ],
        );
        crate::ui::util::standardize_dialog(&dlg);
        let content = GtkBox::builder()
            .orientation(gtk4::Orientation::Vertical)
            .margin_start(20)
            .margin_end(20)
            .margin_top(20)
            .margin_bottom(20)
            .spacing(8)
            .build();
        let msg = Label::builder()
            .label(format!(
                "Plan validation found {} error(s). Fix them before saving?",
                result.errors.len()
            ))
            .wrap(true)
            .build();
        content.append(&msg);

        // Show a brief list of errors so the user knows what's wrong
        let errors_box = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(4)
            .build();
        let max_preview = 6usize;
        for err in result.errors.iter().take(max_preview) {
            let line = format!("• [{}] {} @ {}", err.code, err.message, err.path);
            let lbl = gtk4::Label::builder()
                .label(line)
                .wrap(true)
                .halign(gtk4::Align::Start)
                .build();
            errors_box.append(&lbl);
        }
        if result.errors.len() > max_preview {
            let more = gtk4::Label::builder()
                .label(format!(
                    "… and {} more. Click Fix for details.",
                    result.errors.len() - max_preview
                ))
                .halign(gtk4::Align::Start)
                .build();
            errors_box.append(&more);
        }
        let scrolled = gtk4::ScrolledWindow::builder()
            .min_content_height(140)
            .child(&errors_box)
            .build();
        content.append(&scrolled);
        dlg.content_area().append(&content);

        // Synchronous wait like confirm_dialog helper
        let decision = std::rc::Rc::new(std::cell::Cell::new(ResponseType::Cancel));
        let done = std::rc::Rc::new(std::cell::Cell::new(false));
        let decision_c = decision.clone();
        let done_c = done.clone();
        dlg.connect_response(move |d, resp| {
            decision_c.set(resp);
            done_c.set(true);
            d.close();
        });
        dlg.present();
        let ctx = glib::MainContext::default();
        while !done.get() {
            ctx.iteration(true);
        }

        match decision.get() {
            ResponseType::Accept => true, // Save Anyway
            ResponseType::Reject => {
                // Open validation dialog and cancel this save
                show_validation_dialog(state.clone(), paths.clone());
                false
            }
            _ => false,
        }
    } else {
        false
    }
}
