pub mod recent_files;

use libadwaita::gio::Menu;
use gtk4::{PopoverMenuBar, MenuButton};
use std::sync::{Arc, Mutex};
use crate::state::AppState;

pub fn create_main_menubar(state: Arc<Mutex<AppState>>) -> PopoverMenuBar {
    let root = Menu::new();
    
    let file_menu = create_file_menu(state);
    let edit_menu = create_edit_menu();
    let add_menu = create_add_menu();
    let tools_menu = create_tools_menu();
    
    root.append_submenu(Some("_File"), &file_menu);
    root.append_submenu(Some("_Edit"), &edit_menu);
    root.append_submenu(Some("_Add"), &add_menu);
    root.append_submenu(Some("_Tools"), &tools_menu);
    
    PopoverMenuBar::from_model(Some(&root))
}

pub fn create_help_menu_button() -> MenuButton {
    let help_menu = Menu::new();
    help_menu.append(Some("_Documentation"), Some("app.help"));
    help_menu.append(Some("_About"), Some("app.about"));
    
    MenuButton::builder()
        .label("_Help")
        .use_underline(true)
        .menu_model(&help_menu)
        .build()
}

fn create_file_menu(state: Arc<Mutex<AppState>>) -> Menu {
    let file_menu = Menu::new();
    file_menu.append(Some("_New…"), Some("app.new"));
    file_menu.append(Some("_Open…"), Some("win.open"));
    file_menu.append(Some("_Save"), Some("app.save"));
    file_menu.append(Some("Save _As…"), Some("app.save_as"));
    file_menu.append(Some("_Quit"), Some("app.quit"));
    
    let recent_menu = {
        // Build with current MRU order from service when available
        let recent_menu = Menu::new();
        let service = { state.lock().unwrap().recent_files_service.clone() };
        recent_files::populate_recent_menu_with_service(&recent_menu, 5, &["application/json"], Some(&service));

        // Also keep auto-updates on RecentManager changes
        let menu_clone = recent_menu.clone();
        let service_clone = service.clone();
        use gtk4::{RecentManager, prelude::RecentManagerExt};
        let recent_mgr = RecentManager::default();
        recent_mgr.connect_changed(move |_| {
            recent_files::populate_recent_menu_with_service(&menu_clone, 5, &["application/json"], Some(&service_clone));
        });
        recent_menu
    };
    file_menu.append_submenu(Some("Open _Recent"), &recent_menu);
    
    file_menu
}

fn create_edit_menu() -> Menu {
    let edit_menu = Menu::new();
    edit_menu.append(Some("Undo"), Some("win.open"));
    edit_menu.append(Some("Manage Exercise Groups"), Some("app.manage_groups"));
    edit_menu
}

fn create_add_menu() -> Menu {
    let add_menu = Menu::new();
    add_menu.append(Some("_Day"), Some("app.add_day"));
    add_menu.append(Some("_Segment"), Some("app.add_segment"));
    add_menu.append(Some("_Comment"), Some("app.add_comment"));
    add_menu
}

fn create_tools_menu() -> Menu {
    let tools_menu = Menu::new();
    tools_menu.append(Some("Attach _Media to Sets…"), Some("app.attach_media"));
    tools_menu.append(Some("Send _Plan to Device…"), Some("app.send_to_device"));
    tools_menu.append(Some("_Validate Plan…"), Some("app.validate_plan"));
    tools_menu
}
