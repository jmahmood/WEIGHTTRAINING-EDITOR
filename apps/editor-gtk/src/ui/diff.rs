/// **Death to Windows!** - Diff dialog UI for Sprint 2
/// Side-by-side JSON comparison with metrics delta display
use gtk4::prelude::*;
use gtk4::{
    Box, Dialog, DialogFlags, Grid, Label, Orientation, Paned, ResponseType, ScrolledWindow,
};
use serde_json::json;
use weightlifting_core::{ChangeType, DiffMetrics, PlanChange, PlanDiff, PlanVersion};

/// Diff dialog for showing plan version differences
#[allow(dead_code)]
pub struct DiffDialog {
    pub dialog: Dialog,
    pub diff: PlanDiff,
}

#[allow(dead_code)]
impl DiffDialog {
    /// Create a new diff dialog
    pub fn new(diff: PlanDiff, parent: Option<&gtk4::Window>) -> Self {
        let dialog = Dialog::with_buttons(
            Some(&format!(
                "Plan Diff: {} → {}",
                diff.from_version, diff.to_version
            )),
            parent,
            DialogFlags::MODAL | DialogFlags::DESTROY_WITH_PARENT,
            &[("Close", ResponseType::Close)],
        );
        crate::ui::util::standardize_dialog(&dialog);

        dialog.set_default_size(1200, 800);

        let content = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(12)
            .margin_start(16)
            .margin_end(16)
            .margin_top(16)
            .margin_bottom(16)
            .build();

        // Metrics summary
        let metrics_section = Self::create_metrics_section(&diff.metrics);
        content.append(&metrics_section);

        // Side-by-side comparison
        let comparison_section = Self::create_comparison_section(&diff);
        content.append(&comparison_section);

        // Change list
        let changes_section = Self::create_changes_section(&diff.changes);
        content.append(&changes_section);

        dialog.content_area().append(&content);

        Self { dialog, diff }
    }

    /// Create metrics summary section
    fn create_metrics_section(metrics: &DiffMetrics) -> Box {
        let metrics_box = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(24)
            .css_classes(vec!["card".to_string()])
            .build();

        // Total changes
        let total_changes = Self::create_metric_card(
            "Total Changes",
            &metrics.total_changes.to_string(),
            "primary",
        );
        metrics_box.append(&total_changes);

        // Additions
        let additions =
            Self::create_metric_card("Additions", &metrics.additions.to_string(), "success");
        metrics_box.append(&additions);

        // Modifications
        let modifications = Self::create_metric_card(
            "Modifications",
            &metrics.modifications.to_string(),
            "warning",
        );
        metrics_box.append(&modifications);

        // Deletions
        let deletions =
            Self::create_metric_card("Deletions", &metrics.deletions.to_string(), "error");
        metrics_box.append(&deletions);

        // Segments added
        let segments_added = Self::create_metric_card(
            "Segments Added",
            &metrics.segments_added.to_string(),
            "info",
        );
        metrics_box.append(&segments_added);

        metrics_box
    }

    /// Create a single metric card
    fn create_metric_card(title: &str, value: &str, style: &str) -> Box {
        let card = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(4)
            .margin_start(8)
            .margin_end(8)
            .margin_top(8)
            .margin_bottom(8)
            .css_classes(vec![format!("metric-card-{}", style)])
            .build();

        let title_label = Label::builder()
            .label(title)
            .css_classes(vec!["caption".to_string()])
            .halign(gtk4::Align::Center)
            .build();

        let value_label = Label::builder()
            .label(value)
            .css_classes(vec!["title-1".to_string()])
            .halign(gtk4::Align::Center)
            .build();

        card.append(&title_label);
        card.append(&value_label);

        card
    }

    /// Create side-by-side comparison section
    fn create_comparison_section(diff: &PlanDiff) -> Box {
        let comparison_box = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(12)
            .build();

        let header = Label::builder()
            .label("Side-by-Side Comparison")
            .css_classes(vec!["heading".to_string()])
            .halign(gtk4::Align::Start)
            .build();
        comparison_box.append(&header);

        let paned = Paned::builder()
            .orientation(Orientation::Horizontal)
            .position(600)
            .build();

        // Left side: From version
        let from_section =
            Self::create_version_section("From Version", &diff.from_version.to_string(), "#ff6b6b");
        paned.set_start_child(Some(&from_section));

        // Right side: To version
        let to_section =
            Self::create_version_section("To Version", &diff.to_version.to_string(), "#51cf66");
        paned.set_end_child(Some(&to_section));

        comparison_box.append(&paned);
        comparison_box
    }

    /// Create version section for side-by-side comparison
    fn create_version_section(title: &str, version: &str, _color: &str) -> Box {
        let section = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(8)
            .margin_start(8)
            .margin_end(8)
            .build();

        let header = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .build();

        let title_label = Label::builder()
            .label(title)
            .css_classes(vec!["heading".to_string()])
            .halign(gtk4::Align::Start)
            .hexpand(true)
            .build();

        let version_label = Label::builder()
            .label(version)
            .css_classes(vec!["dim-label".to_string()])
            .halign(gtk4::Align::End)
            .build();

        header.append(&title_label);
        header.append(&version_label);
        section.append(&header);

        // Placeholder for JSON content
        let json_placeholder = Label::builder()
            .label(
                "JSON content would be displayed here\nwith syntax highlighting and diff markers",
            )
            .css_classes(vec!["monospace".to_string(), "dim-label".to_string()])
            .halign(gtk4::Align::Center)
            .valign(gtk4::Align::Center)
            .hexpand(true)
            .vexpand(true)
            .build();

        let scrolled = ScrolledWindow::builder()
            .child(&json_placeholder)
            .min_content_height(400)
            .build();

        section.append(&scrolled);
        section
    }

    /// Create changes list section
    fn create_changes_section(changes: &[PlanChange]) -> Box {
        let changes_box = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(8)
            .build();

        let header = Label::builder()
            .label("Detailed Changes")
            .css_classes(vec!["heading".to_string()])
            .halign(gtk4::Align::Start)
            .build();
        changes_box.append(&header);

        let grid = Grid::builder()
            .row_spacing(4)
            .column_spacing(12)
            .margin_start(8)
            .margin_end(8)
            .build();

        // Column headers
        let type_header = Label::builder()
            .label("Type")
            .css_classes(vec!["caption-heading".to_string()])
            .halign(gtk4::Align::Start)
            .build();
        grid.attach(&type_header, 0, 0, 1, 1);

        let path_header = Label::builder()
            .label("Path")
            .css_classes(vec!["caption-heading".to_string()])
            .halign(gtk4::Align::Start)
            .build();
        grid.attach(&path_header, 1, 0, 1, 1);

        let description_header = Label::builder()
            .label("Description")
            .css_classes(vec!["caption-heading".to_string()])
            .halign(gtk4::Align::Start)
            .build();
        grid.attach(&description_header, 2, 0, 1, 1);

        // Add each change as a row
        for (i, change) in changes.iter().enumerate() {
            let row = i + 1;

            // Change type with color coding
            let type_label = Label::builder()
                .label(match change.change_type {
                    ChangeType::Added => "➕ Added",
                    ChangeType::Removed => "➖ Removed",
                    ChangeType::Modified => "✏️ Modified",
                })
                .css_classes(vec![match change.change_type {
                    ChangeType::Added => "success".to_string(),
                    ChangeType::Removed => "error".to_string(),
                    ChangeType::Modified => "warning".to_string(),
                }])
                .halign(gtk4::Align::Start)
                .build();
            grid.attach(&type_label, 0, row as i32, 1, 1);

            // JSON path
            let path_label = Label::builder()
                .label(&change.path)
                .css_classes(vec!["monospace".to_string(), "dim-label".to_string()])
                .halign(gtk4::Align::Start)
                .tooltip_text(&change.path)
                .build();
            grid.attach(&path_label, 1, row as i32, 1, 1);

            // Description
            let desc_label = Label::builder()
                .label(&change.description)
                .halign(gtk4::Align::Start)
                .hexpand(true)
                .wrap(true)
                .build();
            grid.attach(&desc_label, 2, row as i32, 1, 1);
        }

        let scrolled = ScrolledWindow::builder()
            .child(&grid)
            .min_content_height(200)
            .build();

        changes_box.append(&scrolled);
        changes_box
    }

    /// Show the diff dialog
    pub fn show(&self) {
        self.dialog.present();
    }

    /// Create a sample diff for testing
    pub fn create_sample_diff() -> PlanDiff {
        PlanDiff {
            from_version: PlanVersion::new(1, 0, 0),
            to_version: PlanVersion::new(1, 1, 0),
            changes: vec![
                PlanChange {
                    change_type: ChangeType::Added,
                    path: "/schedule/0/segments/2".to_string(),
                    old_value: None,
                    new_value: Some(json!({ "type": "straight", "ex": "BENCH.PRESS", "reps": 5 })),
                    description: "Added bench press exercise".to_string(),
                },
                PlanChange {
                    change_type: ChangeType::Modified,
                    path: "/schedule/0/segments/0/reps".to_string(),
                    old_value: Some(json!(8)),
                    new_value: Some(json!(10)),
                    description: "Increased reps from 8 to 10".to_string(),
                },
                PlanChange {
                    change_type: ChangeType::Removed,
                    path: "/dictionary/OLD.EXERCISE".to_string(),
                    old_value: Some(json!("Old Exercise")),
                    new_value: None,
                    description: "Removed unused exercise".to_string(),
                },
            ],
            metrics: DiffMetrics {
                total_changes: 3,
                additions: 1,
                modifications: 1,
                deletions: 1,
                segments_added: 1,
                segments_removed: 0,
                segments_modified: 1,
                exercises_added: 0,
                exercises_removed: 1,
            },
        }
    }
}

impl Default for DiffDialog {
    fn default() -> Self {
        Self::new(Self::create_sample_diff(), None)
    }
}
