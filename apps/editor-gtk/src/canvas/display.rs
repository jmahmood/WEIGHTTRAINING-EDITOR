use crate::canvas::formatting::format_segment;
use crate::dialogs::day::show_edit_day_dialog;
use crate::dialogs::segment::{edit_segment_if_applicable, show_add_segment_to_day_dialog};
use crate::operations::segment::{
    move_segment_down, move_segment_up, remove_exercise_from_circuit, remove_exercise_from_superset,
};
use crate::state::{
    segment::{
        select_segment_exclusively, select_segment_range, toggle_segment_selection_with_ctrl,
    },
    AppState, FocusMode,
};
use glib;
use glib::clone;
use gtk4::prelude::*;
use gtk4::{gdk::ModifierType, Box, Button, Label, Orientation};
use libadwaita::StatusPage;
use std::sync::{Arc, Mutex};

pub fn update_canvas_content(state: Arc<Mutex<AppState>>) {
    let app_state = state.lock().unwrap();

    if let Some(scrolled) = &app_state.canvas_scrolled {
        if let Some(plan) = &app_state.current_plan {
            // Create content box for plan display
            let content_box = Box::builder()
                .orientation(Orientation::Vertical)
                .spacing(8)
                .margin_start(8)
                .margin_end(8)
                .margin_top(8)
                .margin_bottom(8)
                .build();

            // Plan header
            let plan_header = Label::builder()
                .label(format!("Plan: {}", plan.name))
                .css_classes(vec!["title-1".to_string()])
                .halign(gtk4::Align::Start)
                .build();
            let right_click = gtk4::GestureClick::new();
            right_click.set_button(3);
            let state_clone = state.clone();
            let plan_header_for_menu = plan_header.clone();
            right_click.connect_pressed(move |_g, n, x, y| {
                if n == 1 {
                    crate::ui::util::show_edit_context_menu(
                        &plan_header_for_menu,
                        x,
                        y,
                        std::boxed::Box::new(clone!(@strong state_clone => move || {
                            crate::dialogs::plan::show_edit_plan_metadata_dialog(state_clone.clone());
                        }))
                    );
                }
            });
            plan_header.add_controller(right_click);
            content_box.append(&plan_header);

            // Display days and segments
            for (day_idx, day) in plan.schedule.iter().enumerate() {
                let day_box = Box::builder()
                    .orientation(Orientation::Vertical)
                    .spacing(4)
                    .css_classes(vec!["card".to_string()])
                    .build();

                // Day header with label and add buttons
                let day_header_box = Box::builder()
                    .orientation(Orientation::Horizontal)
                    .spacing(8)
                    .build();

                let day_label = Label::builder()
                    .label(format!("Day {}: {}", day.day, day.label))
                    .css_classes(vec!["heading".to_string()])
                    .halign(gtk4::Align::Start)
                    .hexpand(true)
                    .build();

                // Add focus highlighting for days
                if app_state.focus_mode == FocusMode::Day && app_state.focused_day == Some(day_idx)
                {
                    day_label.add_css_class("focused-day");
                }

                // Make day label clickable to focus on the day
                let day_click_gesture = gtk4::GestureClick::new();
                day_click_gesture.set_button(1); // Left click only
                let state_for_day_click = state.clone();
                let current_day_idx_for_click = day_idx;

                day_click_gesture.connect_pressed(move |_gesture, n_press, _x, _y| {
                    if n_press == 1 {
                        let mut app_state = state_for_day_click.lock().unwrap();
                        app_state.set_focused_day(current_day_idx_for_click);
                        println!("Focused on day {}", current_day_idx_for_click + 1);
                        drop(app_state);

                        // Update UI on next idle
                        let state_clone = state_for_day_click.clone();
                        glib::idle_add_local_once(move || {
                            update_canvas_content(state_clone);
                        });
                    }
                });

                day_label.add_controller(day_click_gesture);
                day_header_box.append(&day_label);

                // Add segment button for this day
                let add_segment_to_day_btn = Button::with_label("+ Segment");
                add_segment_to_day_btn.set_css_classes(&["small-button"]);
                let state_for_segment = state.clone();
                let target_day_idx = day_idx;
                add_segment_to_day_btn.connect_clicked(move |_| {
                    show_add_segment_to_day_dialog(state_for_segment.clone(), target_day_idx);
                });
                day_header_box.append(&add_segment_to_day_btn);

                // Edit day label button
                let edit_day_btn = Button::from_icon_name("document-edit-symbolic");
                edit_day_btn.set_css_classes(&["flat", "small-button"]);
                edit_day_btn.set_tooltip_text(Some("Edit day label"));
                let state_for_edit = state.clone();
                let edit_day_idx = day_idx;
                edit_day_btn.connect_clicked(move |_| {
                    show_edit_day_dialog(state_for_edit.clone(), edit_day_idx);
                });
                day_header_box.append(&edit_day_btn);

                day_box.append(&day_header_box);

                for (seg_idx, segment) in day.segments.iter().enumerate() {
                    match segment {
                        weightlifting_core::Segment::Superset(superset) => {
                            create_superset_display(
                                &day_box,
                                superset,
                                state.clone(),
                                day_idx,
                                seg_idx,
                                seg_idx > 0,
                                seg_idx < day.segments.len() - 1,
                                &app_state,
                            );
                        }
                        weightlifting_core::Segment::Circuit(circuit) => {
                            create_circuit_display(
                                &day_box,
                                circuit,
                                state.clone(),
                                day_idx,
                                seg_idx,
                                seg_idx > 0,
                                seg_idx < day.segments.len() - 1,
                                &app_state,
                            );
                        }
                        _ => {
                            // Standard segment display for non-superset/circuit segments
                            let seg_row_box = Box::builder()
                                .orientation(Orientation::Horizontal)
                                .spacing(4)
                                .margin_start(12)
                                .build();

                            // Reorder buttons
                            let move_up_btn = Button::from_icon_name("go-up-symbolic");
                            move_up_btn.set_css_classes(&["flat", "small-button"]);
                            move_up_btn.set_sensitive(seg_idx > 0);
                            let state_for_up = state.clone();
                            let up_day_idx = day_idx;
                            let up_seg_idx = seg_idx;
                            move_up_btn.connect_clicked(move |_| {
                                move_segment_up(state_for_up.clone(), up_day_idx, up_seg_idx);
                            });
                            seg_row_box.append(&move_up_btn);

                            let move_down_btn = Button::from_icon_name("go-down-symbolic");
                            move_down_btn.set_css_classes(&["flat", "small-button"]);
                            move_down_btn.set_sensitive(seg_idx < day.segments.len() - 1);
                            let state_for_down = state.clone();
                            let down_day_idx = day_idx;
                            let down_seg_idx = seg_idx;
                            move_down_btn.connect_clicked(move |_| {
                                move_segment_down(
                                    state_for_down.clone(),
                                    down_day_idx,
                                    down_seg_idx,
                                );
                            });
                            seg_row_box.append(&move_down_btn);

                            let seg_label = Label::builder()
                                .label(format_segment(segment, Some(&plan.dictionary)))
                                .halign(gtk4::Align::Start)
                                .hexpand(true)
                                .build();

                            // Make segment clickable for selection (left-click)
                            let left_gesture = gtk4::GestureClick::new();
                            left_gesture.set_button(1); // 1 = left button only
                            let state_clone_left = state.clone();
                            let current_day_idx_left = day_idx;
                            let current_seg_idx_left = seg_idx;

                            left_gesture.connect_pressed(move |gesture, n_press, _x, _y| {
                                if n_press == 1 {
                                    let modifiers = gesture.current_event_state();

                                    if modifiers.contains(ModifierType::CONTROL_MASK) {
                                        // Ctrl+Click: toggle selection
                                        toggle_segment_selection_with_ctrl(
                                            state_clone_left.clone(),
                                            current_day_idx_left,
                                            current_seg_idx_left,
                                        );
                                    } else if modifiers.contains(ModifierType::SHIFT_MASK) {
                                        // Shift+Click: range selection
                                        select_segment_range(
                                            state_clone_left.clone(),
                                            current_day_idx_left,
                                            current_seg_idx_left,
                                        );
                                    } else {
                                        // Regular click: exclusive selection
                                        select_segment_exclusively(
                                            state_clone_left.clone(),
                                            current_day_idx_left,
                                            current_seg_idx_left,
                                        );
                                    }
                                }
                            });

                            // Make segment right-click show context menu with Edit
                            let right_gesture = gtk4::GestureClick::new();
                            right_gesture.set_button(3); // 3 = right button only
                            let state_clone_right = state.clone();
                            let current_day_idx_right = day_idx;
                            let current_seg_idx_right = seg_idx;
                            let segment_clone = segment.clone();
                            let seg_label_for_menu = seg_label.clone();

                            right_gesture.connect_pressed(move |_gesture, n_press, x, y| {
                                if n_press == 1 {
                                    crate::ui::util::show_edit_context_menu(
                                        &seg_label_for_menu,
                                        x,
                                        y,
                                        std::boxed::Box::new(clone!(@strong state_clone_right, @strong segment_clone => move || {
                                            edit_segment_if_applicable(state_clone_right.clone(), &segment_clone, current_day_idx_right, current_seg_idx_right);
                                        }))
                                    );
                                }
                            });

                            seg_label.add_controller(left_gesture);
                            seg_label.add_controller(right_gesture);

                            if app_state.selected_segments.contains(&(day_idx, seg_idx)) {
                                seg_label.add_css_class("selected-segment");
                            }

                            if app_state.focused_segment == Some((day_idx, seg_idx)) {
                                seg_label.add_css_class("focused-segment");
                            }

                            seg_row_box.append(&seg_label);
                            day_box.append(&seg_row_box);
                        }
                    }
                }

                content_box.append(&day_box);
            }

            scrolled.set_child(Some(&content_box));
        } else {
            // Show empty state
            let status_page = StatusPage::builder()
                .icon_name("document-new-symbolic")
                .title("No Plan Open")
                .description("Create a new plan or open an existing one to start editing")
                .build();
            scrolled.set_child(Some(&status_page));
        }
    }
}

fn create_superset_display(
    parent_box: &Box,
    superset: &weightlifting_core::SupersetSegment,
    state: Arc<Mutex<AppState>>,
    day_idx: usize,
    seg_idx: usize,
    can_move_up: bool,
    can_move_down: bool,
    app_state: &AppState,
) {
    use gtk4::{Box, Button, Label, Orientation};

    // Get dictionary for exercise name lookups
    let dictionary = app_state.current_plan.as_ref().map(|p| &p.dictionary);

    // Main superset container
    let superset_container = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(2)
        .margin_start(12)
        .build();
    superset_container.add_css_class("superset-container");

    // Superset header with reorder buttons
    let header_box = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(4)
        .build();

    // Reorder buttons
    let move_up_btn = Button::from_icon_name("go-up-symbolic");
    move_up_btn.set_css_classes(&["flat", "small-button"]);
    move_up_btn.set_sensitive(can_move_up);
    let state_for_up = state.clone();
    move_up_btn.connect_clicked(move |_| {
        move_segment_up(state_for_up.clone(), day_idx, seg_idx);
    });
    header_box.append(&move_up_btn);

    let move_down_btn = Button::from_icon_name("go-down-symbolic");
    move_down_btn.set_css_classes(&["flat", "small-button"]);
    move_down_btn.set_sensitive(can_move_down);
    let state_for_down = state.clone();
    move_down_btn.connect_clicked(move |_| {
        move_segment_down(state_for_down.clone(), day_idx, seg_idx);
    });
    header_box.append(&move_down_btn);

    // Superset title
    let label = superset.label.as_deref().unwrap_or("Superset");
    let superset_label = Label::builder()
        .label(format!(
            "🔗 {} ({} rounds, {}s rest, {}s between rounds)",
            label, superset.rounds, superset.rest_sec, superset.rest_between_rounds_sec
        ))
        .halign(gtk4::Align::Start)
        .hexpand(true)
        .build();
    superset_label.add_css_class("superset-segment");

    // Make superset header clickable for selection and editing
    let left_gesture = gtk4::GestureClick::new();
    left_gesture.set_button(1); // 1 = left button only
    let state_clone_left = state.clone();
    left_gesture.connect_pressed(move |gesture, n_press, _, _| {
        if n_press == 1 {
            let modifiers = gesture.current_event_state();

            if modifiers.contains(ModifierType::CONTROL_MASK) {
                // Ctrl+Click: toggle selection
                toggle_segment_selection_with_ctrl(state_clone_left.clone(), day_idx, seg_idx);
            } else if modifiers.contains(ModifierType::SHIFT_MASK) {
                // Shift+Click: range selection
                select_segment_range(state_clone_left.clone(), day_idx, seg_idx);
            } else {
                // Regular click: exclusive selection
                select_segment_exclusively(state_clone_left.clone(), day_idx, seg_idx);
            }
        }
    });

    let right_gesture = gtk4::GestureClick::new();
    right_gesture.set_button(3); // 3 = right button only
    let state_clone_right = state.clone();
    let superset_clone = superset.clone();
    let superset_label_for_menu = superset_label.clone();
    right_gesture.connect_pressed(move |_gesture, n_press, x, y| {
        if n_press == 1 {
            crate::ui::util::show_edit_context_menu(
                &superset_label_for_menu,
                x,
                y,
                std::boxed::Box::new(clone!(@strong state_clone_right, @strong superset_clone => move || {
                    let segment = weightlifting_core::Segment::Superset(superset_clone.clone());
                    edit_segment_if_applicable(state_clone_right.clone(), &segment, day_idx, seg_idx);
                }))
            );
        }
    });

    superset_label.add_controller(left_gesture);
    superset_label.add_controller(right_gesture);

    if app_state.selected_segments.contains(&(day_idx, seg_idx)) {
        superset_label.add_css_class("selected-segment");
    }

    if app_state.focused_segment == Some((day_idx, seg_idx)) {
        superset_label.add_css_class("focused-segment");
    }

    header_box.append(&superset_label);
    superset_container.append(&header_box);

    // Individual exercises
    for (ex_idx, item) in superset.items.iter().enumerate() {
        let ex_box = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(4)
            .margin_start(20)
            .build();

        // Look up exercise name from dictionary
        let ex_name = if let Some(dict) = dictionary {
            dict.get(&item.ex)
                .cloned()
                .unwrap_or_else(|| item.ex.clone())
        } else {
            item.ex.clone()
        };

        let mut ex_text = if let Some(reps) = &item.reps {
            match reps {
                weightlifting_core::RepsOrRange::Range(r) => {
                    if r.min == r.max {
                        format!("  • {}: {}x{}", ex_name, item.sets, r.min)
                    } else {
                        format!("  • {}: {}x{}-{}", ex_name, item.sets, r.min, r.max)
                    }
                }
            }
        } else {
            format!("  • {}: {}x?", ex_name, item.sets)
        };

        if let Some(rpe) = item.rpe {
            ex_text = format!("{} @ RPE {}", ex_text, rpe);
        }

        let ex_label = Label::builder()
            .label(&ex_text)
            .halign(gtk4::Align::Start)
            .hexpand(true)
            .build();
        ex_label.add_css_class("superset-exercise");

        // Remove button for individual exercise
        let remove_btn = Button::from_icon_name("edit-delete-symbolic");
        remove_btn.set_css_classes(&["flat", "small-button", "destructive-action"]);
        remove_btn.set_tooltip_text(Some("Remove this exercise from superset"));

        let state_for_remove = state.clone();
        remove_btn.connect_clicked(move |_| {
            remove_exercise_from_superset(state_for_remove.clone(), day_idx, seg_idx, ex_idx);
        });

        ex_box.append(&ex_label);
        ex_box.append(&remove_btn);
        superset_container.append(&ex_box);
    }

    parent_box.append(&superset_container);
}

fn create_circuit_display(
    parent_box: &Box,
    circuit: &weightlifting_core::CircuitSegment,
    state: Arc<Mutex<AppState>>,
    day_idx: usize,
    seg_idx: usize,
    can_move_up: bool,
    can_move_down: bool,
    app_state: &AppState,
) {
    use gtk4::{Box, Button, Label, Orientation};

    // Get dictionary for exercise name lookups
    let dictionary = app_state.current_plan.as_ref().map(|p| &p.dictionary);

    // Main circuit container
    let circuit_container = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(2)
        .margin_start(12)
        .build();
    circuit_container.add_css_class("circuit-container");

    // Circuit header with reorder buttons
    let header_box = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(4)
        .build();

    // Reorder buttons
    let move_up_btn = Button::from_icon_name("go-up-symbolic");
    move_up_btn.set_css_classes(&["flat", "small-button"]);
    move_up_btn.set_sensitive(can_move_up);
    let state_for_up = state.clone();
    move_up_btn.connect_clicked(move |_| {
        move_segment_up(state_for_up.clone(), day_idx, seg_idx);
    });
    header_box.append(&move_up_btn);

    let move_down_btn = Button::from_icon_name("go-down-symbolic");
    move_down_btn.set_css_classes(&["flat", "small-button"]);
    move_down_btn.set_sensitive(can_move_down);
    let state_for_down = state.clone();
    move_down_btn.connect_clicked(move |_| {
        move_segment_down(state_for_down.clone(), day_idx, seg_idx);
    });
    header_box.append(&move_down_btn);

    // Circuit title
    let circuit_label = Label::builder()
        .label(format!(
            "🔄 Circuit ({} rounds, {}s rest, {}s between rounds)",
            circuit.rounds, circuit.rest_sec, circuit.rest_between_rounds_sec
        ))
        .halign(gtk4::Align::Start)
        .hexpand(true)
        .build();
    circuit_label.add_css_class("circuit-segment");

    // Make circuit header clickable for selection and editing
    let left_gesture = gtk4::GestureClick::new();
    left_gesture.set_button(1); // 1 = left button only
    let state_clone_left = state.clone();
    left_gesture.connect_pressed(move |gesture, n_press, _, _| {
        if n_press == 1 {
            let modifiers = gesture.current_event_state();

            if modifiers.contains(ModifierType::CONTROL_MASK) {
                // Ctrl+Click: toggle selection
                toggle_segment_selection_with_ctrl(state_clone_left.clone(), day_idx, seg_idx);
            } else if modifiers.contains(ModifierType::SHIFT_MASK) {
                // Shift+Click: range selection
                select_segment_range(state_clone_left.clone(), day_idx, seg_idx);
            } else {
                // Regular click: exclusive selection
                select_segment_exclusively(state_clone_left.clone(), day_idx, seg_idx);
            }
        }
    });

    let right_gesture = gtk4::GestureClick::new();
    right_gesture.set_button(3); // 3 = right button only
    let state_clone_right = state.clone();
    let circuit_clone = circuit.clone();
    let circuit_label_for_menu = circuit_label.clone();
    right_gesture.connect_pressed(move |_gesture, n_press, x, y| {
        if n_press == 1 {
            crate::ui::util::show_edit_context_menu(
                &circuit_label_for_menu,
                x,
                y,
                std::boxed::Box::new(clone!(@strong state_clone_right, @strong circuit_clone => move || {
                    let segment = weightlifting_core::Segment::Circuit(circuit_clone.clone());
                    edit_segment_if_applicable(state_clone_right.clone(), &segment, day_idx, seg_idx);
                }))
            );
        }
    });

    circuit_label.add_controller(left_gesture);
    circuit_label.add_controller(right_gesture);

    if app_state.selected_segments.contains(&(day_idx, seg_idx)) {
        circuit_label.add_css_class("selected-segment");
    }

    if app_state.focused_segment == Some((day_idx, seg_idx)) {
        circuit_label.add_css_class("focused-segment");
    }

    header_box.append(&circuit_label);
    circuit_container.append(&header_box);

    // Individual exercises
    for (ex_idx, item) in circuit.items.iter().enumerate() {
        let ex_box = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(4)
            .margin_start(20)
            .build();

        // Look up exercise name from dictionary
        let ex_name = if let Some(dict) = dictionary {
            dict.get(&item.ex)
                .cloned()
                .unwrap_or_else(|| item.ex.clone())
        } else {
            item.ex.clone()
        };

        let ex_text = if let Some(time_sec) = &item.time_sec {
            match time_sec {
                weightlifting_core::TimeOrRange::Fixed(time) => {
                    format!("  • {}: {}s", ex_name, time)
                }
                _ => format!("  • {}: time", ex_name),
            }
        } else if let Some(reps) = &item.reps {
            match reps {
                weightlifting_core::RepsOrRange::Range(r) => {
                    if r.min == r.max {
                        format!("  • {}: {} reps", ex_name, r.min)
                    } else {
                        format!("  • {}: {}-{} reps", ex_name, r.min, r.max)
                    }
                }
            }
        } else {
            format!("  • {}: ?", ex_name)
        };

        let ex_label = Label::builder()
            .label(&ex_text)
            .halign(gtk4::Align::Start)
            .hexpand(true)
            .build();
        ex_label.add_css_class("circuit-exercise");

        // Remove button for individual exercise
        let remove_btn = Button::from_icon_name("edit-delete-symbolic");
        remove_btn.set_css_classes(&["flat", "small-button", "destructive-action"]);
        remove_btn.set_tooltip_text(Some("Remove this exercise from circuit"));

        let state_for_remove = state.clone();
        remove_btn.connect_clicked(move |_| {
            remove_exercise_from_circuit(state_for_remove.clone(), day_idx, seg_idx, ex_idx);
        });

        ex_box.append(&ex_label);
        ex_box.append(&remove_btn);
        circuit_container.append(&ex_box);
    }

    parent_box.append(&circuit_container);
}
