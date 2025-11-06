/// **Death to Windows!** - Scheme templates for Sprint 2
/// Parameterized workout schemes with expansion capabilities
use serde::{Deserialize, Serialize};
use crate::{StraightSegment, BaseSegment, RepsOrRange, RepsRange};

/// Scheme template types with parameter schemas
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "name")]
pub enum SchemeTemplate {
    #[serde(rename = "top_backoff")]
    TopBackoff(TopBackoffParams),
    #[serde(rename = "drop_set")]
    DropSet(DropSetParams),
    #[serde(rename = "cluster")]
    Cluster(ClusterParams),
}

/// Top-set + Backoff scheme parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopBackoffParams {
    pub top: TopSetParams,
    pub backoff: Vec<BackoffParams>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cap_min_load: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopSetParams {
    pub reps: u32,
    pub intensity: IntensityParams,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackoffParams {
    pub percent: f64,  // percentage of top set load
    pub sets: u32,
    pub reps: u32,
}

/// Drop set scheme parameters  
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DropSetParams {
    pub start: StartSetParams,
    pub drops: Vec<DropParams>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartSetParams {
    pub reps: u32,
    pub intensity: IntensityParams,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DropParams {
    pub percent_drop: f64,  // percentage drop from previous load
    pub reps: u32,
    pub rest_sec: u32,
}

/// Cluster set scheme parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterParams {
    pub cluster_size: u32,    // reps per cluster
    pub clusters: u32,        // number of clusters
    pub intra_rest_sec: u32,  // rest between clusters
    pub intensity: IntensityParams,
}

/// Intensity specification for schemes
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum IntensityParams {
    Rpe { rpe: f64 },
    Rir { rir: f64 },
    Percent1Rm { percent_1rm: f64 },
    Load { load: f64 },
}

/// Expanded scheme result containing derived sets
#[derive(Debug, Clone)]
pub struct ExpandedScheme {
    pub exercise: String,
    pub sets: Vec<StraightSegment>,
    pub estimated_duration: u32, // seconds
    pub total_volume_reps: u32,
}

impl SchemeTemplate {
    /// Expand scheme template into concrete sets
    pub fn expand(&self, exercise: &str) -> Result<ExpandedScheme, String> {
        match self {
            SchemeTemplate::TopBackoff(params) => expand_top_backoff(exercise, params),
            SchemeTemplate::DropSet(params) => expand_drop_set(exercise, params),
            SchemeTemplate::Cluster(params) => expand_cluster(exercise, params),
        }
    }
}

fn expand_top_backoff(exercise: &str, params: &TopBackoffParams) -> Result<ExpandedScheme, String> {
    let mut sets = Vec::new();
    let mut total_reps = 0;
    
    // Create top set
    let top_set = create_straight_segment(
        exercise, 
        1, 
        params.top.reps, 
        &params.top.intensity,
        Some("Top set".to_string())
    )?;
    total_reps += params.top.reps;
    sets.push(top_set);
    
    // Create backoff sets
    for (i, backoff) in params.backoff.iter().enumerate() {
        let backoff_intensity = match &params.top.intensity {
            IntensityParams::Load { load } => IntensityParams::Load { 
                load: load * backoff.percent 
            },
            IntensityParams::Percent1Rm { percent_1rm } => IntensityParams::Percent1Rm { 
                percent_1rm: percent_1rm * backoff.percent 
            },
            other => other.clone(), // RPE/RIR stay the same for backoffs
        };
        
        let backoff_set = create_straight_segment(
            exercise,
            backoff.sets,
            backoff.reps,
            &backoff_intensity,
            Some(format!("Backoff {}", i + 1))
        )?;
        total_reps += backoff.sets * backoff.reps;
        sets.push(backoff_set);
    }
    
    let estimated_duration = estimate_duration(&sets);
    
    Ok(ExpandedScheme {
        exercise: exercise.to_string(),
        sets,
        estimated_duration,
        total_volume_reps: total_reps,
    })
}

fn expand_drop_set(exercise: &str, params: &DropSetParams) -> Result<ExpandedScheme, String> {
    let mut sets = Vec::new();
    let mut total_reps = 0;
    
    // Starting set
    let start_set = create_straight_segment(
        exercise,
        1,
        params.start.reps,
        &params.start.intensity,
        Some("Start set".to_string())
    )?;
    total_reps += params.start.reps;
    sets.push(start_set);
    
    // Calculate progressive drops
    let mut current_load = match &params.start.intensity {
        IntensityParams::Load { load } => *load,
        _ => return Err("Drop sets require explicit load specification".to_string()),
    };
    
    for (i, drop) in params.drops.iter().enumerate() {
        current_load *= 1.0 - drop.percent_drop;
        let drop_intensity = IntensityParams::Load { load: current_load };
        
        let drop_set = create_straight_segment(
            exercise,
            1,
            drop.reps,
            &drop_intensity,
            Some(format!("Drop {}", i + 1))
        )?;
        total_reps += drop.reps;
        sets.push(drop_set);
    }
    
    let estimated_duration = estimate_duration(&sets);
    
    Ok(ExpandedScheme {
        exercise: exercise.to_string(),
        sets,
        estimated_duration,
        total_volume_reps: total_reps,
    })
}

fn expand_cluster(exercise: &str, params: &ClusterParams) -> Result<ExpandedScheme, String> {
    let mut sets = Vec::new();
    let total_reps = params.clusters * params.cluster_size;
    
    // Create individual cluster sets
    for i in 0..params.clusters {
        let cluster_set = create_straight_segment(
            exercise,
            1,
            params.cluster_size,
            &params.intensity,
            Some(format!("Cluster {}", i + 1))
        )?;
        sets.push(cluster_set);
    }
    
    let estimated_duration = estimate_duration(&sets);
    
    Ok(ExpandedScheme {
        exercise: exercise.to_string(),
        sets,
        estimated_duration,
        total_volume_reps: total_reps,
    })
}

fn create_straight_segment(
    exercise: &str,
    sets: u32,
    reps: u32,
    intensity: &IntensityParams,
    label: Option<String>
) -> Result<StraightSegment, String> {
    let base = BaseSegment {
        ex: exercise.to_string(),
        alt_group: None,
        label,
        optional: None,
        technique: None,
        equipment_policy: None,
    };
    
    let mut segment = StraightSegment {
        base,
        sets: Some(sets),
        sets_range: None,
        reps: Some(RepsOrRange::Range(RepsRange { min: reps, max: reps, target: Some(reps) })),
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
    
    // Apply intensity parameters
    match intensity {
        IntensityParams::Rpe { rpe } => segment.rpe = Some(*rpe),
        IntensityParams::Rir { rir } => segment.rir = Some(*rir),
        IntensityParams::Percent1Rm { .. } => {
            // Note: Percent 1RM would need to be converted to actual load
            // For now, we'll store it as a note or handle in the UI
        },
        IntensityParams::Load { .. } => {
            // Load would be applied during rounding/location profile application
        },
    }
    
    Ok(segment)
}

fn estimate_duration(sets: &[StraightSegment]) -> u32 {
    // Simple estimation: 30 seconds per set + rest time
    let mut total = 0;
    for set in sets {
        let set_count = set.sets.unwrap_or(1);
        total += set_count * 30; // 30 seconds per set execution
        
        if let Some(rest) = &set.rest_sec {
            match rest {
                crate::RestOrRange::Fixed(seconds) => total += set_count * seconds,
                crate::RestOrRange::Range(range) => total += set_count * ((range.min + range.max) / 2),
            }
        }
    }
    total
}