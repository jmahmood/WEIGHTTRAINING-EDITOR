/// **Death to Windows!** - Location profiles and rounding algorithms for Sprint 2
/// Equipment-based load rounding that never mutates source data
use serde::{Deserialize, Serialize};

/// Location profile with equipment and rounding rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationProfile {
    pub id: String,
    pub name: String,
    pub bars: Vec<f64>,        // Available bar weights (kg)
    pub plate_pairs: Vec<f64>, // Available plate pairs (kg each)
    pub db_pairs: Vec<f64>,    // Available dumbbell pairs (kg each)
    pub increments: EquipmentIncrements,
    pub strategy: RoundingStrategy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquipmentIncrements {
    pub bb: f64, // Barbell increment (kg)
    pub db: f64, // Dumbbell increment (kg)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RoundingStrategy {
    Nearest, // Round to nearest achievable load
    Up,      // Always round up
    Down,    // Always round down
}

/// Result of rounding preview - never mutates source
#[derive(Debug, Clone)]
pub struct RoundingPreview {
    pub original_load: f64,
    pub rounded_load: f64,
    pub delta: f64,
    pub equipment: String, // "bb" or "db"
    pub plate_solution: Option<PlateSolution>,
}

#[derive(Debug, Clone)]
pub struct PlateSolution {
    pub bar_weight: f64,
    pub plates_per_side: Vec<(f64, u32)>, // (plate_weight, count_per_side)
    pub total_plates_needed: u32,
}

impl LocationProfile {
    /// Create a default home gym profile
    pub fn home_gym() -> Self {
        Self {
            id: "home".to_string(),
            name: "Home Gym".to_string(),
            bars: vec![20.0], // Standard Olympic bar
            plate_pairs: vec![1.25, 2.5, 5.0, 10.0, 15.0, 20.0],
            db_pairs: vec![2.5, 5.0, 7.5, 10.0, 12.5, 15.0, 17.5, 20.0],
            increments: EquipmentIncrements { bb: 2.5, db: 2.5 },
            strategy: RoundingStrategy::Nearest,
        }
    }

    /// Create a commercial gym profile
    pub fn commercial_gym() -> Self {
        Self {
            id: "gym".to_string(),
            name: "Commercial Gym".to_string(),
            bars: vec![20.0, 15.0], // Olympic and women's bars
            plate_pairs: vec![1.25, 2.5, 5.0, 10.0, 15.0, 20.0, 25.0],
            db_pairs: (1..=20).map(|x| x as f64 * 2.5).collect(), // 2.5kg increments up to 50kg
            increments: EquipmentIncrements { bb: 2.5, db: 2.5 },
            strategy: RoundingStrategy::Nearest,
        }
    }

    /// Round a target load for barbell exercises
    pub fn round_barbell_load(&self, target: f64) -> RoundingPreview {
        // Find the best bar
        let bar_weight = self
            .bars
            .iter()
            .find(|&&bar| bar <= target)
            .or(self.bars.first())
            .copied()
            .unwrap_or(20.0);

        let remaining_load = target - bar_weight;

        // Try to solve with available plates first
        if let Some(solution) = self.solve_with_plates(remaining_load, bar_weight) {
            return RoundingPreview {
                original_load: target,
                rounded_load: solution.bar_weight
                    + solution
                        .plates_per_side
                        .iter()
                        .map(|(weight, count)| weight * (*count as f64) * 2.0)
                        .sum::<f64>(),
                delta: solution.bar_weight
                    + solution
                        .plates_per_side
                        .iter()
                        .map(|(weight, count)| weight * (*count as f64) * 2.0)
                        .sum::<f64>()
                    - target,
                equipment: "bb".to_string(),
                plate_solution: Some(solution),
            };
        }

        // Fall back to increment-based rounding
        let increment = self.increments.bb;
        let rounded = match self.strategy {
            RoundingStrategy::Nearest => (target / increment).round() * increment,
            RoundingStrategy::Up => (target / increment).ceil() * increment,
            RoundingStrategy::Down => (target / increment).floor() * increment,
        };

        RoundingPreview {
            original_load: target,
            rounded_load: rounded,
            delta: rounded - target,
            equipment: "bb".to_string(),
            plate_solution: None,
        }
    }

    /// Round a target load for dumbbell exercises
    pub fn round_dumbbell_load(&self, target: f64) -> RoundingPreview {
        // Check if we have exact dumbbell weights
        if let Some(&exact_weight) = self.db_pairs.iter().find(|&&w| (w - target).abs() < 0.1) {
            return RoundingPreview {
                original_load: target,
                rounded_load: exact_weight,
                delta: exact_weight - target,
                equipment: "db".to_string(),
                plate_solution: None,
            };
        }

        // Find nearest available dumbbell based on strategy
        let rounded = match self.strategy {
            RoundingStrategy::Nearest => self
                .db_pairs
                .iter()
                .min_by(|&&a, &&b| (a - target).abs().partial_cmp(&(b - target).abs()).unwrap())
                .copied()
                .unwrap_or_else(|| {
                    let increment = self.increments.db;
                    (target / increment).round() * increment
                }),
            RoundingStrategy::Up => self
                .db_pairs
                .iter()
                .find(|&&w| w >= target)
                .copied()
                .unwrap_or_else(|| {
                    let increment = self.increments.db;
                    (target / increment).ceil() * increment
                }),
            RoundingStrategy::Down => self
                .db_pairs
                .iter()
                .rev()
                .find(|&&w| w <= target)
                .copied()
                .unwrap_or_else(|| {
                    let increment = self.increments.db;
                    (target / increment).floor() * increment
                }),
        };

        RoundingPreview {
            original_load: target,
            rounded_load: rounded,
            delta: rounded - target,
            equipment: "db".to_string(),
            plate_solution: None,
        }
    }

    /// Attempt to solve plate loading with available plates (greedy algorithm)
    fn solve_with_plates(&self, target_load: f64, bar_weight: f64) -> Option<PlateSolution> {
        // Need load per side (target - bar) / 2
        let load_per_side = target_load / 2.0;

        // Greedy algorithm: largest plates first
        let mut plates_per_side = Vec::new();
        let mut remaining = load_per_side;

        // Sort plates from largest to smallest
        let mut sorted_plates = self.plate_pairs.clone();
        sorted_plates.sort_by(|a, b| b.partial_cmp(a).unwrap());

        for &plate_weight in &sorted_plates {
            if remaining >= plate_weight {
                let count = (remaining / plate_weight).floor() as u32;
                if count > 0 {
                    plates_per_side.push((plate_weight, count));
                    remaining -= plate_weight * count as f64;
                }
            }
        }

        // Check if we got close enough (within 1% or 0.5kg)
        let achieved_load = bar_weight
            + plates_per_side
                .iter()
                .map(|(weight, count)| weight * (*count as f64) * 2.0)
                .sum::<f64>();

        let error = (achieved_load - (bar_weight + target_load)).abs();
        if error <= 0.5 || error / (bar_weight + target_load) <= 0.01 {
            let total_plates_needed = plates_per_side
                .iter()
                .map(|(_, count)| count * 2) // Both sides
                .sum();

            Some(PlateSolution {
                bar_weight,
                plates_per_side,
                total_plates_needed,
            })
        } else {
            None
        }
    }
}

impl RoundingPreview {
    /// Format the preview for display
    pub fn format_preview(&self) -> String {
        let delta_str = if self.delta.abs() < 0.01 {
            "exact".to_string()
        } else if self.delta > 0.0 {
            format!("+{:.1}kg", self.delta)
        } else {
            format!("{:.1}kg", self.delta)
        };

        let mut result = format!(
            "{:.1}kg → {:.1}kg ({})",
            self.original_load, self.rounded_load, delta_str
        );

        if let Some(ref solution) = self.plate_solution {
            result.push_str(&format!("\nBar: {:.1}kg", solution.bar_weight));
            if !solution.plates_per_side.is_empty() {
                result.push_str("\nPlates per side: ");
                let plate_desc: Vec<String> = solution
                    .plates_per_side
                    .iter()
                    .map(|(weight, count)| format!("{}×{:.1}kg", count, weight))
                    .collect();
                result.push_str(&plate_desc.join(", "));
            }
        }

        result
    }
}
