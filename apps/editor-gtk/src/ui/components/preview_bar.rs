use crate::state::AppState;
use crate::operations::plan::{save_current_plan, copy_current_plan_to_device};
use gtk4::{Box, Orientation, Label, Button};
use glib::clone;
use gtk4::prelude::*;
use std::sync::{Arc, Mutex};
use weightlifting_core::AppPaths;

pub fn create_preview_bar(state: Arc<Mutex<AppState>>, paths: Arc<AppPaths>) -> Box {
    let preview_box = Box::builder()
        .orientation(Orientation::Horizontal)
        .margin_start(8)
        .margin_end(8)
        .margin_top(4)
        .margin_bottom(8)
        .spacing(16)
        .css_classes(vec!["toolbar".to_string()])
        .build();

    // Location selector
    let location_components = create_location_components();
    preview_box.append(&location_components.0);
    preview_box.append(&location_components.1);
    
    // Metrics
    let (duration_label, volume_label, equipment_label) = create_metrics_labels();
    preview_box.append(&duration_label); // duration
    preview_box.append(&volume_label); // volume
    preview_box.append(&equipment_label); // equipment
    
    // Save button
    let save_btn = create_save_button(state.clone(), paths.clone());
    preview_box.append(&save_btn);

    // Copy to device button
    let copy_btn = Button::from_icon_name("document-send-symbolic");
    copy_btn.set_tooltip_text(Some("Copy plan to device folder"));
    let state_clone_for_copy = state.clone();
    let paths_clone_for_copy = paths.clone();
    copy_btn.connect_clicked(move |_| {
        copy_current_plan_to_device(state_clone_for_copy.clone(), paths_clone_for_copy.clone());
    });
    preview_box.append(&copy_btn);
    
    // Spacer
    let spacer = create_spacer();
    preview_box.append(&spacer);
    
    // Validation status
    let validation_label = create_validation_label();
    preview_box.append(&validation_label);

    // Periodically recompute metrics from current plan
    glib::timeout_add_seconds_local(2, clone!(@strong state, @strong duration_label, @strong volume_label, @strong equipment_label => move || {
        let app_state = state.lock().unwrap();
        if let Some(plan) = &app_state.current_plan {
            let (dur, vol, equip) = compute_metrics(plan);
            duration_label.set_text(&format!("Est. Duration: {}", dur));
            volume_label.set_text(&format!("Volume: {}", vol));
            equipment_label.set_text(&format!("Equipment: {}", equip));
        } else {
            duration_label.set_text("Est. Duration: --");
            volume_label.set_text("Volume: --");
            equipment_label.set_text("Equipment: --");
        }
        glib::ControlFlow::Continue
    }));

    preview_box
}

fn create_location_components() -> (Label, Button) {
    let location_label = Label::new(Some("Location:"));
    let location_btn = Button::with_label("Home Gym");
    (location_label, location_btn)
}

fn create_metrics_labels() -> (Label, Label, Label) {
    let duration_label = Label::new(Some("Est. Duration: --"));
    let volume_label = Label::new(Some("Volume: --"));
    let equipment_label = Label::new(Some("Equipment: --"));
    (duration_label, volume_label, equipment_label)
}

fn create_save_button(state: Arc<Mutex<AppState>>, paths: Arc<AppPaths>) -> Button {
    let save_btn = Button::from_icon_name("document-save-symbolic");
    save_btn.set_tooltip_text(Some("Save current plan"));
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
    
    save_btn
}

fn create_spacer() -> Label {
    let spacer = Label::new(None);
    spacer.set_hexpand(true);
    spacer
}

fn create_validation_label() -> Label {
    Label::builder()
        .label("✓ Valid")
        .css_classes(vec!["success".to_string()])
        .build()
}

fn compute_metrics(plan: &weightlifting_core::Plan) -> (String, String, String) {
    use weightlifting_core::Segment;
    let mut total_sets: u32 = 0;
    let mut total_reps: u32 = 0;
    let mut ex_codes: std::collections::HashSet<String> = std::collections::HashSet::new();

    for day in &plan.schedule {
        for seg in &day.segments {
            match seg {
                Segment::Straight(s) => {
                    let sets = s.sets.unwrap_or(1);
                    total_sets += sets;
                    if let Some(weightlifting_core::RepsOrRange::Range(r)) = &s.reps { total_reps += sets * r.min; }
                    ex_codes.insert(s.base.ex.clone());
                }
                Segment::Rpe(r) => {
                    total_sets += r.sets;
                    if let Some(weightlifting_core::RepsOrRange::Range(x)) = &r.reps { total_reps += r.sets * x.min; }
                    ex_codes.insert(r.base.ex.clone());
                }
                Segment::Amrap(a) => {
                    total_sets += 1;
                    total_reps += a.base_reps;
                    ex_codes.insert(a.base.ex.clone());
                }
                Segment::Percentage(p) => {
                    for pres in &p.prescriptions {
                        total_sets += pres.sets;
                        total_reps += pres.sets * pres.reps;
                    }
                    ex_codes.insert(p.base.ex.clone());
                }
                Segment::Superset(su) => {
                    for item in &su.items {
                        total_sets += item.sets;
                        if let Some(weightlifting_core::RepsOrRange::Range(r)) = &item.reps { total_reps += item.sets * r.min; }
                        ex_codes.insert(item.ex.clone());
                    }
                }
                Segment::Circuit(ci) => {
                    for item in &ci.items {
                        if let Some(weightlifting_core::RepsOrRange::Range(r)) = &item.reps { total_reps += r.min; }
                        ex_codes.insert(item.ex.clone());
                    }
                }
                Segment::Scheme(sc) => {
                    for set in &sc.sets {
                        let sets = set.sets.unwrap_or(1);
                        total_sets += sets;
                        if let Some(weightlifting_core::RepsOrRange::Range(r)) = &set.reps { total_reps += sets * r.min; }
                    }
                    ex_codes.insert(sc.base.ex.clone());
                }
                Segment::Complex(cx) => {
                    total_sets += cx.sets;
                    for item in &cx.sequence { ex_codes.insert(item.ex.clone()); }
                }
                Segment::Time(t) => {
                    total_sets += 1;
                    ex_codes.insert(t.base.ex.clone());
                }
                _ => {}
            }
        }
    }

    // Duration estimate: 2 min per set (very rough)
    let total_minutes = (total_sets as f64 * 2.0).round() as u32;
    let duration = if total_minutes >= 60 { format!("{}h {}m", total_minutes / 60, total_minutes % 60) } else { format!("{}m", total_minutes) };
    // Volume: total minimum reps across plan (proxy)
    let volume = format!("{} reps (min est)", total_reps);
    // Equipment: count of unique exercises as simple proxy
    let equipment = format!("{} exercises", ex_codes.len());
    (duration, volume, equipment)
}
