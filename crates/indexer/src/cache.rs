use crate::metrics::{E1RMDataPoint, PRDataPoint, VolumeDataPoint};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};
use thiserror::Error;
use weightlifting_core::AppPaths;

#[derive(Error, Debug)]
pub enum CacheError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Cache corruption: {0}")]
    Corruption(String),
}

/// Status information about cached metrics
#[derive(Debug, Serialize, Deserialize)]
pub struct CacheStatus {
    pub e1rm_entries: usize,
    pub volume_entries: usize,
    pub pr_entries: usize,
    pub last_updated: Option<String>,
    pub cache_size_bytes: u64,
}

/// Metrics cache manager
pub struct MetricsCache {
    cache_dir: PathBuf,
}

impl MetricsCache {
    pub fn new(app_paths: &AppPaths) -> Self {
        let cache_dir = app_paths.metrics_cache_dir();

        // Ensure cache directory exists
        if let Err(e) = fs::create_dir_all(&cache_dir) {
            eprintln!("Warning: Failed to create cache directory: {}", e);
        }

        Self { cache_dir }
    }

    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    /// Store E1RM data in cache
    pub fn store_e1rm_data(&self, data: &[E1RMDataPoint]) -> Result<(), CacheError> {
        let path = self.cache_dir.join("e1rm_data.json");
        let file = File::create(path)?;
        let writer = BufWriter::new(file);

        let cached_data = CachedE1RMData {
            last_updated: Utc::now(),
            entries: data.to_vec(),
        };

        serde_json::to_writer_pretty(writer, &cached_data)?;
        Ok(())
    }

    /// Load E1RM data from cache
    pub fn load_e1rm_data(&self) -> Result<Vec<E1RMDataPoint>, CacheError> {
        let path = self.cache_dir.join("e1rm_data.json");

        if !path.exists() {
            return Ok(Vec::new());
        }

        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let cached_data: CachedE1RMData = serde_json::from_reader(reader)?;

        Ok(cached_data.entries)
    }

    /// Store volume data in cache
    pub fn store_volume_data(&self, data: &[VolumeDataPoint]) -> Result<(), CacheError> {
        let path = self.cache_dir.join("volume_data.json");
        let file = File::create(path)?;
        let writer = BufWriter::new(file);

        let cached_data = CachedVolumeData {
            last_updated: Utc::now(),
            entries: data.to_vec(),
        };

        serde_json::to_writer_pretty(writer, &cached_data)?;
        Ok(())
    }

    /// Load volume data from cache
    pub fn load_volume_data(&self) -> Result<Vec<VolumeDataPoint>, CacheError> {
        let path = self.cache_dir.join("volume_data.json");

        if !path.exists() {
            return Ok(Vec::new());
        }

        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let cached_data: CachedVolumeData = serde_json::from_reader(reader)?;

        Ok(cached_data.entries)
    }

    /// Store PR data in cache
    pub fn store_pr_data(&self, data: &[PRDataPoint]) -> Result<(), CacheError> {
        let path = self.cache_dir.join("pr_data.json");
        let file = File::create(path)?;
        let writer = BufWriter::new(file);

        let cached_data = CachedPRData {
            last_updated: Utc::now(),
            entries: data.to_vec(),
        };

        serde_json::to_writer_pretty(writer, &cached_data)?;
        Ok(())
    }

    /// Load PR data from cache
    pub fn load_pr_data(&self) -> Result<Vec<PRDataPoint>, CacheError> {
        let path = self.cache_dir.join("pr_data.json");

        if !path.exists() {
            return Ok(Vec::new());
        }

        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let cached_data: CachedPRData = serde_json::from_reader(reader)?;

        Ok(cached_data.entries)
    }

    /// Get cache status and statistics
    pub fn get_status(&self) -> Result<CacheStatus, CacheError> {
        let e1rm_data = self.load_e1rm_data()?;
        let volume_data = self.load_volume_data()?;
        let pr_data = self.load_pr_data()?;

        // Calculate cache size
        let cache_size = self.calculate_cache_size()?;

        // Get last updated timestamp
        let last_updated = self.get_last_updated_time();

        Ok(CacheStatus {
            e1rm_entries: e1rm_data.len(),
            volume_entries: volume_data.len(),
            pr_entries: pr_data.len(),
            last_updated,
            cache_size_bytes: cache_size,
        })
    }

    /// Clear all cached data
    pub fn clear_all(&self) -> Result<(), CacheError> {
        let cache_files = ["e1rm_data.json", "volume_data.json", "pr_data.json"];

        for file in &cache_files {
            let path = self.cache_dir.join(file);
            if path.exists() {
                fs::remove_file(path)?;
            }
        }

        Ok(())
    }

    /// Calculate total cache size in bytes
    fn calculate_cache_size(&self) -> Result<u64, CacheError> {
        let mut total_size = 0u64;

        if let Ok(entries) = fs::read_dir(&self.cache_dir) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() {
                        total_size += metadata.len();
                    }
                }
            }
        }

        Ok(total_size)
    }

    /// Get the most recent update timestamp from cache files
    fn get_last_updated_time(&self) -> Option<String> {
        let cache_files = ["e1rm_data.json", "volume_data.json", "pr_data.json"];
        let mut latest_time: Option<DateTime<Utc>> = None;

        for file in &cache_files {
            let path = self.cache_dir.join(file);
            if !path.exists() {
                continue;
            }

            // Try to read the last_updated field from each file
            if let Ok(file) = File::open(&path) {
                if let Ok(value) =
                    serde_json::from_reader::<_, serde_json::Value>(BufReader::new(file))
                {
                    if let Some(timestamp_str) = value.get("last_updated").and_then(|v| v.as_str())
                    {
                        if let Ok(timestamp) = DateTime::parse_from_rfc3339(timestamp_str) {
                            let utc_time = timestamp.with_timezone(&Utc);
                            if latest_time.is_none() || utc_time > latest_time.unwrap() {
                                latest_time = Some(utc_time);
                            }
                        }
                    }
                }
            }
        }

        latest_time.map(|t| t.to_rfc3339())
    }
}

/// Cached E1RM data with metadata
#[derive(Debug, Serialize, Deserialize)]
struct CachedE1RMData {
    last_updated: DateTime<Utc>,
    entries: Vec<E1RMDataPoint>,
}

/// Cached volume data with metadata
#[derive(Debug, Serialize, Deserialize)]
struct CachedVolumeData {
    last_updated: DateTime<Utc>,
    entries: Vec<VolumeDataPoint>,
}

/// Cached PR data with metadata
#[derive(Debug, Serialize, Deserialize)]
struct CachedPRData {
    last_updated: DateTime<Utc>,
    entries: Vec<PRDataPoint>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use tempfile::TempDir;
    use weightlifting_core::AppPaths;

    fn create_test_app_paths() -> (AppPaths, TempDir) {
        let temp_dir = TempDir::new().unwrap();

        // Create a mock AppPaths that uses our temp directory
        // This would need AppPaths to support custom directories
        // For now, we'll test with a direct cache path
        let paths = AppPaths::new().unwrap();
        (paths, temp_dir)
    }

    #[test]
    fn test_cache_roundtrip() {
        let temp_dir = TempDir::new().unwrap();

        // Create a cache with custom directory
        let cache = MetricsCache {
            cache_dir: temp_dir.path().to_path_buf(),
        };

        // Test E1RM data
        let e1rm_data = vec![E1RMDataPoint {
            exercise: "BP.BB.FLAT".to_string(),
            date: NaiveDate::from_ymd_opt(2025, 8, 22).unwrap(),
            e1rm_kg: 120.0,
            source_weight: 100.0,
            source_reps: 5,
            source_rpe: Some(8.0),
            formula: "epley_rpe".to_string(),
        }];

        // Store and load
        cache.store_e1rm_data(&e1rm_data).unwrap();
        let loaded = cache.load_e1rm_data().unwrap();

        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].exercise, "BP.BB.FLAT");
        assert_eq!(loaded[0].e1rm_kg, 120.0);
    }

    #[test]
    fn test_cache_status() {
        let temp_dir = TempDir::new().unwrap();

        let cache = MetricsCache {
            cache_dir: temp_dir.path().to_path_buf(),
        };

        // Initially empty
        let status = cache.get_status().unwrap();
        assert_eq!(status.e1rm_entries, 0);
        assert_eq!(status.volume_entries, 0);
        assert_eq!(status.pr_entries, 0);
    }
}
