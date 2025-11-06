use csv::Reader;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::path::Path;
use chrono::{NaiveDate, NaiveTime};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CsvParseError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("CSV parsing error: {0}")]
    Csv(#[from] csv::Error),
    #[error("Date parsing error: {0}")]
    DateParse(String),
    #[error("Invalid data: {0}")]
    InvalidData(String),
}

/// Session CSV record matching v0.3 spec
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRecord {
    pub session_id: String,
    pub date: NaiveDate,
    pub time: NaiveTime,
    pub plan_name: Option<String>,
    pub day_label: Option<String>,
    pub segment_id: u32,
    pub superset_id: Option<String>,
    pub ex_code: String,
    pub adlib: u8, // 0 = plan-prescribed, 1 = user swap/ad-lib
    pub set_num: u32,
    pub reps: Option<u32>,
    pub time_sec: Option<u32>,
    pub weight: Option<f64>,
    pub unit: String, // kg, lb, bw
    pub is_warmup: u8, // 0 or 1
    pub rpe: Option<f64>,
    pub rir: Option<u32>,
    pub tempo: Option<String>,
    pub rest_sec: Option<u32>,
    pub effort_1to5: u32, // 1-5 scale
    pub tags: Option<String>, // semicolon-separated
    pub notes: Option<String>,
    pub pr_types: Option<String>, // semicolon-separated
}

impl SessionRecord {
    /// Check if this is a main working set (not warmup, has weight/reps)
    pub fn is_working_set(&self) -> bool {
        self.is_warmup == 0 && 
        (self.reps.is_some() || self.time_sec.is_some()) &&
        (self.weight.is_some() || self.unit == "bw")
    }
    
    /// Get estimated 1RM for this set if possible
    pub fn can_calculate_e1rm(&self) -> bool {
        self.is_working_set() && 
        self.reps.is_some() && 
        self.reps.unwrap() > 0 &&
        (self.weight.is_some() || self.unit == "bw")
    }
    
    /// Extract tags as vector
    pub fn get_tags(&self) -> Vec<String> {
        self.tags.as_ref()
            .map(|tags| tags.split(';').map(|s| s.trim().to_string()).collect())
            .unwrap_or_default()
    }
    
    /// Extract PR types as vector
    pub fn get_pr_types(&self) -> Vec<String> {
        self.pr_types.as_ref()
            .map(|prs| prs.split(';').map(|s| s.trim().to_string()).collect())
            .unwrap_or_default()
    }
}

/// CSV parser for session data
pub struct SessionCsvParser {
    // Configuration could go here
}

impl Default for SessionCsvParser {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionCsvParser {
    pub fn new() -> Self {
        Self {}
    }
    
    /// Parse a CSV file containing session data
    pub fn parse_csv_file<P: AsRef<Path>>(
        &self, 
        path: P
    ) -> Result<Vec<SessionRecord>, CsvParseError> {
        let file = File::open(&path)?;
        let mut reader = Reader::from_reader(file);
        
        let mut records = Vec::new();
        
        for result in reader.deserialize() {
            let raw_record: RawSessionRecord = result?;
            let record = self.parse_record(raw_record)?;
            records.push(record);
        }
        
        Ok(records)
    }
    
    fn parse_record(&self, raw: RawSessionRecord) -> Result<SessionRecord, CsvParseError> {
        let date = NaiveDate::parse_from_str(&raw.date, "%Y-%m-%d")
            .map_err(|e| CsvParseError::DateParse(format!("date: {}", e)))?;
            
        let time = NaiveTime::parse_from_str(&raw.time, "%H:%M:%S")
            .map_err(|e| CsvParseError::DateParse(format!("time: {}", e)))?;
        
        // Validate unit
        if !["kg", "lb", "bw"].contains(&raw.unit.as_str()) {
            return Err(CsvParseError::InvalidData(
                format!("Invalid unit: {}", raw.unit)
            ));
        }
        
        // Validate effort scale
        if raw.effort_1to5 < 1 || raw.effort_1to5 > 5 {
            return Err(CsvParseError::InvalidData(
                format!("Invalid effort_1to5: {}", raw.effort_1to5)
            ));
        }

        // Validate RPE bounds if provided
        if let Some(rpe) = raw.rpe {
            if !(6.0..=10.0).contains(&rpe) {
                return Err(CsvParseError::InvalidData(
                    format!("Invalid rpe: {} (expected 6.0-10.0)", rpe)
                ));
            }
        }

        // Validate RIR bounds if provided
        if let Some(rir) = raw.rir {
            if rir > 5 {
                return Err(CsvParseError::InvalidData(
                    format!("Invalid rir: {} (expected 0-5)", rir)
                ));
            }
        }
        
        // Check XOR constraint: either reps or time_sec, not both
        match (raw.reps.as_ref(), raw.time_sec.as_ref()) {
            (Some(_), Some(_)) => {
                return Err(CsvParseError::InvalidData(
                    "Both reps and time_sec specified - only one allowed".to_string()
                ));
            },
            (None, None) if raw.is_warmup == 0 => {
                return Err(CsvParseError::InvalidData(
                    "Neither reps nor time_sec specified for working set".to_string()
                ));
            },
            _ => {} // Valid
        }
        
        // If bodyweight unit, weight must be empty
        if raw.unit == "bw" && raw.weight.is_some() {
            return Err(CsvParseError::InvalidData(
                "Weight must be empty when unit is 'bw'".to_string()
            ));
        }

        Ok(SessionRecord {
            session_id: raw.session_id,
            date,
            time,
            plan_name: if raw.plan_name.is_empty() { None } else { Some(raw.plan_name) },
            day_label: if raw.day_label.is_empty() { None } else { Some(raw.day_label) },
            segment_id: raw.segment_id,
            superset_id: if raw.superset_id.is_empty() { None } else { Some(raw.superset_id) },
            ex_code: raw.ex_code,
            adlib: raw.adlib,
            set_num: raw.set_num,
            reps: raw.reps.and_then(|r| if r == 0 { None } else { Some(r) }),
            time_sec: raw.time_sec,
            weight: raw.weight,
            unit: raw.unit,
            is_warmup: raw.is_warmup,
            rpe: raw.rpe,
            rir: raw.rir,
            tempo: if raw.tempo.is_empty() { None } else { Some(raw.tempo) },
            rest_sec: raw.rest_sec,
            effort_1to5: raw.effort_1to5,
            tags: if raw.tags.is_empty() { None } else { Some(raw.tags) },
            notes: if raw.notes.is_empty() { None } else { Some(raw.notes) },
            pr_types: if raw.pr_types.is_empty() { None } else { Some(raw.pr_types) },
        })
    }
}

/// Raw CSV record for deserialization (handles empty strings)
#[derive(Debug, Deserialize)]
struct RawSessionRecord {
    session_id: String,
    date: String,
    time: String,
    plan_name: String,
    day_label: String,
    segment_id: u32,
    superset_id: String,
    ex_code: String,
    adlib: u8,
    set_num: u32,
    reps: Option<u32>,
    time_sec: Option<u32>,
    weight: Option<f64>,
    unit: String,
    is_warmup: u8,
    rpe: Option<f64>,
    rir: Option<u32>,
    tempo: String,
    rest_sec: Option<u32>,
    effort_1to5: u32,
    tags: String,
    notes: String,
    pr_types: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_valid_csv() {
        let csv_content = r#"session_id,date,time,plan_name,day_label,segment_id,superset_id,ex_code,adlib,set_num,reps,time_sec,weight,unit,is_warmup,rpe,rir,tempo,rest_sec,effort_1to5,tags,notes,pr_types
2025-08-14T09-35-00Z-001,2025-08-14,09:37:12,PHUL,Upper Power,1,,BP.BB.FLAT,0,1,5,,85,kg,0,7.5,2,2-1-1,150,3,,,
2025-08-14T09-35-00Z-001,2025-08-14,10:05:30,PHUL,Upper Power,4,,CORE.BW.PLNK,0,1,,45,,bw,0,8.0,,,60,3,,timed,"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "{}", csv_content).unwrap();
        
        let parser = SessionCsvParser::new();
        let records = parser.parse_csv_file(temp_file.path()).unwrap();
        
        assert_eq!(records.len(), 2);
        
        let first = &records[0];
        assert_eq!(first.ex_code, "BP.BB.FLAT");
        assert_eq!(first.reps, Some(5));
        assert_eq!(first.weight, Some(85.0));
        assert_eq!(first.unit, "kg");
        assert!(first.can_calculate_e1rm());
        
        let second = &records[1];
        assert_eq!(second.ex_code, "CORE.BW.PLNK");
        assert_eq!(second.time_sec, Some(45));
        assert_eq!(second.unit, "bw");
        assert!(!second.can_calculate_e1rm()); // Time-based set
    }

    #[test]
    fn test_invalid_rpe_range_is_rejected() {
        let csv_content = r#"session_id,date,time,plan_name,day_label,segment_id,superset_id,ex_code,adlib,set_num,reps,time_sec,weight,unit,is_warmup,rpe,rir,tempo,rest_sec,effort_1to5,tags,notes,pr_types
2025-08-14T09-35-00Z-001,2025-08-14,09:37:12,PHUL,Upper Power,1,,BP.BB.FLAT,0,1,5,,85,kg,0,5.5,2,2-1-1,150,3,,,
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "{}", csv_content).unwrap();

        let parser = SessionCsvParser::new();
        let result = parser.parse_csv_file(temp_file.path());
        assert!(matches!(result, Err(CsvParseError::InvalidData(_))));
    }

    #[test]
    fn test_invalid_rir_range_is_rejected() {
        let csv_content = r#"session_id,date,time,plan_name,day_label,segment_id,superset_id,ex_code,adlib,set_num,reps,time_sec,weight,unit,is_warmup,rpe,rir,tempo,rest_sec,effort_1to5,tags,notes,pr_types
2025-08-14T09-35-00Z-001,2025-08-14,09:37:12,PHUL,Upper Power,1,,BP.BB.FLAT,0,1,5,,85,kg,0,7.5,6,2-1-1,150,3,,,
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "{}", csv_content).unwrap();

        let parser = SessionCsvParser::new();
        let result = parser.parse_csv_file(temp_file.path());
        assert!(matches!(result, Err(CsvParseError::InvalidData(_))));
    }

    #[test]
    fn test_bw_unit_with_weight_is_rejected() {
        let csv_content = r#"session_id,date,time,plan_name,day_label,segment_id,superset_id,ex_code,adlib,set_num,reps,time_sec,weight,unit,is_warmup,rpe,rir,tempo,rest_sec,effort_1to5,tags,notes,pr_types
2025-08-14T09-35-00Z-001,2025-08-14,09:37:12,PHUL,Upper Power,1,,CORE.BW.PLNK,0,1,,45,10,bw,0,8.0,,,60,3,,timed,
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "{}", csv_content).unwrap();

        let parser = SessionCsvParser::new();
        let result = parser.parse_csv_file(temp_file.path());
        assert!(matches!(result, Err(CsvParseError::InvalidData(_))));
    }
}
