use crate::state::AppState;
use crate::ui::plan::show_no_plan_error_dialog;
use crate::dialogs::comment::show_add_comment_dialog;
use crate::dialogs::day::show_add_day_dialog;
use gtk4::{Box, Orientation, Label, Button, ScrolledWindow};
use libadwaita::StatusPage;
use glib::clone;
use gtk4::prelude::*;
use std::sync::{Arc, Mutex};
use weightlifting_core::AppPaths;

pub fn create_canvas(state: Arc<Mutex<AppState>>, _paths: Arc<AppPaths>) -> Box {
    let canvas_box = Box::builder()
        .orientation(Orientation::Vertical)
        .margin_start(8)
        .margin_end(8)
        .margin_top(8)
        .margin_bottom(8)
        .spacing(8)
        .build();

    // Canvas header
    let canvas_label = Label::builder()
        .label("Workout Canvas")
        .css_classes(vec!["heading".to_string()])
        .halign(gtk4::Align::Start)
        .build();

    canvas_box.append(&canvas_label);

    // Segment controls
    let segment_controls = create_segment_controls(state.clone());
    canvas_box.append(&segment_controls);

    // Canvas content (scrolled)
    let canvas_content = create_canvas_content(state.clone());
    canvas_box.append(&canvas_content);

    canvas_box
}

fn create_segment_controls(state: Arc<Mutex<AppState>>) -> Box {
    let segment_controls = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(4)
        .build();

    // let add_segment_btn = Button::with_label("+ Segment");
    // add_segment_btn.set_tooltip_text(Some("Add Segment (Ctrl+E)"));
    let add_day_btn = Button::with_label("+ Day");
    add_day_btn.set_tooltip_text(Some("Add Day (Ctrl+D)"));
    let add_comment_btn = Button::with_label("+ Comment");
    add_comment_btn.set_tooltip_text(Some("Add Comment (Ctrl+M)"));

    // // Connect canvas button signals
    // add_segment_btn.connect_clicked(clone!(@strong state => move |_| {
    //     // Check if a plan is loaded before showing the dialog
    //     let app_state = state.lock().unwrap();
    //     if app_state.current_plan.is_some() {
    //         drop(app_state); // Release the lock before showing dialog
    //         show_add_segment_dialog(state.clone());
    //     } else {
    //         drop(app_state); // Release the lock before showing error dialog
    //         show_no_plan_error_dialog("add segments");
    //     }
    // }));
    
    add_day_btn.connect_clicked(clone!(@strong state => move |_| {
        // Check if a plan is loaded before showing the dialog
        let app_state = state.lock().unwrap();
        if app_state.current_plan.is_some() {
            drop(app_state); // Release the lock before showing dialog
            show_add_day_dialog(state.clone());
        } else {
            drop(app_state); // Release the lock before showing error dialog
            show_no_plan_error_dialog("add days");
        }
    }));
    
    add_comment_btn.connect_clicked(clone!(@strong state => move |_| {
        // Check if a plan is loaded before showing the dialog
        let app_state = state.lock().unwrap();
        if app_state.current_plan.is_some() {
            drop(app_state); // Release the lock before showing dialog
            show_add_comment_dialog(state.clone());
        } else {
            drop(app_state); // Release the lock before showing error dialog
            show_no_plan_error_dialog("add comments");
        }
    }));

    segment_controls.append(&add_day_btn);
    // segment_controls.append(&add_segment_btn);
    segment_controls.append(&add_comment_btn);

    segment_controls
}

fn create_canvas_content(state: Arc<Mutex<AppState>>) -> ScrolledWindow {
    let scrolled = ScrolledWindow::builder()
        .hexpand(true)
        .vexpand(true)
        .has_frame(true)
        .build();

    let status_page = StatusPage::builder()
        .icon_name("document-new-symbolic")
        .title("No Plan Open")
        .description("Create a new plan or open an existing one to start editing")
        .build();

    scrolled.set_child(Some(&status_page));

    // Store reference to scrolled window for updates
    {
        let mut app_state = state.lock().unwrap();
        app_state.canvas_scrolled = Some(scrolled.clone());
    }

    scrolled
}
