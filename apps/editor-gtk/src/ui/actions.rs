use crate::dialogs::comment::show_add_comment_dialog;
use crate::dialogs::day::show_add_day_dialog;
use crate::dialogs::exercise_groups::show_manage_exercise_groups_dialog;
use crate::dialogs::media::show_attach_media_dialog;
use crate::dialogs::segment::show_add_segment_dialog;
use crate::dialogs::sync::show_send_to_device_dialog;
use crate::operations::plan::create_new_plan;
use crate::operations::plan::save_as_current_plan;
use crate::operations::plan::save_current_plan;
use crate::state::AppState;
use crate::ui::plan::load_plan_from_file;
use crate::ui::plan::open_plan_dialog;
use crate::ui::plan::show_help_dialog;
use glib::clone;
use glib::VariantTy;
use gtk4::prelude::{
    ActionMapExt, ApplicationExt, BoxExt, DialogExt, FileExt, GtkApplicationExt, GtkWindowExt,
    RecentManagerExt,
};
use gtk4::{Application, ApplicationWindow};
use libadwaita::gio::SimpleAction;
use std::sync::{Arc, Mutex};
use weightlifting_core::AppPaths;

pub fn setup_actions(
    app: &Application,
    window: &ApplicationWindow,
    state: Arc<Mutex<AppState>>,
    paths: Arc<AppPaths>,
) {
    setup_file_actions(app, window, state.clone(), paths.clone());
    setup_edit_actions(app, state.clone());
    setup_help_actions(app);
    setup_tools_actions(app, state.clone(), paths.clone());
}

fn open_terminal_in_file_directory(state: Arc<Mutex<AppState>>, paths: Arc<AppPaths>) {
    let directory = {
        let app_state = state.lock().unwrap();

        // Try to get directory from current_file_path first
        if let Some(file_path) = &app_state.current_file_path {
            file_path.parent().map(|p| p.to_path_buf())
        } else if app_state.plan_id.is_some() {
            // Fall back to drafts directory if no file path
            Some(paths.drafts_dir())
        } else {
            None
        }
    };

    if let Some(dir) = directory {
        // Try to open terminal in the directory
        // Try common Linux terminal emulators in order of preference
        let terminals = [
            ("gnome-terminal", vec!["--working-directory"]),
            ("konsole", vec!["--workdir"]),
            ("xfce4-terminal", vec!["--working-directory"]),
            ("xterm", vec!["-e", "cd"]),
        ];

        let dir_str = dir.to_string_lossy().to_string();
        let mut success = false;

        for (terminal, args) in &terminals {
            let mut cmd = std::process::Command::new(terminal);

            if terminal == &"xterm" {
                // xterm needs special handling
                cmd.arg("-e")
                    .arg("bash")
                    .arg("-c")
                    .arg(format!("cd '{}' && exec bash", dir_str));
            } else {
                for arg in args {
                    cmd.arg(arg);
                }
                cmd.arg(&dir_str);
            }

            if let Ok(_) = cmd.spawn() {
                println!("Opened terminal in: {}", dir_str);
                success = true;
                break;
            }
        }

        if !success {
            use gtk4::{ButtonsType, DialogFlags, MessageDialog, MessageType};
            let dlg = MessageDialog::new(
                crate::ui::util::parent_for_dialog().as_ref(),
                DialogFlags::MODAL,
                MessageType::Error,
                ButtonsType::Ok,
                &format!(
                    "Could not open terminal. No supported terminal emulator found.\nDirectory: {}",
                    dir_str
                ),
            );
            crate::ui::util::standardize_dialog(&dlg);
            dlg.connect_response(|d, _| d.close());
            dlg.present();
        }
    } else {
        use gtk4::{ButtonsType, DialogFlags, MessageDialog, MessageType};
        let dlg = MessageDialog::new(
            crate::ui::util::parent_for_dialog().as_ref(),
            DialogFlags::MODAL,
            MessageType::Info,
            ButtonsType::Ok,
            "No file is currently open. Open or create a plan first.",
        );
        crate::ui::util::standardize_dialog(&dlg);
        dlg.connect_response(|d, _| d.close());
        dlg.present();
    }
}

fn setup_file_actions(
    app: &Application,
    window: &ApplicationWindow,
    state: Arc<Mutex<AppState>>,
    paths: Arc<AppPaths>,
) {
    let open = SimpleAction::new("open", None);
    open.connect_activate(clone!(@strong state, @strong paths => move |_, _| {
        open_plan_dialog(state.clone(), paths.clone());
    }));
    window.add_action(&open);

    let open_recent = SimpleAction::new("open_recent", Some(VariantTy::STRING));
    open_recent.connect_activate(clone!(@strong state, @strong paths => move |_, param| {
        if let Some(uri) = param.and_then(|v| v.get::<String>()) {
            let gfile = libadwaita::gio::File::for_uri(&uri);
            if let Some(pathbuf) = gfile.path() {
                // Update system recent and MRU, persist
                let _ = gtk4::RecentManager::default().add_item(&uri);
                {
                    let mut app_state = state.lock().unwrap();
                    app_state.recent_files_service.mark_opened(&uri, 9);
                    let uris = app_state.recent_files_service.get_all_uris();
                    app_state.preferences.set_recent_mru(uris);
                    let _ = app_state.preferences.save(&paths);
                }
                load_plan_from_file(state.clone(), pathbuf);
            }
        }
    }));
    window.add_action(&open_recent);

    // Add numbered recent file actions for keyboard shortcuts
    for i in 1..=9 {
        let action_name = format!("open_recent_{}", i);
        let recent_action = SimpleAction::new(&action_name, None);
        recent_action.connect_activate(
            clone!(@strong state, @strong paths, @weak window => move |_, _| {
                // Get URI for this position from MRU (no clobbering refresh here)
                let uri = {
                    let app_state = state.lock().unwrap();
                    app_state.recent_files_service.get_uri_at_position(i)
                };

                if let Some(uri) = uri {
                    let gfile = libadwaita::gio::File::for_uri(&uri);
                    if let Some(pathbuf) = gfile.path() {
                        // Update system recent and MRU, persist
                        let _ = gtk4::RecentManager::default().add_item(&uri);
                        {
                            let mut app_state = state.lock().unwrap();
                            app_state.recent_files_service.mark_opened(&uri, 9);
                            let uris = app_state.recent_files_service.get_all_uris();
                            app_state.preferences.set_recent_mru(uris);
                            let _ = app_state.preferences.save(&paths);
                        }
                        load_plan_from_file(state.clone(), pathbuf);
                    }
                }
            }),
        );
        window.add_action(&recent_action);
    }

    let new_plan = SimpleAction::new("new", None);
    new_plan.connect_activate(clone!(@strong state => move |_, _| {
        create_new_plan(state.clone());
        println!("New Plan button clicked - created new plan");
    }));
    app.add_action(&new_plan);

    let save = SimpleAction::new("save", None);
    save.connect_activate(clone!(@strong state, @strong paths => move |_, _| {
        save_current_plan(state.clone(), paths.clone());
    }));
    app.add_action(&save);

    let save_as = SimpleAction::new("save_as", None);
    save_as.connect_activate(clone!(@strong state, @strong paths => move |_, _| {
        save_as_current_plan(state.clone(), paths.clone());
    }));
    app.add_action(&save_as);

    let quit = SimpleAction::new("quit", None);
    quit.connect_activate(clone!(@weak app => move |_, _| app.quit()));
    app.add_action(&quit);
}

fn setup_edit_actions(app: &Application, state: Arc<Mutex<AppState>>) {
    let manage_groups = SimpleAction::new("manage_groups", None);
    manage_groups.connect_activate(clone!(@strong state => move |_, _| {
        show_manage_exercise_groups_dialog(state.clone());
    }));
    app.add_action(&manage_groups);

    let add_day = SimpleAction::new("add_day", None);
    add_day.connect_activate(clone!(@strong state => move |_, _| {
        show_add_day_dialog(state.clone());
    }));
    app.add_action(&add_day);

    let add_segment = SimpleAction::new("add_segment", None);
    add_segment.connect_activate(clone!(@strong state => move |_, _| {
        show_add_segment_dialog(state.clone());
    }));
    app.add_action(&add_segment);

    let add_comment = SimpleAction::new("add_comment", None);
    add_comment.connect_activate(clone!(@strong state => move |_, _| {
        show_add_comment_dialog(state.clone());
    }));
    app.add_action(&add_comment);
}

fn setup_help_actions(app: &Application) {
    let help = SimpleAction::new("help", None);
    help.connect_activate(move |_, _| {
        show_help_dialog();
    });
    app.add_action(&help);

    let about = SimpleAction::new("about", None);
    about.connect_activate(|_, _| {
        // Custom lightweight About dialog (compatible with GTK4)
        use gtk4::{Box as GtkBox, Dialog, DialogFlags, Label, ResponseType};
        let dialog = Dialog::with_buttons(
            Some("About"),
            crate::ui::util::parent_for_dialog().as_ref(),
            DialogFlags::MODAL,
            &[("OK", ResponseType::Ok)],
        );
        crate::ui::util::standardize_dialog(&dialog);
        let content = GtkBox::builder()
            .orientation(gtk4::Orientation::Vertical)
            .margin_start(20)
            .margin_end(20)
            .margin_top(20)
            .margin_bottom(20)
            .spacing(8)
            .build();
        let title = Label::builder()
            .label("Weightlifting Plan Editor")
            .css_classes(vec!["title-1".to_string()])
            .build();
        let version = Label::new(Some(concat!("Version ", env!("CARGO_PKG_VERSION"))));
        let desc = Label::new(Some(
            "Rust + GTK4/libadwaita desktop editor for training plans.",
        ));
        desc.set_wrap(true);
        content.append(&title);
        content.append(&version);
        content.append(&desc);
        dialog.content_area().append(&content);
        dialog.connect_response(|d, _| d.close());
        dialog.present();
    });
    app.add_action(&about);
}

fn setup_tools_actions(app: &Application, state: Arc<Mutex<AppState>>, paths: Arc<AppPaths>) {
    let attach_media = SimpleAction::new("attach_media", None);
    attach_media.connect_activate(clone!(@strong paths => move |_, _| {
        show_attach_media_dialog(paths.clone());
    }));
    app.add_action(&attach_media);

    let send_to_device = SimpleAction::new("send_to_device", None);
    send_to_device.connect_activate(clone!(@strong state, @strong paths => move |_, _| {
        show_send_to_device_dialog(state.clone(), paths.clone());
    }));
    app.add_action(&send_to_device);

    let validate_plan = SimpleAction::new("validate_plan", None);
    validate_plan.connect_activate(clone!(@strong state, @strong paths => move |_, _| {
        crate::dialogs::validation::show_validation_dialog(state.clone(), paths.clone());
    }));
    app.add_action(&validate_plan);

    let open_terminal = SimpleAction::new("open_terminal", None);
    open_terminal.connect_activate(clone!(@strong state, @strong paths => move |_, _| {
        open_terminal_in_file_directory(state.clone(), paths.clone());
    }));
    app.add_action(&open_terminal);
}

pub fn setup_keyboard_shortcuts(app: &Application) {
    app.set_accels_for_action("app.new", &["<Primary>N"]);
    app.set_accels_for_action("win.open", &["<Primary>O"]);
    app.set_accels_for_action("app.save", &["<Primary>S"]);
    app.set_accels_for_action("app.save_as", &["<Primary><Shift>S"]);
    app.set_accels_for_action("app.quit", &["<Primary>Q"]);
    app.set_accels_for_action("app.help", &["F1"]);

    // Add keyboard shortcuts for recent files (Ctrl+1 through Ctrl+9)
    for i in 1..=9 {
        let action_name = format!("win.open_recent_{}", i);
        let shortcut = format!("<Primary>{}", i);
        app.set_accels_for_action(&action_name, &[&shortcut]);
    }

    // Add keyboard shortcuts for edit actions
    app.set_accels_for_action("app.add_day", &["<Primary>d"]);
    app.set_accels_for_action("app.add_segment", &["<Primary>e"]);
    app.set_accels_for_action("app.add_comment", &["<Primary>m"]);
    // Tools
    app.set_accels_for_action("app.attach_media", &["<Primary>M"]);
    app.set_accels_for_action("app.open_terminal", &["<Primary><Shift>O"]);
}
