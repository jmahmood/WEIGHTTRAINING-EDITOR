use crate::csv_parser::SessionRecord;
use chrono::{NaiveDate, Datelike, Duration};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, BTreeMap};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MetricsError {
    #[error("Invalid data: {0}")]
    InvalidData(String),
    #[error("Calculation error: {0}")]
    CalculationError(String),
}

/// E1RM data point for a specific exercise and date
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct E1RMDataPoint {
    pub exercise: String,
    pub date: NaiveDate,
    pub e1rm_kg: f64,
    pub source_weight: f64,
    pub source_reps: u32,
    pub source_rpe: Option<f64>,
    pub formula: String, // "epley", "brzycki", etc.
}

/// Weekly volume data aggregated by body part or exercise
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeDataPoint {
    pub category: String, // exercise code or body part
    pub week_start: NaiveDate, // Monday of the week
    pub total_sets: u32,
    pub total_reps: u32,
    pub total_tonnage_kg: f64,
    pub session_count: u32,
}

/// Personal record data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PRDataPoint {
    pub exercise: String,
    pub pr_type: String, // "1RM", "5RM", "volume", etc.
    pub date: NaiveDate,
    pub value: f64, // weight in kg or total volume
    pub reps: Option<u32>,
    pub notes: Option<String>,
}

/// Calculator for estimated 1RM values
pub struct E1RMCalculator {
    // Could add configuration for preferred formulas, bodyweight handling, etc.
}

impl Default for E1RMCalculator {
    fn default() -> Self {
        Self::new()
    }
}

impl E1RMCalculator {
    pub fn new() -> Self {
        Self {}
    }
    
    /// Calculate historical E1RM data from session records
    pub fn calculate_historical_e1rms(
        &self,
        sessions: &[SessionRecord]
    ) -> Result<Vec<E1RMDataPoint>, MetricsError> {
        let mut e1rm_data = Vec::new();
        
        for record in sessions {
            if !record.can_calculate_e1rm() {
                continue;
            }
            
            let reps = record.reps.unwrap();
            let weight_kg = self.convert_to_kg(record.weight.unwrap_or(0.0), &record.unit)?;
            
            // Use Epley formula as default, with RPE adjustment if available
            let e1rm = if let Some(rpe) = record.rpe {
                self.calculate_e1rm_with_rpe(weight_kg, reps, rpe)
            } else {
                self.calculate_epley_e1rm(weight_kg, reps)
            };
            
            e1rm_data.push(E1RMDataPoint {
                exercise: record.ex_code.clone(),
                date: record.date,
                e1rm_kg: e1rm,
                source_weight: weight_kg,
                source_reps: reps,
                source_rpe: record.rpe,
                formula: if record.rpe.is_some() { "epley_rpe".to_string() } else { "epley".to_string() },
            });
        }
        
        Ok(e1rm_data)
    }
    
    /// Epley formula: 1RM = weight * (1 + reps/30)
    fn calculate_epley_e1rm(&self, weight: f64, reps: u32) -> f64 {
        weight * (1.0 + (reps as f64 / 30.0))
    }
    
    /// Enhanced Epley with RPE adjustment
    fn calculate_e1rm_with_rpe(&self, weight: f64, reps: u32, rpe: f64) -> f64 {
        // RPE adjustment: higher RPE means closer to true max
        // This is a simplified model - could be enhanced with lookup tables
        let rpe_adjustment = match rpe {
            r if r >= 9.5 => 1.0,
            r if r >= 9.0 => 0.97,
            r if r >= 8.5 => 0.94,
            r if r >= 8.0 => 0.91,
            r if r >= 7.5 => 0.88,
            _ => 0.85,
        };
        
        let base_e1rm = self.calculate_epley_e1rm(weight, reps);
        base_e1rm / rpe_adjustment
    }
    
    /// Convert weight to kg based on unit
    fn convert_to_kg(&self, weight: f64, unit: &str) -> Result<f64, MetricsError> {
        match unit {
            "kg" => Ok(weight),
            "lb" => Ok(weight * 0.453592), // lb to kg
            "bw" => {
                // For bodyweight exercises, we'd need user bodyweight
                // For now, assume 75kg average - this could be configurable
                Ok(75.0)
            },
            _ => Err(MetricsError::InvalidData(format!("Unknown unit: {}", unit)))
        }
    }
}

/// Calculator for volume metrics
pub struct VolumeCalculator {
    // Could add body part mappings, etc.
}

impl Default for VolumeCalculator {
    fn default() -> Self {
        Self::new()
    }
}

impl VolumeCalculator {
    pub fn new() -> Self {
        Self {}
    }
    
    /// Calculate weekly volume data from session records
    pub fn calculate_weekly_volumes(
        &self,
        sessions: &[SessionRecord]
    ) -> Result<Vec<VolumeDataPoint>, MetricsError> {
        // Group by exercise and week
        let mut weekly_data: BTreeMap<(String, NaiveDate), VolumeAggregator> = BTreeMap::new();
        let mut session_counts: BTreeMap<(String, NaiveDate), std::collections::HashSet<String>> = BTreeMap::new();
        
        for record in sessions {
            if !record.is_working_set() {
                continue;
            }
            
            let week_start = self.get_week_start(record.date);
            let exercise = record.ex_code.clone();
            let key = (exercise.clone(), week_start);
            
            let aggregator = weekly_data.entry(key.clone()).or_insert(VolumeAggregator::new());
            
            // Add set data
            aggregator.total_sets += 1;
            
            if let Some(reps) = record.reps {
                aggregator.total_reps += reps;
                
                // Calculate tonnage if we have weight
                if let Some(weight) = record.weight {
                    let weight_kg = self.convert_to_kg(weight, &record.unit)?;
                    aggregator.total_tonnage_kg += weight_kg * (reps as f64);
                }
            }
            
            // Track unique sessions
            session_counts.entry(key.clone()).or_default()
                .insert(record.session_id.clone());
        }
        
        // Convert to output format
        let mut volume_data = Vec::new();
        for ((exercise, week_start), aggregator) in weekly_data {
            let session_count = session_counts.get(&(exercise.clone(), week_start))
                .map(|set| set.len() as u32)
                .unwrap_or(0);
                
            volume_data.push(VolumeDataPoint {
                category: exercise,
                week_start,
                total_sets: aggregator.total_sets,
                total_reps: aggregator.total_reps,
                total_tonnage_kg: aggregator.total_tonnage_kg,
                session_count,
            });
        }
        
        Ok(volume_data)
    }
    
    /// Get the Monday of the week for a given date
    fn get_week_start(&self, date: NaiveDate) -> NaiveDate {
        let days_from_monday = date.weekday().num_days_from_monday() as i64;
        date - Duration::days(days_from_monday)
    }
    
    /// Convert weight to kg (reused from E1RMCalculator)
    fn convert_to_kg(&self, weight: f64, unit: &str) -> Result<f64, MetricsError> {
        match unit {
            "kg" => Ok(weight),
            "lb" => Ok(weight * 0.453592),
            "bw" => Ok(75.0), // Default bodyweight
            _ => Err(MetricsError::InvalidData(format!("Unknown unit: {}", unit)))
        }
    }
}

/// Helper struct for aggregating volume data
struct VolumeAggregator {
    total_sets: u32,
    total_reps: u32,
    total_tonnage_kg: f64,
}

impl VolumeAggregator {
    fn new() -> Self {
        Self {
            total_sets: 0,
            total_reps: 0,
            total_tonnage_kg: 0.0,
        }
    }
}

/// Personal record tracker
pub struct PRTracker {
    // Could add configuration for PR types to track
}

impl Default for PRTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl PRTracker {
    pub fn new() -> Self {
        Self {}
    }
    
    /// Identify personal records from session data
    pub fn identify_prs(&self, sessions: &[SessionRecord]) -> Result<Vec<PRDataPoint>, MetricsError> {
        let mut pr_data = Vec::new();
        
        // Group by exercise and track max weights for different rep ranges
        let mut exercise_maxes: HashMap<String, HashMap<u32, (f64, NaiveDate)>> = HashMap::new();
        
        for record in sessions {
            if !record.can_calculate_e1rm() {
                continue;
            }
            
            let reps = record.reps.unwrap();
            let weight_kg = self.convert_to_kg(record.weight.unwrap_or(0.0), &record.unit)?;
            
            let exercise_entry = exercise_maxes.entry(record.ex_code.clone()).or_default();
            
            // Check if this is a new PR for this rep count
            let is_pr = match exercise_entry.get(&reps) {
                Some((current_max, _)) => weight_kg > *current_max,
                None => true, // First time doing this rep count
            };
            
            if is_pr {
                exercise_entry.insert(reps, (weight_kg, record.date));
                
                // Determine PR type
                let pr_type = match reps {
                    1 => "1RM".to_string(),
                    2..=3 => format!("{}RM", reps),
                    4..=6 => "Strength".to_string(),
                    7..=12 => "Volume".to_string(),
                    _ => "Endurance".to_string(),
                };
                
                pr_data.push(PRDataPoint {
                    exercise: record.ex_code.clone(),
                    pr_type,
                    date: record.date,
                    value: weight_kg,
                    reps: Some(reps),
                    notes: record.notes.clone(),
                });
            }
        }
        
        Ok(pr_data)
    }
    
    /// Convert weight to kg (reused from other calculators)
    fn convert_to_kg(&self, weight: f64, unit: &str) -> Result<f64, MetricsError> {
        match unit {
            "kg" => Ok(weight),
            "lb" => Ok(weight * 0.453592),
            "bw" => Ok(75.0), // Default bodyweight
            _ => Err(MetricsError::InvalidData(format!("Unknown unit: {}", unit)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_epley_calculation() {
        let calc = E1RMCalculator::new();
        
        // 100kg x 5 reps should give ~116.7kg 1RM
        let e1rm = calc.calculate_epley_e1rm(100.0, 5);
        assert!((e1rm - 116.67).abs() < 0.1);
    }
    
    #[test]
    fn test_week_start_calculation() {
        let calc = VolumeCalculator::new();
        
        // Test various days of week
        let tuesday = NaiveDate::from_ymd_opt(2025, 8, 19).unwrap(); // Tuesday
        let monday = calc.get_week_start(tuesday);
        assert_eq!(monday, NaiveDate::from_ymd_opt(2025, 8, 18).unwrap()); // Previous Monday
        
        let friday = NaiveDate::from_ymd_opt(2025, 8, 22).unwrap(); // Friday
        let monday = calc.get_week_start(friday);
        assert_eq!(monday, NaiveDate::from_ymd_opt(2025, 8, 18).unwrap()); // Same Monday
    }
}