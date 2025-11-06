use crate::state::AppState;
use std::sync::{Arc, Mutex};
use weightlifting_core::Segment;

/// Replace all references to an exercise code throughout the plan (schedule + groups).
pub fn replace_exercise_references(state: Arc<Mutex<AppState>>, from: &str, to: &str) {
    let mut s = state.lock().unwrap();
    if let Some(plan) = &mut s.current_plan {
        // Update schedule
        for day in &mut plan.schedule {
            for seg in &mut day.segments {
                replace_in_segment(seg, from, to);
            }
        }
        // Update groups
        for members in plan.groups.values_mut() {
            for mem in members.iter_mut() {
                if mem == from { *mem = to.to_string(); }
            }
        }
        s.mark_modified();
    }
}

fn replace_in_segment(seg: &mut Segment, from: &str, to: &str) {
    match seg {
        Segment::Straight(s) => {
            if s.base.ex == from { s.base.ex = to.to_string(); }
        }
        Segment::Rpe(s) => {
            if s.base.ex == from { s.base.ex = to.to_string(); }
        }
        Segment::Percentage(s) => {
            if s.base.ex == from { s.base.ex = to.to_string(); }
        }
        Segment::Amrap(s) => {
            if s.base.ex == from { s.base.ex = to.to_string(); }
        }
        Segment::Superset(sup) => {
            for item in &mut sup.items {
                if item.ex == from { item.ex = to.to_string(); }
            }
        }
        Segment::Circuit(circ) => {
            for item in &mut circ.items {
                if item.ex == from { item.ex = to.to_string(); }
            }
        }
        Segment::Scheme(sch) => {
            if sch.base.ex == from { sch.base.ex = to.to_string(); }
        }
        Segment::Complex(cmp) => {
            if let Some(ex) = &mut cmp.anchor_load.ex {
                if ex == from { *ex = to.to_string(); }
            }
            for item in &mut cmp.sequence {
                if item.ex == from { item.ex = to.to_string(); }
            }
        }
        Segment::GroupChoose(g) => {
            for s in &mut g.from { replace_in_segment(s, from, to); }
        }
        Segment::GroupRotate(g) => {
            for s in &mut g.items { replace_in_segment(s, from, to); }
        }
        Segment::GroupOptional(g) => {
            for s in &mut g.items { replace_in_segment(s, from, to); }
        }
        Segment::GroupSuperset(g) => {
            for s in &mut g.items { replace_in_segment(s, from, to); }
        }
        Segment::Time(t) => {
            if t.base.ex == from { t.base.ex = to.to_string(); }
        }
        Segment::Comment(_) => {}
    }
}

/// Rename a dictionary key and update references from old->new.
/// If `new_code` already exists, this function only updates references and removes the old dictionary key.
pub fn rename_exercise_code(state: Arc<Mutex<AppState>>, old_code: &str, new_code: &str) {
    let mut s = state.lock().unwrap();
    if let Some(plan) = &mut s.current_plan {
        // Move or remove dictionary entry
        if let Some(name) = plan.dictionary.remove(old_code) {
            plan.dictionary.entry(new_code.to_string()).or_insert(name);
        }
        // Update references
        drop(s);
        replace_exercise_references(state.clone(), old_code, new_code);
    }
}

/// Create a dictionary entry if missing.
pub fn ensure_dictionary_entry(state: Arc<Mutex<AppState>>, code: &str, name: &str) {
    let mut s = state.lock().unwrap();
    if let Some(plan) = &mut s.current_plan {
        plan.dictionary.entry(code.to_string()).or_insert(name.to_string());
        s.mark_modified();
    }
}
