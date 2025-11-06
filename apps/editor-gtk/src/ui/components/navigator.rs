use crate::state::AppState;
use crate::operations::plan::create_new_plan;
use gtk4::{Box, Orientation, Label, Button, ScrolledWindow};
use glib::clone;
use gtk4::prelude::*;
use std::sync::{Arc, Mutex};
use weightlifting_core::AppPaths;

#[allow(dead_code)]
pub fn create_navigator(state: Arc<Mutex<AppState>>, paths: Arc<AppPaths>) -> Box {
    let navigator_box = Box::builder()
        .orientation(Orientation::Vertical)
        .margin_start(8)
        .margin_end(8)
        .margin_top(8)
        .margin_bottom(8)
        .spacing(8)
        .build();

    // Navigator header
    let nav_label = Label::builder()
        .label("Plan Navigator")
        .css_classes(vec!["heading".to_string()])
        .halign(gtk4::Align::Start)
        .build();

    navigator_box.append(&nav_label);

    // Plan controls
    let plan_controls = create_plan_controls(state.clone(), paths.clone());
    navigator_box.append(&plan_controls);

    // Plan list (scrolled)
    let plan_list_container = create_plan_list();
    navigator_box.append(&plan_list_container);

    navigator_box
}

#[allow(dead_code)]
fn create_plan_controls(state: Arc<Mutex<AppState>>, _paths: Arc<AppPaths>) -> Box {
    let plan_controls = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(4)
        .homogeneous(true)
        .build();

    let new_btn = Button::with_label("New Plan");

    // Connect button signals
    new_btn.connect_clicked(clone!(@strong state => move |_| {
        create_new_plan(state.clone());
        println!("New Plan button clicked - created new plan");
    }));

    plan_controls.append(&new_btn);

    plan_controls
}

#[allow(dead_code)]
fn create_plan_list() -> ScrolledWindow {
    let scrolled = ScrolledWindow::builder()
        .hexpand(true)
        .vexpand(true)
        .has_frame(true)
        .build();

    let plan_list = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(4)
        .margin_start(8)
        .margin_end(8)
        .margin_top(8)
        .margin_bottom(8)
        .build();

    // Sample plan entry
    let plan_entry = Label::builder()
        .label("Sample Plan\nDraft • Modified today")
        .halign(gtk4::Align::Start)
        .valign(gtk4::Align::Start)
        .build();

    plan_list.append(&plan_entry);
    scrolled.set_child(Some(&plan_list));

    scrolled
}
