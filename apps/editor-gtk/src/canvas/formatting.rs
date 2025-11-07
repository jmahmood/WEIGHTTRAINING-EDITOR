use std::collections::HashMap;
use weightlifting_core::Segment;

fn name_for(ex: &str, label: Option<&String>, dict: Option<&HashMap<String, String>>) -> String {
    if let Some(d) = dict {
        if let Some(name) = d.get(ex) {
            return name.clone();
        }
    }
    // Fall back to label or code
    label.cloned().unwrap_or_else(|| ex.to_string())
}

pub fn format_segment(segment: &Segment, dict: Option<&HashMap<String, String>>) -> String {
    match segment {
        weightlifting_core::Segment::Straight(s) => format!(
            "{}: {} sets × {}",
            name_for(&s.base.ex, s.base.label.as_ref(), dict),
            s.sets.unwrap_or(1),
            match &s.reps {
                Some(weightlifting_core::RepsOrRange::Range(r)) =>
                    format!("{}-{} reps", r.min, r.max),
                None => "unspecified reps".to_string(),
            }
        ),
        weightlifting_core::Segment::Amrap(a) => format!(
            "{}: AMRAP {}-{} reps",
            name_for(&a.base.ex, a.base.label.as_ref(), dict),
            a.base_reps,
            a.cap_reps
        ),
        weightlifting_core::Segment::Rpe(r) => format!(
            "{}: {} sets @ RPE {}",
            name_for(&r.base.ex, r.base.label.as_ref(), dict),
            r.sets,
            r.rpe
        ),
        weightlifting_core::Segment::Percentage(p) => format!(
            "{}: {} prescriptions",
            name_for(&p.base.ex, p.base.label.as_ref(), dict),
            p.prescriptions.len()
        ),
        weightlifting_core::Segment::Comment(c) => {
            let icon = c.icon.as_deref().unwrap_or("🏋️");
            format!("{} {}", icon, c.text)
        }
        weightlifting_core::Segment::Superset(s) => {
            let label = s.label.as_deref().unwrap_or("Superset");
            let exercises: Vec<String> = s
                .items
                .iter()
                .map(|item| {
                    if let Some(reps) = &item.reps {
                        match reps {
                            weightlifting_core::RepsOrRange::Range(r) => {
                                if r.min == r.max {
                                    format!("{}x{}", item.sets, r.min)
                                } else {
                                    format!("{}x{}-{}", item.sets, r.min, r.max)
                                }
                            }
                        }
                    } else {
                        format!("{}x?", item.sets)
                    }
                })
                .collect();
            format!(
                "🔗 {} ({} rounds): {}",
                label,
                s.rounds,
                exercises.join(" + ")
            )
        }
        weightlifting_core::Segment::Circuit(c) => {
            let exercises: Vec<String> = c
                .items
                .iter()
                .map(|item| {
                    if let Some(time_sec) = &item.time_sec {
                        match time_sec {
                            weightlifting_core::TimeOrRange::Fixed(time) => format!("{}s", time),
                            _ => "time".to_string(),
                        }
                    } else if let Some(reps) = &item.reps {
                        match reps {
                            weightlifting_core::RepsOrRange::Range(r) => {
                                if r.min == r.max {
                                    format!("{}", r.min)
                                } else {
                                    format!("{}-{}", r.min, r.max)
                                }
                            }
                        }
                    } else {
                        "?".to_string()
                    }
                })
                .collect();
            format!(
                "🔄 Circuit ({} rounds): {}",
                c.rounds,
                exercises.join(" → ")
            )
        }
        weightlifting_core::Segment::GroupChoose(g) => {
            format!("🎯 Choose {} from: {} options", g.pick, g.from.len())
        }
        weightlifting_core::Segment::Scheme(s) => {
            let label = name_for(&s.base.ex, s.base.label.as_ref(), dict);
            let sets_count = s.sets.len();
            let sets_summary = if sets_count > 0 {
                let first_set = &s.sets[0];
                if let Some(sets) = first_set.sets {
                    if sets_count == 1 {
                        format!("{} sets", sets)
                    } else {
                        format!("{} sets + {} more", sets, sets_count - 1)
                    }
                } else {
                    format!("{} scheme sets", sets_count)
                }
            } else {
                "empty scheme".to_string()
            };
            format!("📋 Scheme {}: {}", label, sets_summary)
        }
        weightlifting_core::Segment::Complex(c) => {
            let anchor_description = match c.anchor_load.mode.as_str() {
                "pct_1rm" => {
                    if let Some(pct) = c.anchor_load.pct {
                        format!("{}%", (pct * 100.0) as u32)
                    } else {
                        "pct".to_string()
                    }
                }
                "fixed_kg" => {
                    if let Some(kg) = c.anchor_load.kg {
                        format!("{}kg", kg)
                    } else {
                        "fixed".to_string()
                    }
                }
                _ => c.anchor_load.mode.clone(),
            };
            let sequence_exercises = c
                .sequence
                .iter()
                .map(|item| {
                    if let Some(d) = dict {
                        d.get(&item.ex).cloned().unwrap_or(item.ex.clone())
                    } else {
                        item.ex.clone()
                    }
                })
                .collect::<Vec<_>>()
                .join(" → ");
            format!(
                "🔀 Complex {} sets @ {}: {}",
                c.sets, anchor_description, sequence_exercises
            )
        }
        _ => format!("{:?}", segment),
    }
}
