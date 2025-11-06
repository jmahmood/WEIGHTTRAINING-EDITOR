use crate::{ValidationError, ValidationErrorInfo, ValidationResult};
use weightlifting_core::Plan;
use jsonschema::JSONSchema;
use regex::Regex;
use std::sync::OnceLock;

static TEMPO_REGEX: OnceLock<Regex> = OnceLock::new();
static EX_CODE_REGEX: OnceLock<Regex> = OnceLock::new();

pub struct PlanValidator;

impl PlanValidator {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self)
    }

    pub fn validate(&self, plan: &Plan) -> ValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // First, validate against JSON schema
        let plan_json = match serde_json::to_value(plan) {
            Ok(json) => json,
            Err(_) => {
                errors.push(ValidationErrorInfo::new(
                    ValidationError::E190SchemaViolation,
                    "/",
                    None,
                    Some("Failed to serialize plan to JSON"),
                ));
                return ValidationResult { errors, warnings };
            }
        };

        // Compile schema and validate
        if let Ok(schema_value) = serde_json::from_str::<serde_json::Value>(weightlifting_core::PLAN_SCHEMA_V0_3) {
            if let Ok(schema) = JSONSchema::compile(&schema_value) {
                if let Err(validation_errors) = schema.validate(&plan_json) {
                    for error in validation_errors {
                        errors.push(ValidationErrorInfo::new(
                            ValidationError::E190SchemaViolation,
                            &error.instance_path.to_string(),
                            None,
                            Some(&error.to_string()),
                        ));
                    }
                }
            }
        }

        // Semantic validation  
        self.validate_semantic(plan, &mut errors, &mut warnings);

        ValidationResult { errors, warnings }
    }

    fn validate_semantic(
        &self,
        plan: &Plan,
        errors: &mut Vec<ValidationErrorInfo>,
        _warnings: &mut [ValidationErrorInfo],
    ) {
        // Validate exercise code format in dictionary (allow 2 or 3 uppercase segments)
        let code_rx = EX_CODE_REGEX.get_or_init(|| {
            // PATTERN.IMPLEMENT[.VARIANT], segments: A-Z, 0-9, _; 2 or 3 parts
            Regex::new(r"^[A-Z0-9_]+\.[A-Z0-9_]+(?:\.[A-Z0-9_]+)?$").expect("Invalid code regex")
        });
        for key in plan.dictionary.keys() {
            if !code_rx.is_match(key) {
                errors.push(ValidationErrorInfo::new(
                    ValidationError::E105InvalidExerciseCodeFormat(key.clone()),
                    "/dictionary",
                    Some(key),
                    Some("Expected PATTERN.IMPLEMENT[.VARIANT] in UPPERCASE"),
                ));
            }
        }

        // Validate groups reference existing dictionary items
        for (group_name, members) in &plan.groups {
            if members.is_empty() {
                errors.push(ValidationErrorInfo::new(
                    ValidationError::E120GroupEmptySegments,
                    "/groups",
                    Some(group_name),
                    Some("Group must contain at least one exercise"),
                ));
            }
            for mem in members {
                if !plan.dictionary.contains_key(mem) {
                    errors.push(ValidationErrorInfo::new(
                        ValidationError::E103GroupUnknownExercise(group_name.clone(), mem.clone()),
                        "/groups",
                        Some(group_name),
                        Some(mem),
                    ));
                }
            }
        }

        // Validate dictionary references
        for day in &plan.schedule {
            for (segment_idx, segment) in day.segments.iter().enumerate() {
                let path = format!("/schedule/{}/segments/{}", day.day - 1, segment_idx);
                self.validate_segment(segment, &path, plan, errors, &mut Vec::new());
            }
        }
    }

    fn validate_segment(
        &self,
        segment: &weightlifting_core::Segment,
        path: &str,
        plan: &Plan,
        errors: &mut Vec<ValidationErrorInfo>,
        _warnings: &mut [ValidationErrorInfo],
    ) {
        use weightlifting_core::Segment;
        
        match segment {
            Segment::Straight(s) => {
                self.validate_exercise(&s.base.ex, path, plan, errors);
                if let Some(ag) = &s.base.alt_group { self.validate_alt_group(ag, path, plan, errors); }
                self.validate_reps_time_conflict(&s.reps, &s.time_sec, path, errors);
                if let Some(tempo) = &s.tempo {
                    self.validate_tempo(tempo, path, errors);
                }
            }
            Segment::Rpe(s) => {
                self.validate_exercise(&s.base.ex, path, plan, errors);
                if let Some(ag) = &s.base.alt_group { self.validate_alt_group(ag, path, plan, errors); }
                self.validate_reps_time_conflict(&s.reps, &s.time_sec, path, errors);
            }
            Segment::Comment(_) => {
                // Comments don't need exercise validation
            }
            Segment::GroupChoose(g) => {
                self.validate_group_choose(g, path, plan, errors);
            }
            Segment::GroupRotate(g) => {
                self.validate_group_rotate(g, path, plan, errors);
            }
            Segment::GroupOptional(g) => {
                self.validate_group_optional(g, path, plan, errors);
            }
            Segment::GroupSuperset(g) => {
                self.validate_group_superset(g, path, plan, errors);
            }
            Segment::Scheme(s) => {
                self.validate_scheme_segment(s, path, plan, errors);
                if let Some(ag) = &s.base.alt_group { self.validate_alt_group(ag, path, plan, errors); }
            }
            Segment::Complex(c) => {
                self.validate_complex_segment(c, path, plan, errors);
            }
            Segment::Time(t) => {
                self.validate_exercise(&t.base.ex, path, plan, errors);
                if let Some(ag) = &t.base.alt_group { self.validate_alt_group(ag, path, plan, errors); }
                if let Some(interval) = &t.interval {
                    self.validate_interval(interval, path, errors);
                }
            }
            _ => {
                // TODO: Implement validation for other segment types
            }
        }
    }

    fn validate_exercise(
        &self,
        ex: &str,
        path: &str,
        plan: &Plan,
        errors: &mut Vec<ValidationErrorInfo>,
    ) {
        if ex.is_empty() {
            errors.push(ValidationErrorInfo::new(
                ValidationError::E101MissingExercise,
                path,
                Some("ex"),
                Some("Exercise code is required"),
            ));
            return;
        }

        if !plan.dictionary.contains_key(ex) {
            errors.push(ValidationErrorInfo::new(
                ValidationError::E102UnknownExercise(ex.to_string()),
                path,
                Some("ex"),
                Some(ex),
            ));
        }
    }

    fn validate_reps_time_conflict(
        &self,
        reps: &Option<weightlifting_core::RepsOrRange>,
        time_sec: &Option<weightlifting_core::TimeOrRange>,
        path: &str,
        errors: &mut Vec<ValidationErrorInfo>,
    ) {
        if reps.is_some() && time_sec.is_some() {
            errors.push(ValidationErrorInfo::new(
                ValidationError::E110RepsSecondsConflict,
                path,
                Some("reps"),
                Some("Choose exactly one of reps or time_sec"),
            ));
        }
    }

    fn validate_alt_group(
        &self,
        group: &str,
        path: &str,
        plan: &Plan,
        errors: &mut Vec<ValidationErrorInfo>,
    ) {
        if !plan.groups.contains_key(group) {
            errors.push(ValidationErrorInfo::new(
                ValidationError::E104UnknownAltGroup(group.to_string()),
                path,
                Some("alt_group"),
                Some("Create this group or choose an existing one"),
            ));
        }
    }

    fn validate_tempo(
        &self,
        tempo: &weightlifting_core::Tempo,
        path: &str,
        errors: &mut Vec<ValidationErrorInfo>,
    ) {
        // For structured tempo, validate each component is 0-10
        if tempo.ecc > 10 || tempo.bottom > 10 || tempo.con > 10 || tempo.top > 10 {
            errors.push(ValidationErrorInfo::new(
                ValidationError::E140TempoInvalidFormat(format!(
                    "{}-{}-{}-{}",
                    tempo.ecc, tempo.bottom, tempo.con, tempo.top
                )),
                path,
                Some("tempo"),
                Some("Tempo components must be 0-10 seconds"),
            ));
        }
    }

    /// Validate string tempo format (for backwards compatibility)
    pub fn validate_tempo_string(tempo: &str) -> bool {
        let regex = TEMPO_REGEX.get_or_init(|| {
            Regex::new(r"^([0-9]{1,2}|\*)-([0-9]{1,2}|\*)-([0-9]{1,2}|\*)(?:-([0-9]{1,2}|\*))?$")
                .expect("Invalid tempo regex")
        });
        
        if !regex.is_match(tempo) {
            return false;
        }

        // Validate each part is 0-10 or *
        let parts: Vec<&str> = tempo.split('-').collect();
        for part in parts {
            if part != "*" {
                if let Ok(num) = part.parse::<u32>() {
                    if num > 10 {
                        return false;
                    }
                } else {
                    return false;
                }
            }
        }

        true
    }

    fn validate_group_choose(
        &self,
        group: &weightlifting_core::GroupChooseSegment,
        path: &str,
        plan: &Plan,
        errors: &mut Vec<ValidationErrorInfo>,
    ) {
        if group.from.is_empty() {
            errors.push(ValidationErrorInfo::new(
                ValidationError::E120GroupEmptySegments,
                path,
                Some("segments"),
                Some("Group must contain at least one segment"),
            ));
        }

        for (idx, segment) in group.from.iter().enumerate() {
            let segment_path = format!("{}/from/{}", path, idx);
            self.validate_segment(segment, &segment_path, plan, errors, &mut Vec::new());
        }
    }

    fn validate_group_rotate(
        &self,
        group: &weightlifting_core::GroupRotateSegment,
        path: &str,
        plan: &Plan,
        errors: &mut Vec<ValidationErrorInfo>,
    ) {
        if group.items.is_empty() {
            errors.push(ValidationErrorInfo::new(
                ValidationError::E120GroupEmptySegments,
                path,
                Some("items"),
                Some("Group must contain at least one segment"),
            ));
        }

        for (idx, segment) in group.items.iter().enumerate() {
            let segment_path = format!("{}/items/{}", path, idx);
            self.validate_segment(segment, &segment_path, plan, errors, &mut Vec::new());
        }
    }

    fn validate_group_optional(
        &self,
        group: &weightlifting_core::GroupOptionalSegment,
        path: &str,
        plan: &Plan,
        errors: &mut Vec<ValidationErrorInfo>,
    ) {
        if group.items.is_empty() {
            errors.push(ValidationErrorInfo::new(
                ValidationError::E120GroupEmptySegments,
                path,
                Some("items"),
                Some("Group must contain at least one segment"),
            ));
        }

        for (idx, segment) in group.items.iter().enumerate() {
            let segment_path = format!("{}/items/{}", path, idx);
            self.validate_segment(segment, &segment_path, plan, errors, &mut Vec::new());
        }
    }

    fn validate_group_superset(
        &self,
        group: &weightlifting_core::GroupSupersetSegment,
        path: &str,
        plan: &Plan,
        errors: &mut Vec<ValidationErrorInfo>,
    ) {
        if group.items.is_empty() {
            errors.push(ValidationErrorInfo::new(
                ValidationError::E120GroupEmptySegments,
                path,
                Some("items"),
                Some("Group must contain at least one segment"),
            ));
        }

        for (idx, segment) in group.items.iter().enumerate() {
            let segment_path = format!("{}/items/{}", path, idx);
            self.validate_segment(segment, &segment_path, plan, errors, &mut Vec::new());
        }
    }

    fn validate_scheme_segment(
        &self,
        scheme: &weightlifting_core::SchemeSegment,
        path: &str,
        plan: &Plan,
        errors: &mut Vec<ValidationErrorInfo>,
    ) {
        // Validate the exercise exists
        self.validate_exercise(&scheme.base.ex, path, plan, errors);
        
        // Try to expand the scheme template to validate it
        if let Some(ref template) = scheme.template {
            match template.expand(&scheme.base.ex) {
                Ok(expanded) => {
                    // Validate the expanded scheme makes sense
                    if expanded.sets.is_empty() {
                        errors.push(ValidationErrorInfo::new(
                            ValidationError::E160SchemeEmptyExpansion,
                            path,
                            Some("template"),
                            Some("Scheme expansion resulted in no sets"),
                        ));
                    }
                    
                    if expanded.total_volume_reps == 0 {
                        errors.push(ValidationErrorInfo::new(
                            ValidationError::E161SchemeZeroVolume,
                            path,
                            Some("template"),
                            Some("Scheme expansion resulted in zero total reps"),
                        ));
                    }
                    
                    // Validate each expanded set
                    for (idx, set) in expanded.sets.iter().enumerate() {
                        let set_path = format!("{}/expanded/{}", path, idx);
                        self.validate_segment(&weightlifting_core::Segment::Straight(set.clone()), &set_path, plan, errors, &mut Vec::new());
                    }
                },
                Err(expansion_error) => {
                    errors.push(ValidationErrorInfo::new(
                        ValidationError::E162SchemeExpansionFailed(expansion_error),
                        path,
                        Some("template"),
                        Some("Failed to expand scheme template"),
                    ));
                }
            }
        }
    }

    fn validate_complex_segment(
        &self,
        complex: &weightlifting_core::ComplexSegment,
        path: &str,
        plan: &Plan,
        errors: &mut Vec<ValidationErrorInfo>,
    ) {
        // Validate anchor load configuration
        match complex.anchor_load.mode.as_str() {
            "pct_1rm" => {
                if complex.anchor_load.ex.is_none() {
                    errors.push(ValidationErrorInfo::new(
                        ValidationError::E170ComplexMissingExercise,
                        path,
                        Some("anchor_load.ex"),
                        Some("pct_1rm mode requires an exercise reference"),
                    ));
                }
                if complex.anchor_load.pct.is_none() {
                    errors.push(ValidationErrorInfo::new(
                        ValidationError::E171ComplexMissingPercentage,
                        path,
                        Some("anchor_load.pct"),
                        Some("pct_1rm mode requires a percentage value"),
                    ));
                }
                if let Some(ex) = &complex.anchor_load.ex {
                    self.validate_exercise(ex, path, plan, errors);
                }
            }
            "fixed_kg" => {
                if complex.anchor_load.kg.is_none() {
                    errors.push(ValidationErrorInfo::new(
                        ValidationError::E172ComplexMissingLoad,
                        path,
                        Some("anchor_load.kg"),
                        Some("fixed_kg mode requires a load value"),
                    ));
                }
            }
            _ => {
                errors.push(ValidationErrorInfo::new(
                    ValidationError::E173ComplexInvalidMode(complex.anchor_load.mode.clone()),
                    path,
                    Some("anchor_load.mode"),
                    Some("Mode must be 'pct_1rm' or 'fixed_kg'"),
                ));
            }
        }

        // Validate sequence items
        if complex.sequence.is_empty() {
            errors.push(ValidationErrorInfo::new(
                ValidationError::E174ComplexEmptySequence,
                path,
                Some("sequence"),
                Some("Complex must contain at least one sequence item"),
            ));
        }

        for (idx, item) in complex.sequence.iter().enumerate() {
            let item_path = format!("{}/sequence/{}", path, idx);
            self.validate_exercise(&item.ex, &item_path, plan, errors);
            if let Some(ag) = &item.alt_group { self.validate_alt_group(ag, &item_path, plan, errors); }
            
            // Validate reps range
            let weightlifting_core::RepsOrRange::Range(range) = &item.reps;
            if range.min == 0 || range.max == 0 {
                errors.push(ValidationErrorInfo::new(
                    ValidationError::E175ComplexZeroReps,
                    &item_path,
                    Some("reps"),
                    Some("Reps must be greater than 0"),
                ));
            }
        }

        // Validate sets count
        if complex.sets == 0 {
            errors.push(ValidationErrorInfo::new(
                ValidationError::E176ComplexZeroSets,
                path,
                Some("sets"),
                Some("Sets must be greater than 0"),
            ));
        }
    }

    fn validate_interval(
        &self,
        interval: &weightlifting_core::Interval,
        path: &str,
        errors: &mut Vec<ValidationErrorInfo>,
    ) {
        if interval.work_sec == 0 {
            errors.push(ValidationErrorInfo::new(
                ValidationError::E150IntervalZeroWork,
                path,
                Some("work_sec"),
                Some("Work interval must be greater than 0"),
            ));
        }

        if interval.repeats == 0 {
            errors.push(ValidationErrorInfo::new(
                ValidationError::E151IntervalZeroRepeats,
                path,
                Some("repeats"),
                Some("Interval repeats must be greater than 0"),
            ));
        }
        
        // Additional validations for reasonable time limits
        if interval.work_sec > 3600 {
            errors.push(ValidationErrorInfo::new(
                ValidationError::E130SecondsOutOfRange(interval.work_sec),
                path,
                Some("work_sec"),
                Some("Work period should not exceed 1 hour (3600 seconds)"),
            ));
        }
        
        if interval.rest_sec > 3600 {
            errors.push(ValidationErrorInfo::new(
                ValidationError::E130SecondsOutOfRange(interval.rest_sec),
                path,
                Some("rest_sec"),
                Some("Rest period should not exceed 1 hour (3600 seconds)"),
            ));
        }
        
        if interval.repeats > 1000 {
            errors.push(ValidationErrorInfo::new(
                ValidationError::E151IntervalZeroRepeats,
                path,
                Some("repeats"),
                Some("Repeats should not exceed 1000 for practical reasons"),
            ));
        }
    }

}
