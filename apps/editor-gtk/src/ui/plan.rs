use crate::canvas::update_canvas_content;
use crate::state::AppState;
use glib::clone;
use gtk4::prelude::*;
use gtk4::{Box, Label, Orientation, RecentManager};
use std::sync::{Arc, Mutex};
use weightlifting_core::{AppPaths, Plan};

pub fn show_no_plan_error_dialog(action: &str) {
    use gtk4::{Box as GtkBox, Dialog, DialogFlags, Label, ResponseType};

    let dialog = Dialog::with_buttons(
        Some("No Plan Loaded"),
        crate::ui::util::parent_for_dialog().as_ref(),
        DialogFlags::MODAL,
        &[("OK", ResponseType::Ok)],
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

    let message = Label::builder()
        .label(format!(
            "You need to create a new plan or open an existing plan before you can {}.",
            action
        ))
        .wrap(true)
        .justify(gtk4::Justification::Center)
        .build();

    let suggestion = Label::builder()
        .label("Use File → New Plan or File → Open Plan to get started.")
        .css_classes(vec!["dim-label".to_string()])
        .wrap(true)
        .justify(gtk4::Justification::Center)
        .build();

    content.append(&message);
    content.append(&suggestion);
    dialog.content_area().append(&content);

    dialog.connect_response(|dialog, _| {
        dialog.close();
    });

    dialog.present();
}

pub fn show_help_dialog() {
    use gtk4::{Box as GtkBox, Dialog, DialogFlags, ResponseType};

    let dialog = Dialog::with_buttons(
        Some("Keyboard Shortcuts"),
        crate::ui::util::parent_for_dialog().as_ref(),
        DialogFlags::MODAL,
        &[("OK", ResponseType::Ok)],
    );
    crate::ui::util::standardize_dialog(&dialog);
    let content = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .margin_start(20)
        .margin_end(20)
        .margin_top(20)
        .margin_bottom(20)
        .spacing(8)
        .build();

    let help_text = Label::builder()
        .label("Keyboard Shortcuts:\n\nCtrl+N: New Plan\nCtrl+O: Open Plan\nCtrl+S: Save Draft\nCtrl+Shift+S: Save As\nCtrl+Enter: Promote Plan\nCtrl+Z: Undo\n\nUp/Down: Move focus\nCtrl+Up/Down: Move between days\nShift+Ctrl+Up/Down: Reorder selected segments\nDelete: Delete Selected Segments (with confirmation)\n\n+: Add Segment to focused day\nG: Group Selected Segments\nU: Ungroup Selected Segments\nEsc: Clear Selection\n\nF1: Help\n")
        .halign(gtk4::Align::Start)
        .build();

    content.append(&help_text);
    dialog.content_area().append(&content);

    dialog.connect_response(|dialog, _| {
        dialog.close();
    });

    dialog.present();
}

pub fn open_plan_dialog(state: Arc<Mutex<AppState>>, paths: Arc<AppPaths>) {
    use gtk4::{Dialog, DialogFlags, FileChooserAction, FileFilter, ResponseType};

    let dialog = Dialog::with_buttons(
        Some("Open Plan"),
        crate::ui::util::parent_for_dialog().as_ref(),
        DialogFlags::MODAL,
        &[
            ("Cancel", ResponseType::Cancel),
            ("Open", ResponseType::Accept),
        ],
    );
    crate::ui::util::standardize_dialog(&dialog);
    let file_chooser = gtk4::FileChooserWidget::new(FileChooserAction::Open);

    // Set filter for JSON files
    let filter = FileFilter::new();
    filter.add_pattern("*.json");
    filter.set_name(Some("Weightlifting Plans (*.json)"));
    file_chooser.add_filter(&filter);

    // Set initial directory - prefer last opened directory, fall back to drafts
    let initial_dir = {
        let app_state = state.lock().unwrap();
        app_state
            .last_opened_directory
            .clone()
            .or_else(|| paths.drafts_dir().canonicalize().ok())
    };

    if let Some(dir) = initial_dir {
        let gfile = gtk4::gio::File::for_path(&dir);
        let _ = file_chooser.set_current_folder(Some(&gfile));
    }

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
                    // Update/persist MRU immediately
                    let file_uri = format!("file://{}", path.display());
                    {
                        let mut app_state = state.lock().unwrap();
                        app_state.recent_files_service.mark_opened(&file_uri, 9);
                        let uris = app_state.recent_files_service.get_all_uris();
                        app_state.preferences.set_recent_mru(uris);
                        let _ = app_state.preferences.save(&paths);
                    }

                    load_plan_from_file(state.clone(), path);
                }
            }
        }
        dialog.close();
    }));

    dialog.present();
}

fn mark_recent_visit(uri: &str, _mime: &str) {
    let recent_manager = RecentManager::default();

    // For now, use add_item which should still work better than the previous approach
    // TODO: Research the correct GTK4-rs API for add_full with RecentData
    let _ = recent_manager.add_item(uri);
}

pub fn load_plan_from_file(state: Arc<Mutex<AppState>>, path: std::path::PathBuf) {
    match std::fs::read_to_string(&path) {
        Ok(json_content) => {
            match serde_json::from_str::<Plan>(&json_content) {
                Ok(plan) => {
                    let mut app_state = state.lock().unwrap();
                    let plan_id = path.file_stem().unwrap().to_string_lossy().to_string();

                    app_state.current_plan = Some(plan.clone());
                    app_state.plan_id = Some(plan_id);
                    app_state.current_file_path = Some(path.clone()); // Set the loaded file path
                    app_state.mark_saved(); // Just loaded, so not modified

                    // Record visit to recent manager
                    let file_uri = format!("file://{}", path.display());
                    mark_recent_visit(&file_uri, "application/json");

                    println!("Loaded plan: {}", plan.name);
                    // Update UI on main thread to prevent crashes
                    let state_clone = state.clone();
                    glib::idle_add_local_once(move || {
                        update_canvas_content(state_clone);
                    });
                }
                Err(e) => println!("Failed to parse plan JSON: {}", e),
            }
        }
        Err(e) => println!("Failed to read plan file: {}", e),
    }
}

fn _add_exercise_to_current_plan(state: Arc<Mutex<AppState>>) {
    use weightlifting_core::{BaseSegment, Day, RepsOrRange, RepsRange, Segment, StraightSegment};

    let mut app_state = state.lock().unwrap();

    if let Some(plan) = &mut app_state.current_plan {
        // Ensure we have at least one day
        if plan.schedule.is_empty() {
            let day = Day {
                day: 1,
                label: "Training Day".to_string(),
                time_cap_min: None,
                goal: None,
                equipment_policy: None,
                segments: vec![],
            };
            plan.schedule.push(day);
        }

        // Add exercise to first day
        let straight_segment = StraightSegment {
            base: BaseSegment {
                ex: "BENCH.BB.FLAT".to_string(),
                alt_group: None,
                group_role: None,
                per_week: None,
                load_axis_target: None,
                label: Some("Bench Press".to_string()),
                optional: None,
                technique: None,
                equipment_policy: None,
            },
            sets: Some(3),
            sets_range: None,
            reps: Some(RepsOrRange::Range(RepsRange {
                min: 8,
                max: 10,
                target: None,
            })),
            time_sec: None,
            rest_sec: None,
            rir: None,
            rpe: Some(8.0),
            tempo: None,
            vbt: None,
            load_mode: None,
            intensifier: None,
            auto_stop: None,
            interval: None,
        };

        let segment = Segment::Straight(straight_segment);
        plan.schedule[0].segments.push(segment);

        app_state.mark_modified();
        println!("Added exercise: Bench Press (3x8-10 @ RPE 8)");
    } else {
        println!("No plan open. Create a new plan first.");
    }
}

fn _add_comment_to_current_plan(state: Arc<Mutex<AppState>>) {
    use weightlifting_core::{CommentSegment, Day, Segment};

    let mut app_state = state.lock().unwrap();

    if let Some(plan) = &mut app_state.current_plan {
        // Ensure we have at least one day
        if plan.schedule.is_empty() {
            let day = Day {
                day: 1,
                label: "Training Day".to_string(),
                time_cap_min: None,
                goal: None,
                equipment_policy: None,
                segments: vec![],
            };
            plan.schedule.push(day);
        }

        let comment_segment = CommentSegment {
            text: "Rest between exercises - Death to Windows!".to_string(),
            icon: Some("note".to_string()),
        };

        let segment = Segment::Comment(comment_segment);
        plan.schedule[0].segments.push(segment);

        app_state.mark_modified();
        println!("Added comment segment");
    } else {
        println!("No plan open. Create a new plan first.");
    }
}
