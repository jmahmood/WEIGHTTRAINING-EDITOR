use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Top-level plan object matching v0.3 spec
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license_note: Option<String>,
    pub unit: Unit,
    pub dictionary: HashMap<String, String>,
    pub groups: HashMap<String, Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exercise_meta: Option<HashMap<String, ExerciseMeta>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phase: Option<Phase>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub week_overrides: Option<HashMap<String, Vec<WeekOverride>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub equipment_policy: Option<EquipmentPolicy>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progression: Option<Progression>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warmup: Option<WarmupConfig>,
    pub schedule: Vec<Day>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Unit {
    Kg,
    Lb,
    Bw,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExerciseMeta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub equipment: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub home_friendly: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Phase {
    pub index: u32,
    pub weeks: Vec<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeekOverride {
    pub target: OverrideTarget,
    pub modifier: OverrideModifier,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverrideTarget {
    pub day: u32,
    pub segment_idx: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverrideModifier {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rpe_cap: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquipmentPolicy {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub forbidden: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Progression {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reps_first: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub load_increment_kg: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cap_rpe: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarmupConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stages: Option<Vec<WarmupStage>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub round_to: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub merge_after_rounding: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarmupStage {
    pub pct_1rm: f64,
    pub reps: u32,
}

/// Day object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Day {
    pub day: u32,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_cap_min: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goal: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub equipment_policy: Option<EquipmentPolicy>,
    pub segments: Vec<Segment>,
}

/// Segment types (oneOf)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Segment {
    #[serde(rename = "straight")]
    Straight(StraightSegment),
    #[serde(rename = "rpe")]
    Rpe(RpeSegment),
    #[serde(rename = "percentage")]
    Percentage(PercentageSegment),
    #[serde(rename = "amrap")]
    Amrap(AmrapSegment),
    #[serde(rename = "superset")]
    Superset(SupersetSegment),
    #[serde(rename = "circuit")]
    Circuit(CircuitSegment),
    #[serde(rename = "scheme")]
    Scheme(SchemeSegment),
    #[serde(rename = "complex")]
    Complex(ComplexSegment),
    #[serde(rename = "comment")]
    Comment(CommentSegment),
    #[serde(rename = "choose")]
    GroupChoose(GroupChooseSegment),
    #[serde(rename = "group.rotate")]
    GroupRotate(GroupRotateSegment),
    #[serde(rename = "group.optional")]
    GroupOptional(GroupOptionalSegment),
    #[serde(rename = "group.superset")]
    GroupSuperset(GroupSupersetSegment),
    #[serde(rename = "time")]
    Time(TimeSegment),
}

/// Common fields for executable segments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseSegment {
    pub ex: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alt_group: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub optional: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub technique: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub equipment_policy: Option<EquipmentPolicy>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StraightSegment {
    #[serde(flatten)]
    pub base: BaseSegment,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sets: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sets_range: Option<Range>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reps: Option<RepsOrRange>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_sec: Option<TimeOrRange>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rest_sec: Option<RestOrRange>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rir: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rpe: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tempo: Option<Tempo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vbt: Option<Vbt>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub load_mode: Option<LoadMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intensifier: Option<Intensifier>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_stop: Option<AutoStop>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interval: Option<TimeInterval>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpeSegment {
    #[serde(flatten)]
    pub base: BaseSegment,
    pub sets: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reps: Option<RepsOrRange>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_sec: Option<TimeOrRange>,
    pub rpe: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rest_sec: Option<RestOrRange>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub anchor: Option<Anchor>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PercentageSegment {
    #[serde(flatten)]
    pub base: BaseSegment,
    pub prescriptions: Vec<PercentagePrescription>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PercentagePrescription {
    pub sets: u32,
    pub reps: u32,
    pub pct_1rm: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmrapSegment {
    #[serde(flatten)]
    pub base: BaseSegment,
    pub base_reps: u32,
    pub cap_reps: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupersetSegment {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pairing: Option<String>,
    pub rounds: u32,
    pub rest_sec: u32,
    pub rest_between_rounds_sec: u32,
    pub items: Vec<SupersetItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupersetItem {
    pub ex: String,
    pub sets: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reps: Option<RepsOrRange>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_sec: Option<TimeOrRange>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rpe: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alt_group: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intensifier: Option<Intensifier>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitSegment {
    pub rounds: u32,
    pub rest_sec: u32,
    pub rest_between_rounds_sec: u32,
    pub items: Vec<CircuitItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitItem {
    pub ex: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reps: Option<RepsOrRange>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_sec: Option<TimeOrRange>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alt_group: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemeSegment {
    #[serde(flatten)]
    pub base: BaseSegment,
    pub sets: Vec<SchemeSet>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub load_mode: Option<LoadMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template: Option<crate::SchemeTemplate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemeSet {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sets: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reps: Option<RepsOrRange>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_sec: Option<TimeOrRange>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rpe: Option<RpeOrRange>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rest_sec: Option<RestOrRange>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub anchor: Option<Anchor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub track_pr: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexSegment {
    pub anchor_load: AnchorLoad,
    pub sets: u32,
    pub rest_sec: u32,
    pub sequence: Vec<ComplexSequenceItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnchorLoad {
    pub mode: String, // "pct_1rm" | "fixed_kg"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ex: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pct: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kg: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexSequenceItem {
    pub ex: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alt_group: Option<String>,
    pub reps: RepsOrRange,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentSegment {
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
}


/// Common data types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Range {
    pub min: u32,
    pub max: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RepsOrRange {
    Range(RepsRange),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepsRange {
    pub min: u32,
    pub max: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TimeOrRange {
    Fixed(u32),
    Range(TimeRange),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    pub min: u32,
    pub max: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RestOrRange {
    Fixed(u32),
    Range(RestRange),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestRange {
    pub min: u32,
    pub max: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RpeOrRange {
    Fixed(f64),
    Range(RpeRange),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpeRange {
    pub min: f64,
    pub max: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tempo {
    pub ecc: u32,
    pub bottom: u32,
    pub con: u32,
    pub top: u32,
    pub units: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vbt {
    pub target_mps: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub loss_cap_pct: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LoadMode {
    Added,
    Assisted,
    BodyweightOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Intensifier {
    pub kind: IntensifierKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub when: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub drop_pct: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub steps: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clusters: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reps_per_cluster: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intra_rest_sec: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IntensifierKind {
    Dropset,
    RestPause,
    Myo,
    Cluster,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoStop {
    pub reason: String,
    pub threshold: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Anchor {
    pub of_set_index: u32,
    pub multiplier: f64,
}

/// Location profile for equipment/rounding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationProfile {
    pub id: String,
    pub name: String,
    pub bars: Vec<f64>,
    pub plates: Vec<f64>,
    pub dumbbells: Vec<f64>,
    pub increments: f64,
    pub rounding_rules: RoundingRules,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoundingRules {
    pub method: String, // "nearest", "up", "down"
    pub precision: f64,
}

/// Group container state for persistence and UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupState {
    /// For choose groups - tracks last selected item
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_choice_item_id: Option<String>,
    
    /// For choose groups - selection history
    #[serde(skip_serializing_if = "Option::is_none")]
    pub choice_history: Option<Vec<String>>,
    
    /// For rotate groups - current rotation index
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotation_index: Option<u32>,
    
    /// For optional groups - enabled by default flag
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled_by_default: Option<bool>,
    
    /// For optional groups - last used timestamp for analytics
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_used: Option<String>, // ISO 8601 timestamp
}

/// Extended group segments with state persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupChooseSegment {
    pub pick: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotation: Option<String>,
    pub from: Vec<Segment>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<GroupState>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupRotateSegment {
    pub items: Vec<Segment>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<GroupState>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupOptionalSegment {
    pub items: Vec<Segment>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<GroupState>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupSupersetSegment {
    pub items: Vec<Segment>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rest_between: Option<u32>, // seconds between items in superset
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<GroupState>,
}

/// Interval configuration for time-based sets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeInterval {
    pub work: u32,    // work duration in seconds
    pub rest: u32,    // rest duration in seconds  
    pub repeats: u32, // number of work/rest cycles
}

impl Plan {
    pub fn new(name: String) -> Self {
        Self {
            name,
            author: Some("Program Author".to_string()),
            source_url: Some("https://example.com/plan".to_string()),
            license_note: Some("For personal use; see source.".to_string()),
            unit: Unit::Kg,
            dictionary: HashMap::new(),
            groups: HashMap::new(),
            exercise_meta: None,
            phase: None,
            week_overrides: None,
            equipment_policy: None,
            progression: None,
            warmup: None,
            schedule: vec![],
        }
    }
}

impl Default for GroupState {
    fn default() -> Self {
        Self::new()
    }
}

impl GroupState {
    pub fn new() -> Self {
        Self {
            last_choice_item_id: None,
            choice_history: None,
            rotation_index: None,
            enabled_by_default: None,
            last_used: None,
        }
    }
    
    pub fn for_choose() -> Self {
        Self {
            last_choice_item_id: None,
            choice_history: Some(vec![]),
            rotation_index: None,
            enabled_by_default: None,
            last_used: None,
        }
    }
    
    pub fn for_rotate() -> Self {
        Self {
            last_choice_item_id: None,
            choice_history: None,
            rotation_index: Some(0),
            enabled_by_default: None,
            last_used: None,
        }
    }
    
    pub fn for_optional() -> Self {
        Self {
            last_choice_item_id: None,
            choice_history: None,
            rotation_index: None,
            enabled_by_default: Some(true),
            last_used: None,
        }
    }
}

impl GroupChooseSegment {
    pub fn new(from: Vec<Segment>) -> Self {
        Self {
            pick: 1,
            rotation: None,
            from,
            state: Some(GroupState::for_choose()),
        }
    }
}

impl GroupRotateSegment {
    pub fn new(items: Vec<Segment>) -> Self {
        Self {
            items,
            state: Some(GroupState::for_rotate()),
        }
    }
}

impl GroupOptionalSegment {
    pub fn new(items: Vec<Segment>) -> Self {
        Self {
            items,
            state: Some(GroupState::for_optional()),
        }
    }
}

impl GroupSupersetSegment {
    pub fn new(items: Vec<Segment>) -> Self {
        Self {
            items,
            rest_between: None,
            state: None,
        }
    }
}

/// Time-based segment with intervals (work/rest/repeats)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSegment {
    #[serde(flatten)]
    pub base: BaseSegment,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interval: Option<Interval>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rpe: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rir: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub load_mode: Option<LoadMode>,
}

/// Interval specification for time-based training
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Interval {
    pub work_sec: u32,        // Work period in seconds
    pub rest_sec: u32,        // Rest period in seconds  
    pub repeats: u32,         // Number of work/rest cycles
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warmup_sec: Option<u32>, // Optional warmup period
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cooldown_sec: Option<u32>, // Optional cooldown period
}

impl TimeSegment {
    pub fn new(ex: String, interval: Interval) -> Self {
        Self {
            base: BaseSegment {
                ex,
                alt_group: None,
                label: None,
                optional: None,
                technique: None,
                equipment_policy: None,
            },
            interval: Some(interval),
            rpe: None,
            rir: None,
            load_mode: None,
        }
    }
    
    /// Calculate total duration including warmup, work periods, rest periods, and cooldown
    pub fn total_duration_sec(&self) -> u32 {
        if let Some(ref interval) = self.interval {
            let warmup = interval.warmup_sec.unwrap_or(0);
            let work_rest_cycles = interval.repeats * (interval.work_sec + interval.rest_sec);
            let cooldown = interval.cooldown_sec.unwrap_or(0);
            warmup + work_rest_cycles + cooldown
        } else {
            0
        }
    }
    
    /// Calculate total work time (excluding rest periods)
    pub fn total_work_sec(&self) -> u32 {
        if let Some(ref interval) = self.interval {
            interval.repeats * interval.work_sec
        } else {
            0
        }
    }
}