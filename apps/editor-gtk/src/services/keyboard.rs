use crate::state::AppState;
use crate::operations::plan::{create_new_plan, save_current_plan, save_as_current_plan, promote_current_plan};
use crate::operations::segment::{group_selected_segments, ungroup_selected_segments, clear_segment_selection, undo_last_action};
use crate::state::segment::{move_selected_segments_up, move_selected_segments_down, perform_segment_deletion, navigate_to_previous_segment, navigate_to_next_segment, navigate_to_previous_day, navigate_to_next_day, edit_focused_segment, add_segment_to_focused_day};
use crate::ui::plan::open_plan_dialog;
use crate::ui::plan::show_help_dialog;
use gtk4::{ApplicationWindow, gdk::{Key, ModifierType}, EventControllerKey};
use gtk4::prelude::*;
use glib::clone;
use std::sync::{Arc, Mutex};
use weightlifting_core::AppPaths;

pub fn setup_keyboard_shortcuts(window: &ApplicationWindow, state: Arc<Mutex<AppState>>, paths: Arc<AppPaths>) {
    let key_controller = EventControllerKey::new();
    
    // Set propagation phase to CAPTURE to intercept events before widgets handle them
    key_controller.set_propagation_phase(gtk4::PropagationPhase::Capture);
    
    key_controller.connect_key_pressed(clone!(@strong state, @strong paths, @weak window => @default-return glib::Propagation::Proceed, move |_, key, _, modifiers| {
        // Check if focus is in a text entry widget - if so, don't intercept letter keys
        let focus_is_text_entry = if let Some(focus) = GtkWindowExt::focus(&window) {
            focus.is::<gtk4::Entry>() || focus.is::<gtk4::TextView>() || focus.is::<gtk4::Text>()
        } else {
            false
        };

        match (key, modifiers) {
            // Ctrl+N: New Plan
            (Key::n, ModifierType::CONTROL_MASK) => {
                create_new_plan(state.clone());
                glib::Propagation::Stop
            },
            // Ctrl+O: Open Plan  
            (Key::o, ModifierType::CONTROL_MASK) => {
                open_plan_dialog(state.clone(), paths.clone());
                glib::Propagation::Stop
            },
            // Shift+Ctrl+S: Save As
            (Key::s, modifiers) if modifiers == (ModifierType::CONTROL_MASK | ModifierType::SHIFT_MASK) => {
                save_as_current_plan(state.clone(), paths.clone());
                glib::Propagation::Stop
            },
            // Ctrl+S: Save Draft
            (Key::s, ModifierType::CONTROL_MASK) => {
                save_current_plan(state.clone(), paths.clone());
                glib::Propagation::Stop
            },
            // Ctrl+Enter: Promote Plan
            (Key::Return, ModifierType::CONTROL_MASK) => {
                promote_current_plan(state.clone(), paths.clone());
                glib::Propagation::Stop
            },
            // G: Group selected segments (no modifiers, not in text entry)
            (Key::g, modifiers) if modifiers.is_empty() && !focus_is_text_entry => {
                group_selected_segments(state.clone());
                glib::Propagation::Stop
            },
            // U: Ungroup selected segments (no modifiers, not in text entry)
            (Key::u, modifiers) if modifiers.is_empty() && !focus_is_text_entry => {
                ungroup_selected_segments(state.clone());
                glib::Propagation::Stop
            },
            // Ctrl+Z: Undo (placeholder)
            (Key::z, ModifierType::CONTROL_MASK) => {
                undo_last_action(state.clone());
                glib::Propagation::Stop
            },
            // F1: Help
            (Key::F1, _) => {
                show_help_dialog();
                glib::Propagation::Stop
            },
            // Up: Navigate to previous segment (not in text entry)
            (Key::Up, modifiers) if modifiers.is_empty() && !focus_is_text_entry => {
                navigate_to_previous_segment(state.clone());
                glib::Propagation::Stop
            },
            // Down: Navigate to next segment (not in text entry)
            (Key::Down, modifiers) if modifiers.is_empty() && !focus_is_text_entry => {
                navigate_to_next_segment(state.clone());
                glib::Propagation::Stop
            },
            // Ctrl+Up: Navigate to previous day
            (Key::Up, ModifierType::CONTROL_MASK) => {
                navigate_to_previous_day(state.clone());
                glib::Propagation::Stop
            },
            // Ctrl+Down: Navigate to next day
            (Key::Down, ModifierType::CONTROL_MASK) => {
                navigate_to_next_day(state.clone());
                glib::Propagation::Stop
            },
            // Shift+Ctrl+Up: Move selected segments up
            (Key::Up, modifiers) if modifiers == (ModifierType::CONTROL_MASK | ModifierType::SHIFT_MASK) => {
                move_selected_segments_up(state.clone());
                glib::Propagation::Stop
            },
            // Shift+Ctrl+Down: Move selected segments down
            (Key::Down, modifiers) if modifiers == (ModifierType::CONTROL_MASK | ModifierType::SHIFT_MASK) => {
                move_selected_segments_down(state.clone());
                glib::Propagation::Stop
            },
            // Delete: Delete selected segments (not in text entry)
            (Key::Delete, modifiers) if modifiers.is_empty() && !focus_is_text_entry => {
                perform_segment_deletion(state.clone());
                glib::Propagation::Stop
            },
            // Enter: Edit focused segment (not in text entry)
            (Key::Return, modifiers) if modifiers.is_empty() && !focus_is_text_entry => {
                edit_focused_segment(state.clone());
                glib::Propagation::Stop
            },
            // Escape: Clear selection (not in text entry)
            (Key::Escape, modifiers) if modifiers.is_empty() && !focus_is_text_entry => {
                clear_segment_selection(state.clone());
                glib::Propagation::Stop
            },
            // Plus: Add segment to focused day (not in text entry)
            (Key::plus, modifiers) if modifiers.is_empty() && !focus_is_text_entry => {
                println!("DEBUG: Plus key detected");
                add_segment_to_focused_day(state.clone());
                glib::Propagation::Stop
            },
            // Plus with shift modifier (typical + key, not in text entry)
            (Key::equal, modifiers) if modifiers == ModifierType::SHIFT_MASK && !focus_is_text_entry => {
                println!("DEBUG: Shift+Equal key detected (+ key)");
                add_segment_to_focused_day(state.clone());
                glib::Propagation::Stop
            },
            // Also try the KP_Plus key (numpad +, not in text entry)
            (Key::KP_Add, modifiers) if modifiers.is_empty() && !focus_is_text_entry => {
                println!("DEBUG: Numpad + key detected");
                add_segment_to_focused_day(state.clone());
                glib::Propagation::Stop
            },
            _ => glib::Propagation::Proceed
        }
    }));
    
    window.add_controller(key_controller);
}