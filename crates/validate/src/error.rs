use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum ValidationError {
    #[error("Invalid type: {0}")]
    E100InvalidType(String),
    #[error("Missing exercise")]
    E101MissingExercise,
    #[error("Unknown exercise: {0}")]
    E102UnknownExercise(String),
    #[error("Group '{0}' references unknown exercise code '{1}'")]
    E103GroupUnknownExercise(String, String),
    #[error("Unknown alt_group: {0}")]
    E104UnknownAltGroup(String),
    #[error("Invalid exercise code format: {0}")]
    E105InvalidExerciseCodeFormat(String),
    #[error("Reps and seconds conflict")]
    E110RepsSecondsConflict,
    #[error("Invalid reps range")]
    E111RepsRangeInvalid,
    #[error("Intensity conflict")]
    E120IntensityConflict,
    #[error("Group has empty segments")]
    E120GroupEmptySegments,
    #[error("Non-positive load: {0}")]
    E121LoadNonPositive(f64),
    #[error("RPE out of range: {0}")]
    E122RpeOutOfRange(f64),
    #[error("Percent 1RM out of range: {0}")]
    E123Percent1RmOutOfRange(f64),
    #[error("Seconds out of range: {0}")]
    E130SecondsOutOfRange(u32),
    #[error("Invalid interval")]
    E131IntervalInvalid,
    #[error("Invalid tempo format: {0}")]
    E140TempoInvalidFormat(String),
    #[error("Invalid sets: {0}")]
    E150SetsInvalid(u32),
    #[error("Interval work seconds must be greater than 0")]
    E150IntervalZeroWork,
    #[error("Interval repeats must be greater than 0")]
    E151IntervalZeroRepeats,
    #[error("Scheme expansion resulted in no sets")]
    E160SchemeEmptyExpansion,
    #[error("Scheme expansion resulted in zero total reps")]
    E161SchemeZeroVolume,
    #[error("Scheme expansion failed: {0}")]
    E162SchemeExpansionFailed(String),
    #[error("Complex segment missing exercise for pct_1rm mode")]
    E170ComplexMissingExercise,
    #[error("Complex segment missing percentage for pct_1rm mode")]
    E171ComplexMissingPercentage,
    #[error("Complex segment missing load for fixed_kg mode")]
    E172ComplexMissingLoad,
    #[error("Complex segment invalid mode: {0}")]
    E173ComplexInvalidMode(String),
    #[error("Complex segment has empty sequence")]
    E174ComplexEmptySequence,
    #[error("Complex segment sequence item has zero reps")]
    E175ComplexZeroReps,
    #[error("Complex segment has zero sets")]
    E176ComplexZeroSets,
    #[error("Schema violation")]
    E190SchemaViolation,
    #[error("Optional group has no items")]
    W210OptionalNoItems,
    #[error("Superset incompatible")]
    W211SupersetIncompatible,
}

impl ValidationError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::E100InvalidType(_) => "E100",
            Self::E101MissingExercise => "E101",
            Self::E102UnknownExercise(_) => "E102",
            Self::E103GroupUnknownExercise(_, _) => "E103",
            Self::E104UnknownAltGroup(_) => "E104",
            Self::E105InvalidExerciseCodeFormat(_) => "E105",
            Self::E110RepsSecondsConflict => "E110",
            Self::E111RepsRangeInvalid => "E111",
            Self::E120IntensityConflict => "E120",
            Self::E120GroupEmptySegments => "E120",
            Self::E121LoadNonPositive(_) => "E121",
            Self::E122RpeOutOfRange(_) => "E122",
            Self::E123Percent1RmOutOfRange(_) => "E123",
            Self::E130SecondsOutOfRange(_) => "E130",
            Self::E131IntervalInvalid => "E131",
            Self::E140TempoInvalidFormat(_) => "E140",
            Self::E150SetsInvalid(_) => "E150",
            Self::E150IntervalZeroWork => "E150",
            Self::E151IntervalZeroRepeats => "E151",
            Self::E160SchemeEmptyExpansion => "E160",
            Self::E161SchemeZeroVolume => "E161",
            Self::E162SchemeExpansionFailed(_) => "E162",
            Self::E170ComplexMissingExercise => "E170",
            Self::E171ComplexMissingPercentage => "E171",
            Self::E172ComplexMissingLoad => "E172",
            Self::E173ComplexInvalidMode(_) => "E173",
            Self::E174ComplexEmptySequence => "E174",
            Self::E175ComplexZeroReps => "E175",
            Self::E176ComplexZeroSets => "E176",
            Self::E190SchemaViolation => "E190",
            Self::W210OptionalNoItems => "W210",
            Self::W211SupersetIncompatible => "W211",
        }
    }

    pub fn is_warning(&self) -> bool {
        matches!(self, Self::W210OptionalNoItems | Self::W211SupersetIncompatible)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ValidationResult {
    pub errors: Vec<ValidationErrorInfo>,
    pub warnings: Vec<ValidationErrorInfo>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ValidationErrorInfo {
    pub code: String,
    pub message: String,
    pub path: String,
    pub field: Option<String>,
    pub hint: Option<String>,
}

impl ValidationErrorInfo {
    pub fn new(
        error: ValidationError,
        path: &str,
        field: Option<&str>,
        hint: Option<&str>,
    ) -> Self {
        Self {
            code: error.code().to_string(),
            message: error.to_string(),
            path: path.to_string(),
            field: field.map(|f| f.to_string()),
            hint: hint.map(|h| h.to_string()),
        }
    }
}
