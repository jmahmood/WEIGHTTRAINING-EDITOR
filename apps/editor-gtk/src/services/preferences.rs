use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use weightlifting_core::AppPaths;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppPreferences {
    pub last_opened_directory: Option<String>,
    pub recent_mru: Vec<String>,
    // Sync memory of last used device and paths
    pub last_sync_transport: Option<String>, // "ssh" | "usb-fs"
    pub last_sync_local_root: Option<String>,
    pub last_sync_ssh_host: Option<String>, // user@host
    pub last_sync_ssh_port: Option<String>, // as string
    pub last_sync_ssh_remote_root: Option<String>,
    pub last_sync_usb_mount: Option<String>,
    pub last_sync_usb_remote_root: Option<String>,
}

impl Default for AppPreferences {
    fn default() -> Self {
        Self {
            last_opened_directory: None,
            recent_mru: Vec::new(),
            last_sync_transport: Some("ssh".to_string()),
            last_sync_local_root: None,
            last_sync_ssh_host: None,
            last_sync_ssh_port: None,
            last_sync_ssh_remote_root: None,
            last_sync_usb_mount: None,
            last_sync_usb_remote_root: None,
        }
    }
}

impl AppPreferences {
    pub fn load(paths: &AppPaths) -> Self {
        let prefs_path = Self::get_preferences_path(paths);

        match std::fs::read_to_string(&prefs_path) {
            Ok(content) => match serde_json::from_str::<AppPreferences>(&content) {
                Ok(prefs) => prefs,
                Err(e) => {
                    println!("Failed to parse preferences: {}, using defaults", e);
                    Self::default()
                }
            },
            Err(_) => {
                // File doesn't exist or can't be read, use defaults
                Self::default()
            }
        }
    }

    pub fn save(&self, paths: &AppPaths) -> Result<(), Box<dyn std::error::Error>> {
        let prefs_path = Self::get_preferences_path(paths);

        // Ensure the parent directory exists
        if let Some(parent) = prefs_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&prefs_path, content)?;

        Ok(())
    }

    pub fn set_last_opened_directory(&mut self, path: Option<PathBuf>) {
        self.last_opened_directory = path.map(|p| p.to_string_lossy().to_string());
    }

    pub fn get_last_opened_directory(&self) -> Option<PathBuf> {
        self.last_opened_directory
            .as_ref()
            .map(|s| PathBuf::from(s))
    }

    pub fn set_recent_mru(&mut self, uris: Vec<String>) {
        self.recent_mru = uris;
    }

    pub fn get_recent_mru(&self) -> Vec<String> {
        self.recent_mru.clone()
    }

    fn get_preferences_path(paths: &AppPaths) -> PathBuf {
        paths.state_dir.join("preferences.json")
    }
}
