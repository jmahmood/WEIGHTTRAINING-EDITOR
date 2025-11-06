use serde_json::{json, Value};
use chrono::NaiveDate;
use std::collections::HashMap;

/// Data transformation utilities for converting metrics to chart-ready format
pub struct DataTransforms;

impl DataTransforms {
    /// Transform E1RM data points for line chart visualization
    pub fn e1rm_to_chart_data(
        e1rm_data: &[(String, NaiveDate, f64)], // (exercise, date, e1rm_kg)
        exercise_filter: Option<&str>
    ) -> Vec<Value> {
        e1rm_data.iter()
            .filter(|(exercise, _, _)| {
                exercise_filter.is_none_or(|filter| exercise == filter)
            })
            .map(|(exercise, date, e1rm)| {
                json!({
                    "exercise": exercise,
                    "date": date.format("%Y-%m-%d").to_string(),
                    "e1rm_kg": e1rm,
                    "e1rm_lb": e1rm * 2.20462 // Convert to lbs for display
                })
            })
            .collect()
    }
    
    /// Transform volume data for stacked bar chart
    pub fn volume_to_chart_data(
        volume_data: &[(String, NaiveDate, u32, u32, f64)], // (category, week_start, sets, reps, tonnage)
        body_part_map: &HashMap<String, String> // exercise -> body part mapping
    ) -> Vec<Value> {
        let mut grouped_data: HashMap<NaiveDate, HashMap<String, (u32, u32, f64)>> = HashMap::new();
        
        // Group by week and body part
        for (exercise, week_start, sets, reps, tonnage) in volume_data {
            let body_part = body_part_map.get(exercise)
                .unwrap_or(exercise)
                .clone();
            
            let week_entry = grouped_data.entry(*week_start).or_default();
            let body_part_entry = week_entry.entry(body_part).or_insert((0, 0, 0.0));
            
            body_part_entry.0 += sets;
            body_part_entry.1 += reps;
            body_part_entry.2 += tonnage;
        }
        
        // Convert to chart data format
        let mut chart_data = Vec::new();
        for (week_start, body_parts) in grouped_data {
            for (body_part, (sets, reps, tonnage)) in body_parts {
                chart_data.push(json!({
                    "week": week_start.format("%Y-%m-%d").to_string(),
                    "body_part": body_part,
                    "sets": sets,
                    "reps": reps,
                    "tonnage_kg": tonnage,
                    "tonnage_lb": tonnage * 2.20462
                }));
            }
        }
        
        // Sort by week for consistent ordering
        chart_data.sort_by(|a, b| {
            a["week"].as_str().cmp(&b["week"].as_str())
        });
        
        chart_data
    }
    
    /// Transform PR data for table/bar chart display
    pub fn pr_to_chart_data(
        pr_data: &[(String, String, NaiveDate, f64, Option<u32>)], // (exercise, pr_type, date, value, reps)
        exercise_filter: Option<&str>
    ) -> Vec<Value> {
        pr_data.iter()
            .filter(|(exercise, _, _, _, _)| {
                exercise_filter.is_none_or(|filter| exercise == filter)
            })
            .map(|(exercise, pr_type, date, value, reps)| {
                json!({
                    "exercise": exercise,
                    "pr_type": pr_type,
                    "date": date.format("%Y-%m-%d").to_string(),
                    "value_kg": value,
                    "value_lb": value * 2.20462,
                    "reps": reps,
                    "display_text": if let Some(r) = reps {
                        format!("{:.1}kg x{}", value, r)
                    } else {
                        format!("{:.1}kg", value)
                    }
                })
            })
            .collect()
    }
    
    /// Transform session frequency data for heatmap
    pub fn session_frequency_to_chart_data(
        session_dates: &[NaiveDate],
        start_date: NaiveDate,
        end_date: NaiveDate
    ) -> Vec<Value> {
        // Count sessions per date
        let mut session_counts: HashMap<NaiveDate, u32> = HashMap::new();
        for date in session_dates {
            if *date >= start_date && *date <= end_date {
                *session_counts.entry(*date).or_insert(0) += 1;
            }
        }
        
        // Generate data for all dates in range (including zeros)
        let mut chart_data = Vec::new();
        let mut current_date = start_date;
        
        while current_date <= end_date {
            let count = session_counts.get(&current_date).unwrap_or(&0);
            
            chart_data.push(json!({
                "date": current_date.format("%Y-%m-%d").to_string(),
                "day_of_week": current_date.format("%u").to_string(), // 1-7 (Mon-Sun)
                "week_of_year": current_date.format("%W").to_string(),
                "session_count": count,
                "intensity": if *count == 0 { 0 } else if *count == 1 { 1 } else { 2 } // 0=none, 1=single, 2=multiple
            }));
            
            current_date = current_date.succ_opt().unwrap_or(current_date);
            if current_date <= current_date.pred_opt().unwrap_or(current_date) {
                break; // Prevent infinite loop
            }
        }
        
        chart_data
    }
    
    /// Get body part mapping for common exercises
    pub fn get_default_body_part_map() -> HashMap<String, String> {
        let mut map = HashMap::new();
        
        // Chest exercises
        map.insert("BP.BB.FLAT".to_string(), "Chest".to_string());
        map.insert("BP.DB.FLAT".to_string(), "Chest".to_string());
        map.insert("BP.BB.INCL".to_string(), "Chest".to_string());
        map.insert("FLY.DB.FLAT".to_string(), "Chest".to_string());
        map.insert("DIP.BW.CHEST".to_string(), "Chest".to_string());
        
        // Back exercises
        map.insert("ROW.BB.BENT".to_string(), "Back".to_string());
        map.insert("ROW.DB.BENT".to_string(), "Back".to_string());
        map.insert("LAT.LAT.WIDE".to_string(), "Back".to_string());
        map.insert("PULLUP.BW.WIDE".to_string(), "Back".to_string());
        map.insert("PULLUP.BW.NEU".to_string(), "Back".to_string());
        map.insert("DL.BB.CONV".to_string(), "Back".to_string());
        
        // Shoulders
        map.insert("OHP.BB.STND".to_string(), "Shoulders".to_string());
        map.insert("OHP.DB.SEAT".to_string(), "Shoulders".to_string());
        map.insert("LAT.DB.SIDE".to_string(), "Shoulders".to_string());
        map.insert("REAR.DB.BENT".to_string(), "Shoulders".to_string());
        
        // Arms
        map.insert("CURL.BB.STND".to_string(), "Arms".to_string());
        map.insert("CURL.DB.ALT".to_string(), "Arms".to_string());
        map.insert("TRIC.DIP.CLOSE".to_string(), "Arms".to_string());
        map.insert("TRIC.EXT.OH".to_string(), "Arms".to_string());
        
        // Legs
        map.insert("SQ.BB.BACK".to_string(), "Legs".to_string());
        map.insert("SQ.BB.FRONT".to_string(), "Legs".to_string());
        map.insert("LEG.PR.45".to_string(), "Legs".to_string());
        map.insert("LUNGE.DB.STND".to_string(), "Legs".to_string());
        map.insert("LEG.CURL.SEAT".to_string(), "Legs".to_string());
        map.insert("LEG.EXT.SEAT".to_string(), "Legs".to_string());
        map.insert("CALF.STND.BB".to_string(), "Legs".to_string());
        
        // Core
        map.insert("CORE.BW.PLNK".to_string(), "Core".to_string());
        map.insert("CORE.BW.CRUNCH".to_string(), "Core".to_string());
        map.insert("CORE.HANG.LEG".to_string(), "Core".to_string());
        
        map
    }
    
    /// Filter data by date range
    pub fn filter_by_date_range<T>(
        data: Vec<T>,
        extract_date: impl Fn(&T) -> NaiveDate,
        start_date: Option<NaiveDate>,
        end_date: Option<NaiveDate>
    ) -> Vec<T> {
        data.into_iter()
            .filter(|item| {
                let date = extract_date(item);
                if let Some(start) = start_date {
                    if date < start {
                        return false;
                    }
                }
                if let Some(end) = end_date {
                    if date > end {
                        return false;
                    }
                }
                true
            })
            .collect()
    }
    
    /// Get unique exercises from data
    pub fn get_unique_exercises(data: &[(String, NaiveDate, f64)]) -> Vec<String> {
        let mut exercises: Vec<String> = data.iter()
            .map(|(exercise, _, _)| exercise.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        exercises.sort();
        exercises
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_e1rm_chart_data() {
        let data = vec![
            ("BP.BB.FLAT".to_string(), NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(), 120.0),
            ("BP.BB.FLAT".to_string(), NaiveDate::from_ymd_opt(2025, 1, 8).unwrap(), 125.0),
            ("SQ.BB.BACK".to_string(), NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(), 150.0),
        ];
        
        // Test with no filter
        let chart_data = DataTransforms::e1rm_to_chart_data(&data, None);
        assert_eq!(chart_data.len(), 3);
        
        // Test with exercise filter
        let filtered_data = DataTransforms::e1rm_to_chart_data(&data, Some("BP.BB.FLAT"));
        assert_eq!(filtered_data.len(), 2);
        assert_eq!(filtered_data[0]["exercise"], "BP.BB.FLAT");
        assert_eq!(filtered_data[0]["e1rm_kg"], 120.0);
    }
    
    #[test]
    fn test_body_part_mapping() {
        let body_part_map = DataTransforms::get_default_body_part_map();
        
        assert_eq!(body_part_map.get("BP.BB.FLAT"), Some(&"Chest".to_string()));
        assert_eq!(body_part_map.get("SQ.BB.BACK"), Some(&"Legs".to_string()));
        assert_eq!(body_part_map.get("OHP.BB.STND"), Some(&"Shoulders".to_string()));
    }
    
    #[test]
    fn test_session_frequency_data() {
        let sessions = vec![
            NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(), // Same day twice
            NaiveDate::from_ymd_opt(2025, 1, 3).unwrap(),
        ];
        
        let start_date = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        let end_date = NaiveDate::from_ymd_opt(2025, 1, 3).unwrap();
        
        let chart_data = DataTransforms::session_frequency_to_chart_data(
            &sessions, start_date, end_date
        );
        
        assert_eq!(chart_data.len(), 3); // 3 days in range
        
        // First day should have 2 sessions
        assert_eq!(chart_data[0]["session_count"], 2);
        assert_eq!(chart_data[0]["intensity"], 2); // Multiple sessions
        
        // Second day should have 0 sessions
        assert_eq!(chart_data[1]["session_count"], 0);
        assert_eq!(chart_data[1]["intensity"], 0);
        
        // Third day should have 1 session
        assert_eq!(chart_data[2]["session_count"], 1);
        assert_eq!(chart_data[2]["intensity"], 1);
    }
}