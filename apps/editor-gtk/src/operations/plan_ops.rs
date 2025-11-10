// Plan modification operations (moved from UI layer)

use crate::canvas::update_canvas_content;
use crate::state::AppState;
use crate::ui::exercise_menus::exercise_data::{
    CircuitExerciseData, SchemeSetData, SupersetExerciseData,
};
use std::sync::{Arc, Mutex};
use weightlifting_core::{
    AmrapSegment, BaseSegment, Day, GroupChooseSegment, GroupState, RepsOrRange, RepsRange,
    RestOrRange, Segment, StraightSegment, TimeInterval, TimeOrRange,
};

pub fn add_rpe_set_to_plan(
    state: Arc<Mutex<AppState>>,
    ex_code: String,
    ex_label: String,
    alt_group: Option<String>,
    sets: u32,
    min_reps: u32,
    max_reps: u32,
    rpe: f64,
    rir: Option<u32>,
) {
    let mut app_state = state.lock().unwrap();
    let target_day = app_state.get_and_clear_target_day().unwrap_or(0);
    app_state.save_to_undo_history();

    if let Some(plan) = &mut app_state.current_plan {
        if !plan.dictionary.contains_key(&ex_code) {
            plan.dictionary.insert(ex_code.clone(), ex_label.clone());
        }
        if plan.schedule.is_empty() {
            let day = Day {
                day: 1,
                label: "Training Day".to_string(),
                time_cap_min: None,
                goal: None,
                equipment_policy: None,
                segments: vec![],
            };
            plan.schedule.push(day);
        }
        let target_day_index = if target_day >= plan.schedule.len() {
            0
        } else {
            target_day
        };
        let straight_segment = StraightSegment {
            base: BaseSegment {
                ex: ex_code,
                alt_group,
                label: Some(ex_label.clone()),
                optional: None,
                technique: None,
                equipment_policy: None,
            },
            sets: Some(sets),
            sets_range: None,
            reps: Some(RepsOrRange::Range(RepsRange {
                min: min_reps,
                max: max_reps,
                target: None,
            })),
            time_sec: None,
            rest_sec: None,
            rir: rir.map(|r| r as f64),
            rpe: Some(rpe),
            tempo: None,
            vbt: None,
            load_mode: None,
            intensifier: None,
            auto_stop: None,
            interval: None,
        };
        plan.schedule[target_day_index]
            .segments
            .push(Segment::Straight(straight_segment));
        app_state.mark_modified();
        if min_reps == max_reps {
            println!(
                "Added RPE set: {} ({}x{} @ RPE {})",
                ex_label, sets, min_reps, rpe
            );
        } else {
            println!(
                "Added RPE set: {} ({}x{}-{} @ RPE {})",
                ex_label, sets, min_reps, max_reps, rpe
            );
        }
        drop(app_state);
        update_canvas_content(state.clone());
    } else {
        println!("No plan open. Create a new plan first.");
    }
}

pub fn add_percentage_set_to_plan(
    state: Arc<Mutex<AppState>>,
    ex_code: String,
    ex_label: String,
    sets: u32,
    reps: u32,
    percentage: f64,
) {
    let mut app_state = state.lock().unwrap();
    let target_day = app_state.get_and_clear_target_day().unwrap_or(0);
    app_state.save_to_undo_history();

    if let Some(plan) = &mut app_state.current_plan {
        if !plan.dictionary.contains_key(&ex_code) {
            plan.dictionary.insert(ex_code.clone(), ex_label.clone());
        }
        if plan.schedule.is_empty() {
            let day = Day {
                day: 1,
                label: "Training Day".to_string(),
                time_cap_min: None,
                goal: None,
                equipment_policy: None,
                segments: vec![],
            };
            plan.schedule.push(day);
        }
        let percentage_label = format!("{} @ {}% 1RM", ex_label, percentage);
        let straight_segment = StraightSegment {
            base: BaseSegment {
                ex: ex_code,
                alt_group: None,
                label: Some(percentage_label.clone()),
                optional: None,
                technique: None,
                equipment_policy: None,
            },
            sets: Some(sets),
            sets_range: None,
            reps: Some(RepsOrRange::Range(RepsRange {
                min: reps,
                max: reps,
                target: None,
            })),
            time_sec: None,
            rest_sec: None,
            rir: None,
            rpe: None,
            tempo: None,
            vbt: None,
            load_mode: None,
            intensifier: None,
            auto_stop: None,
            interval: None,
        };
        let target_day_index = if target_day >= plan.schedule.len() {
            0
        } else {
            target_day
        };
        plan.schedule[target_day_index]
            .segments
            .push(Segment::Straight(straight_segment));
        app_state.mark_modified();
        println!(
            "Added percentage set: {} ({}x{} @ {}% 1RM)",
            ex_label, sets, reps, percentage
        );
        drop(app_state);
        update_canvas_content(state.clone());
    } else {
        println!("No plan open. Create a new plan first.");
    }
}

pub fn add_amrap_to_plan(
    state: Arc<Mutex<AppState>>,
    ex_code: String,
    ex_label: String,
    _time_sec: u32,
    _rest_sec: u32,
    _auto_stop: bool,
) {
    let mut app_state = state.lock().unwrap();
    let target_day = app_state.get_and_clear_target_day().unwrap_or(0);
    app_state.save_to_undo_history();

    if let Some(plan) = &mut app_state.current_plan {
        if !plan.dictionary.contains_key(&ex_code) {
            plan.dictionary.insert(ex_code.clone(), ex_label.clone());
        }
        if plan.schedule.is_empty() {
            let day = Day {
                day: 1,
                label: "Training Day".to_string(),
                time_cap_min: None,
                goal: None,
                equipment_policy: None,
                segments: vec![],
            };
            plan.schedule.push(day);
        }
        let amrap_segment = AmrapSegment {
            base: BaseSegment {
                ex: ex_code,
                alt_group: None,
                label: Some(ex_label.clone()),
                optional: None,
                technique: None,
                equipment_policy: None,
            },
            base_reps: 1,
            cap_reps: 100,
        };
        let target_day_index = if target_day >= plan.schedule.len() {
            0
        } else {
            target_day
        };
        plan.schedule[target_day_index]
            .segments
            .push(Segment::Amrap(amrap_segment));
        app_state.mark_modified();
        println!("Added AMRAP: {}", ex_label);
        drop(app_state);
        update_canvas_content(state.clone());
    } else {
        println!("No plan open. Create a new plan first.");
    }
}

pub fn add_superset_to_plan(
    state: Arc<Mutex<AppState>>,
    label: Option<String>,
    rounds: u32,
    rest_sec: u32,
    rest_between_rounds_sec: u32,
    exercises: Vec<SupersetExerciseData>,
) {
    use weightlifting_core::{RepsOrRange, RepsRange, SupersetItem, SupersetSegment};
    let mut app_state = state.lock().unwrap();
    let target_day = app_state.get_and_clear_target_day().unwrap_or(0);
    app_state.save_to_undo_history();

    if let Some(plan) = &mut app_state.current_plan {
        for exercise in &exercises {
            if !plan.dictionary.contains_key(&exercise.ex_code) {
                plan.dictionary
                    .insert(exercise.ex_code.clone(), exercise.ex_name.clone());
            }
        }
        if plan.schedule.is_empty() {
            let day = Day {
                day: 1,
                label: "Training Day".to_string(),
                time_cap_min: None,
                goal: None,
                equipment_policy: None,
                segments: vec![],
            };
            plan.schedule.push(day);
        }
        let items: Vec<SupersetItem> = exercises
            .iter()
            .map(|exercise| SupersetItem {
                ex: exercise.ex_code.clone(),
                sets: exercise.sets,
                reps: Some(RepsOrRange::Range(RepsRange {
                    min: exercise.reps_min,
                    max: exercise.reps_max,
                    target: None,
                })),
                time_sec: None,
                rpe: exercise.rpe,
                alt_group: exercise.alt_group.clone(),
                intensifier: None,
            })
            .collect();

        let superset_segment = SupersetSegment {
            label: label.clone(),
            pairing: None,
            rounds,
            rest_sec,
            rest_between_rounds_sec,
            items,
        };
        let target_day_index = if target_day >= plan.schedule.len() {
            0
        } else {
            target_day
        };
        plan.schedule[target_day_index]
            .segments
            .push(Segment::Superset(superset_segment));
        app_state.mark_modified();
        let label_text = label.unwrap_or("Superset".to_string());
        println!(
            "Added superset '{}' with {} exercises ({}x rounds, {}s rest, {}s between rounds)",
            label_text,
            exercises.len(),
            rounds,
            rest_sec,
            rest_between_rounds_sec
        );
        drop(app_state);
        update_canvas_content(state.clone());
    } else {
        println!("No plan open. Create a new plan first.");
    }
}

pub fn add_circuit_to_plan(
    state: Arc<Mutex<AppState>>,
    rounds: u32,
    rest_sec: u32,
    rest_between_rounds_sec: u32,
    exercises: Vec<CircuitExerciseData>,
) {
    use weightlifting_core::{CircuitItem, CircuitSegment, RepsOrRange, RepsRange, TimeOrRange};
    let mut app_state = state.lock().unwrap();
    let target_day = app_state.get_and_clear_target_day().unwrap_or(0);
    app_state.save_to_undo_history();

    if let Some(plan) = &mut app_state.current_plan {
        for exercise in &exercises {
            if !plan.dictionary.contains_key(&exercise.ex_code) {
                plan.dictionary
                    .insert(exercise.ex_code.clone(), exercise.ex_name.clone());
            }
        }
        if plan.schedule.is_empty() {
            let day = Day {
                day: 1,
                label: "Training Day".to_string(),
                time_cap_min: None,
                goal: None,
                equipment_policy: None,
                segments: vec![],
            };
            plan.schedule.push(day);
        }
        let items: Vec<CircuitItem> = exercises
            .iter()
            .map(|exercise| CircuitItem {
                ex: exercise.ex_code.clone(),
                reps: if exercise.time_sec.is_some() {
                    None
                } else {
                    Some(RepsOrRange::Range(RepsRange {
                        min: exercise.reps_min,
                        max: exercise.reps_max,
                        target: None,
                    }))
                },
                time_sec: exercise.time_sec.map(TimeOrRange::Fixed),
                alt_group: None,
            })
            .collect();

        let circuit_segment = CircuitSegment {
            rounds,
            rest_sec,
            rest_between_rounds_sec,
            items,
        };
        let target_day_index = if target_day >= plan.schedule.len() {
            0
        } else {
            target_day
        };
        plan.schedule[target_day_index]
            .segments
            .push(Segment::Circuit(circuit_segment));
        app_state.mark_modified();
        println!(
            "Added circuit with {} exercises ({}x rounds, {}s rest, {}s between rounds)",
            exercises.len(),
            rounds,
            rest_sec,
            rest_between_rounds_sec
        );
        drop(app_state);
        update_canvas_content(state.clone());
    } else {
        println!("No plan open. Create a new plan first.");
    }
}

pub fn update_superset_in_plan(
    state: Arc<Mutex<AppState>>,
    day_index: usize,
    segment_index: usize,
    label: Option<String>,
    rounds: u32,
    rest_sec: u32,
    rest_between_rounds_sec: u32,
    exercises: Vec<SupersetExerciseData>,
) {
    use weightlifting_core::{RepsOrRange, RepsRange, SupersetItem, SupersetSegment};
    let mut app_state = state.lock().unwrap();
    app_state.save_to_undo_history();

    if let Some(plan) = &mut app_state.current_plan {
        if day_index < plan.schedule.len()
            && segment_index < plan.schedule[day_index].segments.len()
        {
            for exercise in &exercises {
                if !plan.dictionary.contains_key(&exercise.ex_code) {
                    plan.dictionary
                        .insert(exercise.ex_code.clone(), exercise.ex_name.clone());
                }
            }
            let items: Vec<SupersetItem> = exercises
                .iter()
                .map(|exercise| SupersetItem {
                    ex: exercise.ex_code.clone(),
                    sets: exercise.sets,
                    reps: Some(RepsOrRange::Range(RepsRange {
                        min: exercise.reps_min,
                        max: exercise.reps_max,
                        target: None,
                    })),
                    time_sec: None,
                    rpe: exercise.rpe,
                    alt_group: exercise.alt_group.clone(),
                    intensifier: None,
                })
                .collect();

            let updated_superset = SupersetSegment {
                label: label.clone(),
                pairing: None,
                rounds,
                rest_sec,
                rest_between_rounds_sec,
                items,
            };
            plan.schedule[day_index].segments[segment_index] = Segment::Superset(updated_superset);
            app_state.mark_modified();
            let label_text = label.unwrap_or("Superset".to_string());
            println!("Updated superset '{}' with {} exercises ({}x rounds, {}s rest, {}s between rounds)", label_text, exercises.len(), rounds, rest_sec, rest_between_rounds_sec);
            drop(app_state);
            update_canvas_content(state.clone());
        } else {
            println!("Invalid day or segment index for update");
        }
    } else {
        println!("No plan open. Cannot update superset.");
    }
}

pub fn add_time_based_to_plan(
    state: Arc<Mutex<AppState>>,
    ex_code: String,
    ex_label: String,
    sets: u32,
    time_sec: u32,
    rest_sec: u32,
    interval: Option<(u32, u32, u32)>,
) {
    let mut app_state = state.lock().unwrap();
    let target_day = app_state.get_and_clear_target_day().unwrap_or(0);
    app_state.save_to_undo_history();

    if let Some(plan) = &mut app_state.current_plan {
        if !plan.dictionary.contains_key(&ex_code) {
            plan.dictionary.insert(ex_code.clone(), ex_label.clone());
        }
        if plan.schedule.is_empty() {
            let day = Day {
                day: 1,
                label: "Training Day".to_string(),
                time_cap_min: None,
                goal: None,
                equipment_policy: None,
                segments: vec![],
            };
            plan.schedule.push(day);
        }
        let interval_params = interval.map(|(work, rest, repeats)| TimeInterval {
            work,
            rest,
            repeats,
        });
        let straight_segment = StraightSegment {
            base: BaseSegment {
                ex: ex_code,
                alt_group: None,
                label: Some(ex_label.clone()),
                optional: None,
                technique: None,
                equipment_policy: None,
            },
            sets: Some(sets),
            sets_range: None,
            reps: None,
            time_sec: Some(TimeOrRange::Fixed(time_sec)),
            rest_sec: Some(RestOrRange::Fixed(rest_sec)),
            rir: None,
            rpe: None,
            tempo: None,
            vbt: None,
            load_mode: None,
            intensifier: None,
            auto_stop: None,
            interval: interval_params,
        };
        let target_day_index = if target_day >= plan.schedule.len() {
            0
        } else {
            target_day
        };
        plan.schedule[target_day_index]
            .segments
            .push(Segment::Straight(straight_segment));
        app_state.mark_modified();
        if let Some((work, rest, repeats)) = interval {
            println!(
                "Added time-based set with intervals: {} ({}x{}s, {}s work/{}s rest × {} repeats)",
                ex_label, sets, time_sec, work, rest, repeats
            );
        } else {
            println!(
                "Added time-based set: {} ({}x{}s, rest {}s)",
                ex_label, sets, time_sec, rest_sec
            );
        }
        drop(app_state);
        update_canvas_content(state.clone());
    } else {
        println!("No plan open. Create a new plan first.");
    }
}

pub fn add_scheme_to_plan(
    state: Arc<Mutex<AppState>>,
    ex_code: String,
    ex_label: String,
    top_reps: u32,
    top_rpe: f64,
    _top_rest: u32,
    backoff_sets: u32,
    backoff_reps: u32,
    backoff_percent: f64,
    _backoff_rest: u32,
) {
    use weightlifting_core::{BaseSegment, RpeOrRange, SchemeSegment, SchemeSet};
    let mut app_state = state.lock().unwrap();
    let target_day = app_state.get_and_clear_target_day().unwrap_or(0);
    app_state.save_to_undo_history();

    if let Some(plan) = &mut app_state.current_plan {
        if !plan.dictionary.contains_key(&ex_code) {
            plan.dictionary.insert(ex_code.clone(), ex_label.clone());
        }
        if plan.schedule.is_empty() {
            let day = Day {
                day: 1,
                label: "Training Day".to_string(),
                time_cap_min: None,
                goal: None,
                equipment_policy: None,
                segments: vec![],
            };
            plan.schedule.push(day);
        }
        let top_set = SchemeSet {
            label: Some("Top set".to_string()),
            sets: Some(1),
            reps: Some(RepsOrRange::Range(RepsRange {
                min: top_reps,
                max: top_reps,
                target: Some(top_reps),
            })),
            time_sec: None,
            rpe: Some(RpeOrRange::Fixed(top_rpe)),
            rest_sec: None,
            anchor: None,
            track_pr: None,
        };
        let backoff_set = SchemeSet {
            label: Some(format!("Backoff @ {}%", (backoff_percent * 100.0) as u32)),
            sets: Some(backoff_sets),
            reps: Some(RepsOrRange::Range(RepsRange {
                min: backoff_reps,
                max: backoff_reps,
                target: Some(backoff_reps),
            })),
            time_sec: None,
            rpe: None,
            rest_sec: None,
            anchor: None,
            track_pr: None,
        };
        let scheme_segment = SchemeSegment {
            base: BaseSegment {
                ex: ex_code,
                alt_group: None,
                label: Some(ex_label.clone()),
                optional: None,
                technique: None,
                equipment_policy: None,
            },
            sets: vec![top_set, backoff_set],
            load_mode: None,
            template: None,
        };
        let target_day_index = if target_day >= plan.schedule.len() {
            0
        } else {
            target_day
        };
        plan.schedule[target_day_index]
            .segments
            .push(Segment::Scheme(scheme_segment));
        app_state.mark_modified();
        println!("Added scheme for {}", ex_label);
        drop(app_state);
        update_canvas_content(state.clone());
    } else {
        println!("No plan open. Create a new plan first.");
    }
}

pub fn add_complex_to_plan(
    state: Arc<Mutex<AppState>>,
    mode: String,
    anchor_ex: String,
    pct: Option<f64>,
    kg: Option<f64>,
    sets: u32,
    seq_ex: String,
    seq_reps: u32,
) {
    use weightlifting_core::{
        AnchorLoad, ComplexSegment, ComplexSequenceItem, RepsOrRange, RepsRange,
    };
    let mut app_state = state.lock().unwrap();
    let target_day = app_state.get_and_clear_target_day().unwrap_or(0);
    app_state.save_to_undo_history();

    if let Some(plan) = &mut app_state.current_plan {
        if !plan.dictionary.contains_key(&anchor_ex) {
            plan.dictionary.insert(anchor_ex.clone(), anchor_ex.clone());
        }
        if !plan.dictionary.contains_key(&seq_ex) {
            plan.dictionary.insert(seq_ex.clone(), seq_ex.clone());
        }
        if plan.schedule.is_empty() {
            let day = Day {
                day: 1,
                label: "Training Day".to_string(),
                time_cap_min: None,
                goal: None,
                equipment_policy: None,
                segments: vec![],
            };
            plan.schedule.push(day);
        }
        let anchor_load = AnchorLoad {
            mode,
            ex: if !anchor_ex.is_empty() {
                Some(anchor_ex)
            } else {
                None
            },
            pct,
            kg,
        };
        let sequence = vec![ComplexSequenceItem {
            ex: seq_ex.clone(),
            reps: RepsOrRange::Range(RepsRange {
                min: seq_reps,
                max: seq_reps,
                target: None,
            }),
            alt_group: None,
        }];
        let complex_segment = ComplexSegment {
            anchor_load,
            sets,
            rest_sec: 120,
            sequence,
        };
        let target_day_index = if target_day >= plan.schedule.len() {
            0
        } else {
            target_day
        };
        plan.schedule[target_day_index]
            .segments
            .push(Segment::Complex(complex_segment));
        app_state.mark_modified();
        println!(
            "Added complex: {} sets with sequence exercise {}",
            sets, seq_ex
        );
        drop(app_state);
        update_canvas_content(state.clone());
    } else {
        println!("No plan open. Create a new plan first.");
    }
}

pub fn update_circuit_in_plan(
    state: Arc<Mutex<AppState>>,
    day_index: usize,
    segment_index: usize,
    rounds: u32,
    rest_sec: u32,
    rest_between_rounds_sec: u32,
    exercises: Vec<CircuitExerciseData>,
) {
    use weightlifting_core::{CircuitItem, CircuitSegment, RepsOrRange, RepsRange, TimeOrRange};
    let mut app_state = state.lock().unwrap();
    app_state.save_to_undo_history();

    if let Some(plan) = &mut app_state.current_plan {
        if day_index < plan.schedule.len()
            && segment_index < plan.schedule[day_index].segments.len()
        {
            for exercise in &exercises {
                if !plan.dictionary.contains_key(&exercise.ex_code) {
                    plan.dictionary
                        .insert(exercise.ex_code.clone(), exercise.ex_name.clone());
                }
            }
            let items: Vec<CircuitItem> = exercises
                .iter()
                .map(|exercise| {
                    let reps = if exercise.time_sec.is_some() {
                        None
                    } else {
                        Some(RepsOrRange::Range(RepsRange {
                            min: exercise.reps_min,
                            max: exercise.reps_max,
                            target: None,
                        }))
                    };
                    let time_sec = exercise.time_sec.map(TimeOrRange::Fixed);
                    CircuitItem {
                        ex: exercise.ex_code.clone(),
                        reps,
                        time_sec,
                        alt_group: None,
                    }
                })
                .collect();
            let updated_circuit = CircuitSegment {
                rounds,
                rest_sec,
                rest_between_rounds_sec,
                items,
            };
            plan.schedule[day_index].segments[segment_index] = Segment::Circuit(updated_circuit);
            app_state.mark_modified();
            println!(
                "Updated circuit with {} exercises ({}x rounds, {}s rest, {}s between rounds)",
                exercises.len(),
                rounds,
                rest_sec,
                rest_between_rounds_sec
            );
            drop(app_state);
            update_canvas_content(state.clone());
        } else {
            println!("Invalid day or segment index for circuit update");
        }
    } else {
        println!("No plan open. Create a new plan first.");
    }
}

#[allow(dead_code)]
pub fn update_scheme_in_plan(
    state: Arc<Mutex<AppState>>,
    day_index: usize,
    segment_index: usize,
    ex_code: String,
    ex_label: Option<String>,
) {
    let mut app_state = state.lock().unwrap();
    app_state.save_to_undo_history();
    if let Some(plan) = &mut app_state.current_plan {
        if day_index < plan.schedule.len()
            && segment_index < plan.schedule[day_index].segments.len()
        {
            if !plan.dictionary.contains_key(&ex_code) {
                let display_name = ex_label.clone().unwrap_or_else(|| ex_code.clone());
                plan.dictionary.insert(ex_code.clone(), display_name);
            }
            if let Segment::Scheme(ref mut scheme) =
                plan.schedule[day_index].segments[segment_index]
            {
                scheme.base.ex = ex_code.clone();
                scheme.base.label = ex_label.clone();
                app_state.mark_modified();
                println!(
                    "Updated scheme exercise to {} with label {:?}",
                    ex_code, ex_label
                );
                drop(app_state);
                update_canvas_content(state.clone());
            } else {
                println!("Segment is not a scheme");
            }
        } else {
            println!("Invalid day or segment index for scheme update");
        }
    } else {
        println!("No plan open. Create a new plan first.");
    }
}

pub fn update_scheme_full_in_plan(
    state: Arc<Mutex<AppState>>,
    day_index: usize,
    segment_index: usize,
    ex_code: String,
    ex_label: Option<String>,
    alt_group: Option<String>,
    sets: Vec<SchemeSetData>,
) {
    use weightlifting_core::{
        RepsOrRange, RepsRange, RestOrRange, RpeOrRange, SchemeSet, Segment, TimeOrRange,
    };
    let mut app_state = state.lock().unwrap();
    app_state.save_to_undo_history();
    if let Some(plan) = &mut app_state.current_plan {
        if day_index < plan.schedule.len()
            && segment_index < plan.schedule[day_index].segments.len()
        {
            if !plan.dictionary.contains_key(&ex_code) {
                let display_name = ex_label.clone().unwrap_or_else(|| ex_code.clone());
                plan.dictionary.insert(ex_code.clone(), display_name);
            }
            if let Segment::Scheme(ref mut scheme) =
                plan.schedule[day_index].segments[segment_index]
            {
                // Update base
                scheme.base.ex = ex_code.clone();
                scheme.base.label = ex_label.clone();
                scheme.base.alt_group = alt_group.clone();
                // Map SchemeSetData -> SchemeSet
                let mapped: Vec<SchemeSet> = sets
                    .into_iter()
                    .map(|s| {
                        let reps = match (s.reps_min, s.reps_max) {
                            (Some(min), Some(max)) => Some(RepsOrRange::Range(RepsRange {
                                min,
                                max,
                                target: None,
                            })),
                            _ => None,
                        };
                        let time_sec = s.time_sec.map(TimeOrRange::Fixed);
                        let rpe = s.rpe.map(RpeOrRange::Fixed);
                        let rest_sec = s.rest_sec.map(RestOrRange::Fixed);
                        SchemeSet {
                            label: s.label,
                            sets: s.sets,
                            reps,
                            time_sec,
                            rpe,
                            rest_sec,
                            anchor: None,
                            track_pr: None,
                        }
                    })
                    .collect();
                scheme.sets = mapped;
                let updated_count = scheme.sets.len();
                let _ = scheme;
                app_state.mark_modified();
                println!("Updated scheme with {} sets for {}", updated_count, ex_code);
                drop(app_state);
                update_canvas_content(state.clone());
            } else {
                println!("Segment is not a scheme");
            }
        } else {
            println!("Invalid day or segment index for scheme update");
        }
    } else {
        println!("No plan open. Create a new plan first.");
    }
}

pub fn update_complex_in_plan(
    state: Arc<Mutex<AppState>>,
    day_index: usize,
    segment_index: usize,
    mode: String,
    anchor_ex: String,
    pct: Option<f64>,
    kg: Option<f64>,
    sets: u32,
    rest_sec: u32,
) {
    use weightlifting_core::AnchorLoad;
    let mut app_state = state.lock().unwrap();
    app_state.save_to_undo_history();
    if let Some(plan) = &mut app_state.current_plan {
        if day_index < plan.schedule.len()
            && segment_index < plan.schedule[day_index].segments.len()
        {
            if !anchor_ex.is_empty() && !plan.dictionary.contains_key(&anchor_ex) {
                plan.dictionary.insert(anchor_ex.clone(), anchor_ex.clone());
            }
            if let Segment::Complex(ref mut complex) =
                plan.schedule[day_index].segments[segment_index]
            {
                complex.anchor_load = AnchorLoad {
                    mode,
                    ex: if !anchor_ex.is_empty() {
                        Some(anchor_ex)
                    } else {
                        None
                    },
                    pct,
                    kg,
                };
                complex.sets = sets;
                complex.rest_sec = rest_sec;
                app_state.mark_modified();
                println!("Updated complex: {} sets, {}s rest", sets, rest_sec);
                drop(app_state);
                update_canvas_content(state.clone());
            } else {
                println!("Segment is not a complex");
            }
        } else {
            println!("Invalid day or segment index for complex update");
        }
    } else {
        println!("No plan open. Create a new plan first.");
    }
}

pub fn add_group_choose_to_plan(
    state: Arc<Mutex<AppState>>,
    pick: u32,
    rotation: Option<String>,
    exercises: Vec<(String, String)>,
) {
    let mut app_state = state.lock().unwrap();
    let target_day = app_state.get_and_clear_target_day().unwrap_or(0);
    app_state.save_to_undo_history();
    if let Some(plan) = &mut app_state.current_plan {
        for (ex_code, ex_label) in &exercises {
            if !plan.dictionary.contains_key(ex_code) {
                plan.dictionary.insert(ex_code.clone(), ex_label.clone());
            }
        }
        if plan.schedule.is_empty() {
            let day = Day {
                day: 1,
                label: "Training Day".to_string(),
                time_cap_min: None,
                goal: None,
                equipment_policy: None,
                segments: vec![],
            };
            plan.schedule.push(day);
        }
        let target_day_index = if target_day >= plan.schedule.len() {
            0
        } else {
            target_day
        };
        let from_segments: Vec<Segment> = exercises
            .into_iter()
            .map(|(ex_code, ex_label)| {
                let straight_segment = StraightSegment {
                    base: BaseSegment {
                        ex: ex_code,
                        alt_group: None,
                        label: Some(ex_label),
                        optional: None,
                        technique: None,
                        equipment_policy: None,
                    },
                    sets: Some(3),
                    sets_range: None,
                    reps: Some(RepsOrRange::Range(RepsRange {
                        min: 8,
                        max: 12,
                        target: None,
                    })),
                    time_sec: None,
                    rest_sec: None,
                    rir: None,
                    rpe: None,
                    tempo: None,
                    vbt: None,
                    load_mode: None,
                    intensifier: None,
                    auto_stop: None,
                    interval: None,
                };
                Segment::Straight(straight_segment)
            })
            .collect();

        let group_choose_segment = GroupChooseSegment {
            pick,
            rotation,
            from: from_segments,
            state: Some(GroupState::new()),
        };
        plan.schedule[target_day_index]
            .segments
            .push(Segment::GroupChoose(group_choose_segment));
        app_state.mark_modified();
        println!("Added group (choose) segment with exercises, pick {}", pick);
        drop(app_state);
        update_canvas_content(state.clone());
    } else {
        println!("No plan open. Create a new plan first.");
    }
}
