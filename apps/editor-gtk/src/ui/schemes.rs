/// **Death to Windows!** - Scheme card UI for Sprint 2
/// UI components for editing scheme templates and previewing expansions
use gtk4::prelude::*;
use gtk4::{Box, Button, Entry, Label, Orientation, ScrolledWindow, SpinButton, TextView};
use libadwaita::prelude::*;
use libadwaita::{ActionRow, PreferencesGroup, PreferencesPage};
use weightlifting_core::{
    BackoffParams, IntensityParams, SchemeTemplate, TopBackoffParams, TopSetParams,
};

/// Scheme card widget for displaying and editing scheme templates
#[allow(dead_code)]
pub struct SchemeCard {
    pub widget: Box,
    pub template: Option<SchemeTemplate>,
}

#[allow(dead_code)]
impl SchemeCard {
    pub fn new() -> Self {
        let widget = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(12)
            .margin_start(16)
            .margin_end(16)
            .margin_top(12)
            .margin_bottom(12)
            .css_classes(vec!["card".to_string()])
            .build();

        Self {
            widget,
            template: None,
        }
    }

    /// Create a scheme card for top-backoff template editing
    pub fn for_top_backoff(exercise: &str) -> Self {
        let card = Self::new();

        // Card header
        let header = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(12)
            .build();

        let title = Label::builder()
            .label(format!("Top-Backoff Scheme: {}", exercise))
            .css_classes(vec!["heading".to_string()])
            .halign(gtk4::Align::Start)
            .hexpand(true)
            .build();

        let edit_btn = Button::with_label("Edit Template");
        let preview_btn = Button::with_label("Preview Expansion");

        header.append(&title);
        header.append(&edit_btn);
        header.append(&preview_btn);

        card.widget.append(&header);

        // Template parameters section
        let params_section = card.create_top_backoff_params_ui();
        card.widget.append(&params_section);

        // Preview section (initially hidden)
        let preview_section = card.create_preview_section();
        card.widget.append(&preview_section);

        // Set up button callbacks
        card.setup_button_callbacks(edit_btn, preview_btn, exercise);

        card
    }

    /// Create UI for editing top-backoff parameters
    fn create_top_backoff_params_ui(&self) -> Box {
        let params_box = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(8)
            .build();

        let group_label = Label::builder()
            .label("Template Parameters")
            .css_classes(vec!["heading".to_string()])
            .halign(gtk4::Align::Start)
            .build();
        params_box.append(&group_label);

        // Top set parameters
        let top_section = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(4)
            .css_classes(vec!["linked".to_string()])
            .build();

        let top_label = Label::builder()
            .label("Top Set")
            .halign(gtk4::Align::Start)
            .css_classes(vec!["caption".to_string()])
            .build();
        top_section.append(&top_label);

        // Top set reps
        let reps_row = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .build();

        let reps_label = Label::new(Some("Reps:"));
        reps_label.set_width_chars(12);
        let reps_spin = SpinButton::with_range(1.0, 20.0, 1.0);
        reps_spin.set_value(5.0);

        reps_row.append(&reps_label);
        reps_row.append(&reps_spin);
        top_section.append(&reps_row);

        // Top set intensity
        let intensity_row = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .build();

        let intensity_label = Label::new(Some("RPE:"));
        intensity_label.set_width_chars(12);
        let intensity_spin = SpinButton::with_range(1.0, 10.0, 0.5);
        intensity_spin.set_value(8.0);

        intensity_row.append(&intensity_label);
        intensity_row.append(&intensity_spin);
        top_section.append(&intensity_row);

        params_box.append(&top_section);

        // Backoff sets parameters
        let backoff_section = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(4)
            .css_classes(vec!["linked".to_string()])
            .build();

        let backoff_label = Label::builder()
            .label("Backoff Sets")
            .halign(gtk4::Align::Start)
            .css_classes(vec!["caption".to_string()])
            .build();
        backoff_section.append(&backoff_label);

        // Backoff 1
        let backoff1_row = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .build();

        let b1_percent_label = Label::new(Some("Percent:"));
        b1_percent_label.set_width_chars(12);
        let b1_percent_spin = SpinButton::with_range(0.5, 1.0, 0.05);
        b1_percent_spin.set_value(0.85);

        let b1_sets_label = Label::new(Some("Sets:"));
        let b1_sets_spin = SpinButton::with_range(1.0, 10.0, 1.0);
        b1_sets_spin.set_value(3.0);

        let b1_reps_label = Label::new(Some("Reps:"));
        let b1_reps_spin = SpinButton::with_range(1.0, 20.0, 1.0);
        b1_reps_spin.set_value(5.0);

        backoff1_row.append(&b1_percent_label);
        backoff1_row.append(&b1_percent_spin);
        backoff1_row.append(&b1_sets_label);
        backoff1_row.append(&b1_sets_spin);
        backoff1_row.append(&b1_reps_label);
        backoff1_row.append(&b1_reps_spin);

        backoff_section.append(&backoff1_row);
        params_box.append(&backoff_section);

        params_box
    }

    /// Create preview section for expanded scheme
    fn create_preview_section(&self) -> Box {
        let preview_box = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(8)
            .build();

        let preview_label = Label::builder()
            .label("Expansion Preview")
            .css_classes(vec!["heading".to_string()])
            .halign(gtk4::Align::Start)
            .build();

        let preview_text = TextView::builder()
            .editable(false)
            .css_classes(vec!["monospace".to_string()])
            .build();

        let preview_buffer = preview_text.buffer();
        preview_buffer.set_text("Click 'Preview Expansion' to see the expanded scheme");

        let scrolled = ScrolledWindow::builder()
            .child(&preview_text)
            .min_content_height(150)
            .build();

        preview_box.append(&preview_label);
        preview_box.append(&scrolled);

        // Initially hidden
        preview_box.set_visible(false);

        preview_box
    }

    /// Set up button callbacks for edit and preview functionality
    fn setup_button_callbacks(&self, edit_btn: Button, preview_btn: Button, exercise: &str) {
        let exercise_name = exercise.to_string();

        // Edit button callback
        edit_btn.connect_clicked(move |_| {
            println!("Opening scheme template editor for: {}", exercise_name);
            // This would open a detailed editor dialog
            Self::show_template_editor_dialog(&exercise_name);
        });

        // Preview button callback
        let widget_clone = self.widget.clone();
        let exercise_clone = exercise.to_string();
        preview_btn.connect_clicked(move |_| {
            println!("Generating expansion preview for: {}", exercise_clone);
            Self::update_expansion_preview(&widget_clone, &exercise_clone);
        });
    }

    /// Show detailed template editor dialog
    fn show_template_editor_dialog(exercise: &str) {
        use gtk4::{Box as GtkBox, Dialog, DialogFlags, ResponseType};

        let dialog = Dialog::with_buttons(
            Some(&format!("Edit Scheme Template: {}", exercise)),
            crate::ui::util::parent_for_dialog().as_ref(),
            DialogFlags::MODAL,
            &[
                ("Cancel", ResponseType::Cancel),
                ("Apply", ResponseType::Apply),
            ],
        );

        let content = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .margin_start(20)
            .margin_end(20)
            .margin_top(20)
            .margin_bottom(20)
            .spacing(12)
            .build();

        // Create preferences page for structured editing
        let prefs_page = PreferencesPage::new();

        // Top set group
        let top_group = PreferencesGroup::builder()
            .title("Top Set Parameters")
            .build();

        let reps_row = ActionRow::builder()
            .title("Repetitions")
            .subtitle("Number of reps for the top set")
            .build();

        let reps_entry = Entry::builder().text("5").width_chars(10).build();
        reps_row.add_suffix(&reps_entry);
        top_group.add(&reps_row);

        let intensity_row = ActionRow::builder()
            .title("Intensity")
            .subtitle("RPE or load specification")
            .build();

        let intensity_entry = Entry::builder().text("@8 RPE").width_chars(15).build();
        intensity_row.add_suffix(&intensity_entry);
        top_group.add(&intensity_row);

        prefs_page.add(&top_group);

        // Backoff sets group
        let backoff_group = PreferencesGroup::builder()
            .title("Backoff Set Parameters")
            .build();

        let backoff_row = ActionRow::builder()
            .title("Backoff 1")
            .subtitle("85% × 3 sets × 5 reps")
            .build();

        let edit_backoff_btn = Button::with_label("Edit");
        backoff_row.add_suffix(&edit_backoff_btn);
        backoff_group.add(&backoff_row);

        prefs_page.add(&backoff_group);

        content.append(&prefs_page);
        dialog.content_area().append(&content);

        dialog.connect_response(|dialog, response| {
            match response {
                ResponseType::Apply => {
                    println!("Applying scheme template changes");
                    // This would save the template parameters
                }
                ResponseType::Cancel => {
                    println!("Cancelled template editing");
                }
                _ => {}
            }
            dialog.close();
        });

        dialog.present();
    }

    /// Update the expansion preview with computed scheme
    fn update_expansion_preview(widget: &Box, exercise: &str) {
        // Find the preview section in the widget tree
        let preview_section = Self::find_preview_section(widget);

        if let Some(preview_box) = preview_section {
            preview_box.set_visible(true);

            // Find the TextView in the preview section
            if let Some(scrolled) = preview_box.last_child() {
                if let Some(text_view) = scrolled.first_child() {
                    if let Some(text_view) = text_view.downcast_ref::<TextView>() {
                        let buffer = text_view.buffer();

                        // Create a sample top-backoff template and expand it
                        let template = Self::create_sample_top_backoff_template();

                        match template.expand(exercise) {
                            Ok(expanded) => {
                                let preview_text = format!(
                                    "Exercise: {}\nTotal Sets: {}\nTotal Volume: {} reps\nEstimated Duration: {} seconds\n\nExpanded Sets:\n{}",
                                    expanded.exercise,
                                    expanded.sets.len(),
                                    expanded.total_volume_reps,
                                    expanded.estimated_duration,
                                    expanded.sets.iter().enumerate().map(|(i, set)| {
                                        format!("  {}: {} sets × {} reps", i + 1,
                                               set.sets.unwrap_or(1),
                                               match &set.reps {
                                                   Some(reps) => format!("{:?}", reps),
                                                   None => "unspecified".to_string()
                                               })
                                    }).collect::<Vec<_>>().join("\n")
                                );
                                buffer.set_text(&preview_text);
                            }
                            Err(error) => {
                                buffer.set_text(&format!("Expansion Error: {}", error));
                            }
                        }
                    }
                }
            }
        }
    }

    /// Find the preview section widget in the hierarchy
    fn find_preview_section(widget: &Box) -> Option<Box> {
        // Simple implementation - find the last child which should be the preview
        let mut current = widget.last_child();
        while let Some(child) = current {
            if let Some(box_widget) = child.downcast_ref::<Box>() {
                // Check if this box contains a "Preview" label
                if let Some(first_child) = box_widget.first_child() {
                    if let Some(label) = first_child.downcast_ref::<Label>() {
                        if label.text().contains("Preview") {
                            return Some(box_widget.clone());
                        }
                    }
                }
            }
            current = child.prev_sibling();
        }
        None
    }

    /// Create a sample top-backoff template for preview
    fn create_sample_top_backoff_template() -> SchemeTemplate {
        SchemeTemplate::TopBackoff(TopBackoffParams {
            top: TopSetParams {
                reps: 5,
                intensity: IntensityParams::Rpe { rpe: 8.0 },
            },
            backoff: vec![BackoffParams {
                percent: 0.85,
                sets: 3,
                reps: 5,
            }],
            cap_min_load: None,
        })
    }
}

impl Default for SchemeCard {
    fn default() -> Self {
        Self::new()
    }
}
