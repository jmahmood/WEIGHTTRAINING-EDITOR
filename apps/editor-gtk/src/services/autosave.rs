use crate::state::AppState;
use glib::clone;
use std::sync::{Arc, Mutex};
use weightlifting_core::AppPaths;

pub fn setup_autosave_timer(state: Arc<Mutex<AppState>>, paths: Arc<AppPaths>) {
    // Set up 5-second autosave timer
    glib::timeout_add_seconds_local(
        5,
        clone!(@strong state, @strong paths => move || {
            let app_state = state.lock().unwrap();

            if app_state.is_modified && app_state.current_plan.is_some() {
                if let (Some(plan), Some(plan_id)) = (&app_state.current_plan, &app_state.plan_id) {
                    // Save draft to XDG_STATE_HOME
                    let draft_path = paths.draft_path(plan_id);

                    if let Ok(plan_json) = serde_json::to_string_pretty(plan) {
                        if std::fs::create_dir_all(draft_path.parent().unwrap()).is_ok() {
                            if std::fs::write(&draft_path, plan_json).is_ok() {
                                // Do NOT mark as saved - autosave is only for crash recovery
                                // The user's original file hasn't been updated yet
                                println!("Autosaved draft to: {}", draft_path.display());
                            }
                        }
                    }
                }
            }

            glib::ControlFlow::Continue
        }),
    );
}
