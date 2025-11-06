pub mod segment;

use gtk4::ScrolledWindow;
use weightlifting_core::{Plan, AppPaths};
use crate::services::recent_files::RecentFilesService;
use crate::services::preferences::AppPreferences;

#[derive(Debug, Clone, PartialEq)]
pub enum FocusMode {
    Day,      // Focus is on a day (for day-level operations)
    Segment,  // Focus is on a segment within a day
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub current_plan: Option<Plan>,
    pub plan_id: Option<String>,
    pub current_file_path: Option<std::path::PathBuf>, // Track the current save location
    pub last_opened_directory: Option<std::path::PathBuf>, // Track the last directory used in file dialogs
    pub is_modified: bool,
    pub last_save: std::time::Instant,
    pub selected_segments: std::collections::HashSet<(usize, usize)>, // (day_index, segment_index)
    pub last_selected_segment: Option<(usize, usize)>, // Track last selected for range selection
    pub focused_segment: Option<(usize, usize)>, // Current focused segment for navigation (day_index, segment_index)
    pub focused_day: Option<usize>, // Current focused day for day-level navigation
    pub focus_mode: FocusMode, // Whether we're focusing on days or segments
    pub undo_history: Vec<Plan>, // Single-level undo history
    pub canvas_scrolled: Option<ScrolledWindow>, // Reference to canvas for updates
    pub target_day_for_next_segment: Option<usize>, // For day-specific segment creation
    pub recent_files_service: RecentFilesService, // Manage recent files for keyboard shortcuts
    pub preferences: AppPreferences, // Application preferences
}

impl AppState {
    pub(crate) fn new(paths: &AppPaths) -> Self {
        let preferences = AppPreferences::load(paths);
        let last_opened_directory = preferences.get_last_opened_directory();
        
        let s = Self {
            current_plan: None,
            plan_id: None,
            current_file_path: None,
            last_opened_directory,
            is_modified: false,
            last_save: std::time::Instant::now(),
            selected_segments: std::collections::HashSet::new(),
            last_selected_segment: None,
            focused_segment: None,
            focused_day: None,
            focus_mode: FocusMode::Segment,
            undo_history: Vec::new(),
            canvas_scrolled: None,
            target_day_for_next_segment: None,
            recent_files_service: RecentFilesService::new(),
            preferences,
        };
        // Initialize MRU from persisted preferences
        let initial_mru = s.preferences.get_recent_mru();
        s.recent_files_service.set_uris(initial_mru);
        s
    }
    
    pub(crate) fn mark_modified(&mut self) {
        self.is_modified = true;
    }
    
    pub(crate) fn mark_saved(&mut self) {
        self.is_modified = false;
        self.last_save = std::time::Instant::now();
    }
    
    #[allow(dead_code)]
    pub(crate) fn toggle_segment_selection(&mut self, day_index: usize, segment_index: usize) {
        let key = (day_index, segment_index);
        if self.selected_segments.contains(&key) {
            self.selected_segments.remove(&key);
        } else {
            self.selected_segments.insert(key);
        }
    }
    
    pub(crate) fn clear_selection(&mut self) {
        self.selected_segments.clear();
        self.last_selected_segment = None;
        // Keep focus when clearing selection
    }
    
    pub(crate) fn has_selection(&self) -> bool {
        !self.selected_segments.is_empty()
    }
    
    /// Select only this segment (exclusive selection)
    pub(crate) fn select_segment_exclusively(&mut self, day_index: usize, segment_index: usize) {
        let key = (day_index, segment_index);
        self.selected_segments.clear();
        self.selected_segments.insert(key);
        self.last_selected_segment = Some(key);
    }
    
    /// Toggle selection of a segment (for Ctrl+Click)
    pub(crate) fn toggle_segment_selection_with_ctrl(&mut self, day_index: usize, segment_index: usize) {
        let key = (day_index, segment_index);
        if self.selected_segments.contains(&key) {
            self.selected_segments.remove(&key);
            // If we're removing the last selected, clear it
            if self.last_selected_segment == Some(key) {
                self.last_selected_segment = None;
            }
        } else {
            self.selected_segments.insert(key);
            self.last_selected_segment = Some(key);
        }
    }
    
    /// Select range of segments (for Shift+Click)
    pub(crate) fn select_segment_range(&mut self, day_index: usize, segment_index: usize) {
        let key = (day_index, segment_index);
        
        if let Some((last_day, last_seg)) = self.last_selected_segment {
            // We need to select all segments between last_selected_segment and the current one
            // For now, implement a simple version that works within the same day
            if day_index == last_day {
                let start = last_seg.min(segment_index);
                let end = last_seg.max(segment_index);
                for seg_idx in start..=end {
                    self.selected_segments.insert((day_index, seg_idx));
                }
            } else {
                // Cross-day selection - for now just select the clicked segment
                // This could be enhanced later to select all segments between days
                self.selected_segments.insert(key);
            }
        } else {
            // No previous selection, just select this segment
            self.selected_segments.insert(key);
        }
        
        self.last_selected_segment = Some(key);
    }
    
    /// Save current state to undo history before making destructive changes
    pub(crate) fn save_to_undo_history(&mut self) {
        if let Some(plan) = &self.current_plan {
            // Only keep one level of undo history
            self.undo_history.clear();
            self.undo_history.push(plan.clone());
        }
    }
    
    // /// Check if undo is available
    // pub(crate) fn can_undo(&self) -> bool {
    //     !self.undo_history.is_empty()
    // }
    
    /// Perform undo operation
    pub(crate) fn undo(&mut self) -> bool {
        if let Some(previous_plan) = self.undo_history.pop() {
            self.current_plan = Some(previous_plan);
            self.mark_modified();
            self.clear_selection();
            true
        } else {
            false
        }
    }
    
    /// Set target day for next segment creation
    pub(crate) fn set_target_day_for_next_segment(&mut self, day_index: usize) {
        self.target_day_for_next_segment = Some(day_index);
    }
    
    /// Get and clear target day for segment creation
    pub(crate) fn get_and_clear_target_day(&mut self) -> Option<usize> {
        self.target_day_for_next_segment.take()
    }
    
    /// Set focused segment for navigation
    pub(crate) fn set_focused_segment(&mut self, day_index: usize, segment_index: usize) {
        self.focused_segment = Some((day_index, segment_index));
        self.focus_mode = FocusMode::Segment;
        // Clear day focus when focusing on segment
        self.focused_day = None;
    }
    
    /// Get the currently focused segment
    pub(crate) fn get_focused_segment(&self) -> Option<(usize, usize)> {
        self.focused_segment
    }
    
    /// Initialize focus to the first segment if none is set
    pub(crate) fn ensure_focus_initialized(&mut self) {
        if self.focused_segment.is_none() {
            if let Some(plan) = &self.current_plan {
                for (day_idx, day) in plan.schedule.iter().enumerate() {
                    if !day.segments.is_empty() {
                        self.focused_segment = Some((day_idx, 0));
                        break;
                    }
                }
            }
        }
    }
    
    /// Set focused day for day-level navigation
    pub(crate) fn set_focused_day(&mut self, day_index: usize) {
        self.focused_day = Some(day_index);
        self.focus_mode = FocusMode::Day;
        // Clear segment focus when focusing on day
        self.focused_segment = None;
    }
    
    /// Get the currently focused day
    #[allow(dead_code)]
    pub(crate) fn get_focused_day(&self) -> Option<usize> {
        self.focused_day
    }
    
    /// Clear day focus
    #[allow(dead_code)]
    pub(crate) fn clear_day_focus(&mut self) {
        self.focused_day = None;
        if self.focus_mode == FocusMode::Day {
            self.focus_mode = FocusMode::Segment;
        }
    }
    
    /// Initialize focus to the first item in the unified flow (first day header)
    pub(crate) fn ensure_focus_initialized_with_days(&mut self) {
        if self.focused_segment.is_none() && self.focused_day.is_none() {
            if let Some(plan) = &self.current_plan {
                if !plan.schedule.is_empty() {
                    // Always start at the first day header in the unified flow
                    self.focused_day = Some(0);
                    self.focus_mode = FocusMode::Day;
                }
            }
        }
    }
    
    /// Update last opened directory and save preferences
    pub(crate) fn update_last_opened_directory(&mut self, path: Option<std::path::PathBuf>, paths: &AppPaths) {
        self.last_opened_directory = path.clone();
        self.preferences.set_last_opened_directory(path);
        
        // Save preferences to disk
        if let Err(e) = self.preferences.save(paths) {
            println!("Failed to save preferences: {}", e);
        }
    }
}
