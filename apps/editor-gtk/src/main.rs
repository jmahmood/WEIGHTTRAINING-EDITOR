// Module declarations
mod canvas;
mod dialogs;
mod operations;
mod services;
mod state;
mod ui;

// Imports
use crate::ui::mainmenu::build_ui;
use gtk4::prelude::*;
use gtk4::{gdk, Application, CssProvider};

const APP_ID: &str = "com.github.weightlifting-desktop";

fn main() {
    // **Death to Windows!** - GTK4 + libadwaita native Linux application
    let app = Application::builder().application_id(APP_ID).build();

    // Load custom CSS for segment selection highlighting
    app.connect_startup(|_| {
        let provider = CssProvider::new();
        provider.load_from_data(
            ".selected-segment { background-color: rgba(53, 132, 228, 0.2); border: 1px solid #3584e4; border-radius: 6px; } \
             .focused-segment { background-color: rgba(255, 193, 7, 0.3); border: 2px dashed #ffc107; border-radius: 6px; } \
             .focused-day { background-color: rgba(46, 160, 67, 0.3); border: 2px dashed #2ea043; border-radius: 8px; font-weight: bold; } \
             .selected-alt-group { background-color: rgba(255, 193, 7, 0.3); border-left: 3px solid #ffc107; }"
        );
        gtk4::style_context_add_provider_for_display(&gdk::Display::default().unwrap(), &provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);
    });

    app.connect_activate(build_ui);
    app.run();
}

// Re-export commonly used functions for backward compatibility
pub use services::autosave::setup_autosave_timer;
pub use services::keyboard::setup_keyboard_shortcuts;
