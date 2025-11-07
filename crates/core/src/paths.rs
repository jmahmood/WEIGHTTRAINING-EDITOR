use dirs::{cache_dir, data_dir, state_dir};
use std::fs;
use std::path::PathBuf;

pub struct AppPaths {
    pub data_dir: PathBuf,
    pub state_dir: PathBuf,
    pub cache_dir: PathBuf,
}

impl AppPaths {
    pub fn new() -> Result<Self, std::io::Error> {
        let app_name = "weightlifting-desktop";

        // XDG_DATA_HOME or ~/.local/share
        let data_dir = data_dir()
            .ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Could not determine data directory",
                )
            })?
            .join(app_name);

        // XDG_STATE_HOME or ~/.local/state
        let state_dir = state_dir()
            .ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Could not determine state directory",
                )
            })?
            .join(app_name);

        // XDG_CACHE_HOME or ~/.cache
        let cache_dir = cache_dir()
            .ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Could not determine cache directory",
                )
            })?
            .join(app_name);

        // Create directories if they don't exist
        fs::create_dir_all(&data_dir)?;
        fs::create_dir_all(&state_dir)?;
        fs::create_dir_all(&cache_dir)?;

        Ok(Self {
            data_dir,
            state_dir,
            cache_dir,
        })
    }

    /// Get path for active plan storage: ~/.local/share/weightlifting-desktop/plans/<plan_id>/<version>.json
    pub fn active_plan_path(&self, plan_id: &str, version: &str) -> PathBuf {
        self.data_dir
            .join("plans")
            .join(plan_id)
            .join(format!("{}.json", version))
    }

    /// Get directory for plan versions: ~/.local/share/weightlifting-desktop/plans/<plan_id>/
    pub fn plan_dir(&self, plan_id: &str) -> PathBuf {
        self.data_dir.join("plans").join(plan_id)
    }

    /// Get path for draft storage: ~/.local/state/weightlifting-desktop/drafts/<plan_id>.json
    pub fn draft_path(&self, plan_id: &str) -> PathBuf {
        self.state_dir
            .join("drafts")
            .join(format!("{}.json", plan_id))
    }

    /// Get drafts directory: ~/.local/state/weightlifting-desktop/drafts/
    pub fn drafts_dir(&self) -> PathBuf {
        self.state_dir.join("drafts")
    }

    /// Get cache directory for metrics: ~/.cache/weightlifting-desktop/metrics/
    pub fn metrics_cache_dir(&self) -> PathBuf {
        self.cache_dir.join("metrics")
    }

    /// Ensure required subdirectories exist
    pub fn ensure_subdirs(&self) -> Result<(), std::io::Error> {
        fs::create_dir_all(self.data_dir.join("plans"))?;
        fs::create_dir_all(self.drafts_dir())?;
        fs::create_dir_all(self.metrics_cache_dir())?;
        Ok(())
    }

    /// Path for append-only media attachments JSONL (canonical)
    /// Example: ~/.local/share/weightlifting-desktop/media_attachments.jsonl
    pub fn media_attachments_path(&self) -> PathBuf {
        self.data_dir.join("media_attachments.jsonl")
    }
}

// Standalone functions for FFI compatibility
const APP_NAME: &str = "weightlifting-desktop";

/// Get the application support directory
/// - macOS: ~/Library/Application Support/weightlifting-desktop
/// - Linux: ~/.local/share/weightlifting-desktop
pub fn get_app_support_dir() -> Result<PathBuf, std::io::Error> {
    let base = data_dir().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Could not determine application support directory",
        )
    })?;
    let app_dir = base.join(APP_NAME);
    fs::create_dir_all(&app_dir)?;
    Ok(app_dir)
}

/// Get the cache directory
/// - macOS: ~/Library/Caches/weightlifting-desktop
/// - Linux: ~/.cache/weightlifting-desktop
pub fn get_cache_dir() -> Result<PathBuf, std::io::Error> {
    let base = cache_dir().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Could not determine cache directory",
        )
    })?;
    let app_dir = base.join(APP_NAME);
    fs::create_dir_all(&app_dir)?;
    Ok(app_dir)
}

/// Get the drafts directory (for autosaves and temporary files)
/// - macOS: ~/Library/Application Support/weightlifting-desktop/drafts
/// - Linux: ~/.local/state/weightlifting-desktop/drafts
pub fn get_drafts_dir() -> Result<PathBuf, std::io::Error> {
    let base = if cfg!(target_os = "macos") {
        // On macOS, use Application Support for drafts (no separate state dir)
        data_dir()
    } else {
        // On Linux, use XDG_STATE_HOME or ~/.local/state
        state_dir()
    };

    let drafts = base
        .ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Could not determine drafts directory",
            )
        })?
        .join(APP_NAME)
        .join("drafts");

    fs::create_dir_all(&drafts)?;
    Ok(drafts)
}
