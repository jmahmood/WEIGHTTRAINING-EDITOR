use crate::ui::actions::{setup_actions, setup_keyboard_shortcuts};
use crate::ui::components::{create_canvas, create_preview_bar, create_right_panel};
use crate::ui::menus::{create_main_menubar, create_help_menu_button};
use crate::operations::plan::save_current_plan;
use crate::state::AppState;
use crate::setup_autosave_timer;
use crate::setup_keyboard_shortcuts as setup_app_keyboard_shortcuts;
use libadwaita::HeaderBar;
use gtk4::{Application, ApplicationWindow, Paned, Box, Orientation, Button};
use glib::clone;
use gtk4::prelude::*;
use std::sync::{Arc, Mutex};
use weightlifting_core::AppPaths;

pub fn build_ui(app: &Application) {
    // Initialize libadwaita
    libadwaita::init().expect("Failed to initialize libadwaita");
    
    // Initialize app state and paths
    let paths = Arc::new(AppPaths::new().expect("Failed to initialize app paths"));
    let state = Arc::new(Mutex::new(AppState::new(&paths)));
    
    // Create main window
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Plan Editor")
        .default_width(1800)
        .default_height(900)
        .build();

    // Setup all actions
    setup_actions(app, &window, state.clone(), paths.clone());
    
    // Create UI components
    let (header_bar, main_layout) = create_main_layout(state.clone(), paths.clone());
    
    window.set_titlebar(Some(&header_bar));
    window.set_child(Some(&main_layout));
    
    // Setup keyboard shortcuts
    setup_keyboard_shortcuts(app);
    
    // Set up autosave timer (5-second intervals)
    setup_autosave_timer(state.clone(), paths.clone());
    
    // Setup app-specific keyboard shortcuts
    setup_app_keyboard_shortcuts(&window, state.clone(), paths.clone());
    
    window.present();
}


fn create_main_layout(state: Arc<Mutex<AppState>>, paths: Arc<AppPaths>) -> (HeaderBar, Box) {
    // Create header bar
    let header_bar = create_header_bar(state.clone(), paths.clone());
    
    // Create main layout with two panes (Canvas | Right Panel)
    let main_pane = Paned::builder()
        .orientation(Orientation::Horizontal)
        .position(1100)
        .build();
    // Ensure neither side collapses to zero
    main_pane.set_shrink_start_child(false);
    main_pane.set_shrink_end_child(false);

    // Center: Canvas
    let canvas = create_canvas(state.clone(), paths.clone());
    main_pane.set_start_child(Some(&canvas));

    // Right: Exercises / Exercise Groups tabs
    let right_panel = create_right_panel(state.clone(), paths.clone());
    right_panel.set_width_request(360);
    main_pane.set_end_child(Some(&right_panel));

    // Main container with preview bar at bottom
    let main_box = Box::builder()
        .orientation(Orientation::Vertical)
        .build();

    main_box.append(&main_pane);

    // Bottom: Preview Bar
    let preview_bar = create_preview_bar(state.clone(), paths.clone());
    main_box.append(&preview_bar);

    (header_bar, main_box)
}


fn create_header_bar(state: Arc<Mutex<AppState>>, paths: Arc<AppPaths>) -> HeaderBar {
    let header_bar = HeaderBar::new();
    
    // Create menubar
    let menubar = create_main_menubar(state.clone());
    header_bar.pack_start(&menubar);
    
    // Set up RecentFilesService to update when RecentManager changes
    use gtk4::{RecentManager, prelude::RecentManagerExt};
    let recent_mgr = RecentManager::default();
    // Initial sync from system and persist
    {
        let mut app_state = state.lock().unwrap();
        app_state.recent_files_service.sync_from_recent_manager(9, &["application/json"]);
        let uris = app_state.recent_files_service.get_all_uris();
        app_state.preferences.set_recent_mru(uris);
        let _ = app_state.preferences.save(&paths);
    }
    // On changes, merge rather than clobber MRU, then persist
    recent_mgr.connect_changed(clone!(@strong state, @strong paths => move |_| {
        let mut app_state = state.lock().unwrap();
        app_state.recent_files_service.sync_from_recent_manager(9, &["application/json"]);
        let uris = app_state.recent_files_service.get_all_uris();
        app_state.preferences.set_recent_mru(uris);
        let _ = app_state.preferences.save(&paths);
    }));
    
    // Save button
    let save_btn = Button::from_icon_name("document-save-symbolic");
    save_btn.set_tooltip_text(Some("Save current plan (Ctrl+S)"));
    save_btn.set_sensitive(false); // Initially disabled
    save_btn.connect_clicked(clone!(@strong state, @strong paths => move |_| {
        save_current_plan(state.clone(), paths.clone());
    }));
    
    // Monitor state changes to enable/disable save button
    let save_btn_clone = save_btn.clone();
    glib::timeout_add_seconds_local(1, clone!(@strong state => move || {
        let app_state = state.lock().unwrap();
        let has_plan = app_state.current_plan.is_some();
        let is_modified = app_state.is_modified;
        drop(app_state);
        
        save_btn_clone.set_sensitive(has_plan && is_modified);
        glib::ControlFlow::Continue
    }));
    
    // Help menu button
    let help_btn = create_help_menu_button();
    
    header_bar.pack_end(&help_btn);
    header_bar.pack_end(&save_btn);

    header_bar
}
