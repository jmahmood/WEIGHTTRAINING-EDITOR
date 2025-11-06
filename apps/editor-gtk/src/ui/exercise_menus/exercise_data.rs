// Data structures for exercise dialog management

#[derive(Debug, Clone)]
pub struct SupersetExerciseData {
    pub ex_code: String,
    pub ex_name: String,
    pub sets: u32,
    pub reps_min: u32,
    pub reps_max: u32,
    pub rpe: Option<f64>,
    pub alt_group: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CircuitExerciseData {
    pub ex_code: String,
    pub ex_name: String,
    pub reps_min: u32,
    pub reps_max: u32,
    pub time_sec: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct SchemeSetData {
    pub label: Option<String>,
    pub sets: Option<u32>,
    pub reps_min: Option<u32>,
    pub reps_max: Option<u32>,
    pub time_sec: Option<u32>,
    pub rpe: Option<f64>,
    pub rest_sec: Option<u32>,
}
