use gtk4::prelude::RecentManagerExt;
use gtk4::RecentManager;
use std::sync::{Arc, Mutex};

/// Service to manage recent files and provide keyboard shortcut access
#[derive(Clone, Debug)]
pub struct RecentFilesService {
    recent_uris: Arc<Mutex<Vec<String>>>,
}

impl Default for RecentFilesService {
    fn default() -> Self {
        Self::new()
    }
}

impl RecentFilesService {
    pub fn new() -> Self {
        Self {
            recent_uris: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Set the full MRU list (replaces current).
    pub fn set_uris(&self, uris: Vec<String>) {
        *self.recent_uris.lock().unwrap() = uris;
    }

    /// Mark a URI as opened, moving it to the front of the MRU list.
    pub fn mark_opened(&self, uri: &str, max_items: usize) {
        let mut uris = self.recent_uris.lock().unwrap();
        // Remove any existing occurrence
        uris.retain(|u| u != uri);
        // Insert at front
        uris.insert(0, uri.to_string());
        // Trim to max
        if uris.len() > max_items {
            uris.truncate(max_items);
        }
    }

    /// Update the recent files list from the system's recent manager
    pub fn update_recent_list(&self, max_items: usize, mime_whitelist: &[&str]) {
        let mut infos = RecentManager::default().items();
        // Sort by visited timestamp (descending) with defensive fallbacks
        infos.sort_by(|a, b| {
            let va = a.visited().to_unix();
            let vb = b.visited().to_unix();
            vb.cmp(&va) // descending
                .then_with(|| {
                    let ma = a.modified().to_unix();
                    let mb = b.modified().to_unix();
                    mb.cmp(&ma)
                })
                .then_with(|| {
                    let aa = a.added().to_unix();
                    let ab = b.added().to_unix();
                    ab.cmp(&aa)
                })
        });

        let mut recent_uris = Vec::new();
        let mut added = 0;

        for info in infos.into_iter() {
            if !info.exists() || !info.is_local() {
                continue;
            }
            if !mime_whitelist.is_empty() && !mime_whitelist.contains(&info.mime_type().as_str()) {
                continue;
            }

            recent_uris.push(info.uri().to_string());
            added += 1;
            if added >= max_items {
                break;
            }
        }

        *self.recent_uris.lock().unwrap() = recent_uris;
    }

    /// Merge system recent list into our MRU without losing current order.
    /// - Keeps existing MRU order for known URIs.
    /// - Appends any new URIs (not in MRU) after, ordered by visited desc.
    /// - Drops URIs that no longer exist or are filtered out.
    pub fn sync_from_recent_manager(&self, max_items: usize, mime_whitelist: &[&str]) {
        let mut infos = RecentManager::default().items();
        infos.sort_by(|a, b| {
            let va = a.visited().to_unix();
            let vb = b.visited().to_unix();
            vb.cmp(&va)
                .then_with(|| {
                    let ma = a.modified().to_unix();
                    let mb = b.modified().to_unix();
                    mb.cmp(&ma)
                })
                .then_with(|| {
                    let aa = a.added().to_unix();
                    let ab = b.added().to_unix();
                    ab.cmp(&aa)
                })
        });

        let mut filtered: Vec<String> = Vec::new();
        for info in infos.into_iter() {
            if !info.exists() || !info.is_local() {
                continue;
            }
            if !mime_whitelist.is_empty() && !mime_whitelist.contains(&info.mime_type().as_str()) {
                continue;
            }
            filtered.push(info.uri().to_string());
            if filtered.len() >= max_items {
                break;
            }
        }

        let mut mru = self.recent_uris.lock().unwrap();
        let current = mru.clone();

        let mut seen = std::collections::HashSet::new();
        let mut merged: Vec<String> = Vec::new();

        for uri in current.into_iter() {
            if filtered.contains(&uri) {
                seen.insert(uri.clone());
                merged.push(uri);
                if merged.len() >= max_items {
                    break;
                }
            }
        }

        if merged.len() < max_items {
            for uri in filtered.into_iter() {
                if !seen.contains(&uri) {
                    merged.push(uri);
                    if merged.len() >= max_items {
                        break;
                    }
                }
            }
        }

        *mru = merged;
    }

    /// Get URI for a specific position (1-based index)
    pub fn get_uri_at_position(&self, position: usize) -> Option<String> {
        let uris = self.recent_uris.lock().unwrap();
        if position > 0 && position <= uris.len() {
            Some(uris[position - 1].clone())
        } else {
            None
        }
    }

    /// Get all recent URIs for menu population
    pub fn get_all_uris(&self) -> Vec<String> {
        self.recent_uris.lock().unwrap().clone()
    }

    /// Get count of recent files
    pub fn count(&self) -> usize {
        self.recent_uris.lock().unwrap().len()
    }
}
