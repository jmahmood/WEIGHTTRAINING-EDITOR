use super::data_transforms::DataTransforms;
use super::vega_specs::{AxisEncoding, ChartTemplates, ColorEncoding, VegaLiteSpec};
use chrono::NaiveDate;
use serde_json::{json, Value};
use std::collections::HashMap;

/// Built-in chart types for Sprint 3
pub struct BuiltinCharts;

impl BuiltinCharts {
    /// 1. E1RM over time (line chart with exercise picker)
    pub fn e1rm_over_time(
        e1rm_data: &[(String, NaiveDate, f64)], // (exercise, date, e1rm_kg)
        exercise: Option<&str>,
        title: Option<String>,
    ) -> VegaLiteSpec {
        let chart_data = DataTransforms::e1rm_to_chart_data(e1rm_data, exercise);

        let chart_title = title.unwrap_or_else(|| {
            if let Some(ex) = exercise {
                format!("E1RM Progress - {}", ex)
            } else {
                "E1RM Progress - All Exercises".to_string()
            }
        });

        let mut spec = ChartTemplates::line_chart()
            .title(chart_title)
            .dimensions(800, 400)
            .x_axis("date", "temporal", Some("Date".to_string()))
            .y_axis(
                "e1rm_kg",
                "quantitative",
                Some("Estimated 1RM (kg)".to_string()),
            );

        // If showing multiple exercises, color by exercise
        if exercise.is_none() {
            spec = spec.color_field("exercise", "nominal");
        } else {
            spec = spec.color_value("steelblue");
        }

        spec.data_values(chart_data)
    }

    /// 2. Weekly volume by body-part (stacked bar chart)
    pub fn weekly_volume_by_bodypart(
        volume_data: &[(String, NaiveDate, u32, u32, f64)], // (exercise, week_start, sets, reps, tonnage)
        body_part_map: Option<&HashMap<String, String>>,
        metric: VolumeMetric,
        title: Option<String>,
    ) -> VegaLiteSpec {
        let default_body_parts = DataTransforms::get_default_body_part_map();
        let body_parts = body_part_map.unwrap_or(&default_body_parts);
        let chart_data = DataTransforms::volume_to_chart_data(volume_data, body_parts);

        let (y_field, y_title) = match metric {
            VolumeMetric::Sets => ("sets", "Total Sets"),
            VolumeMetric::Reps => ("reps", "Total Reps"),
            VolumeMetric::TonnageKg => ("tonnage_kg", "Tonnage (kg)"),
        };

        let chart_title = title.unwrap_or_else(|| format!("Weekly {} by Body Part", y_title));

        ChartTemplates::stacked_bar_chart()
            .title(chart_title)
            .dimensions(900, 500)
            .x_axis("week", "ordinal", Some("Week".to_string()))
            .y_axis(y_field, "quantitative", Some(y_title.to_string()))
            .color_field("body_part", "nominal")
            .data_values(chart_data)
    }

    /// 3. PR Board (table/bar showing best sets/1RMs with dates)
    pub fn pr_board(
        pr_data: &[(String, String, NaiveDate, f64, Option<u32>)], // (exercise, pr_type, date, value, reps)
        exercise_filter: Option<&str>,
        display_mode: PRDisplayMode,
        title: Option<String>,
    ) -> VegaLiteSpec {
        let chart_data = DataTransforms::pr_to_chart_data(pr_data, exercise_filter);

        let chart_title = title.unwrap_or_else(|| {
            if let Some(ex) = exercise_filter {
                format!("Personal Records - {}", ex)
            } else {
                "Personal Records Board".to_string()
            }
        });

        match display_mode {
            PRDisplayMode::Table => {
                // For table view, we'll use a simple scatter plot with text marks
                // In a real implementation, this would be handled by the UI as a table
                ChartTemplates::scatter_plot()
                    .title(chart_title)
                    .dimensions(800, 600)
                    .mark("point")
                    .x_axis("date", "temporal", Some("Date Achieved".to_string()))
                    .y_axis("value_kg", "quantitative", Some("Weight (kg)".to_string()))
                    .color_field("pr_type", "nominal")
                    .data_values(chart_data)
            }
            PRDisplayMode::Bar => ChartTemplates::bar_chart()
                .title(chart_title)
                .dimensions(800, 500)
                .x_axis("exercise", "nominal", Some("Exercise".to_string()))
                .y_axis("value_kg", "quantitative", Some("Weight (kg)".to_string()))
                .color_field("pr_type", "nominal")
                .data_values(chart_data),
        }
    }

    /// 4. Session frequency heatmap (calendar style)
    pub fn session_frequency_heatmap(
        session_dates: &[NaiveDate],
        start_date: NaiveDate,
        end_date: NaiveDate,
        title: Option<String>,
    ) -> VegaLiteSpec {
        let chart_data =
            DataTransforms::session_frequency_to_chart_data(session_dates, start_date, end_date);

        let chart_title = title.unwrap_or_else(|| {
            format!(
                "Training Frequency: {} to {}",
                start_date.format("%Y-%m-%d"),
                end_date.format("%Y-%m-%d")
            )
        });

        let mut spec = ChartTemplates::heatmap()
            .title(chart_title)
            .dimensions(800, 200);

        // Custom encoding for heatmap
        spec.encoding.x = Some(AxisEncoding {
            field: "date".to_string(),
            encoding_type: "temporal".to_string(),
            axis: Some(super::vega_specs::AxisSpec {
                title: Some("Date".to_string()),
                format: Some("%b %d".to_string()),
                label_angle: Some(-45.0),
                grid: Some(false),
            }),
            scale: None,
            title: None,
        });

        spec.encoding.y = Some(AxisEncoding {
            field: "day_of_week".to_string(),
            encoding_type: "ordinal".to_string(),
            axis: Some(super::vega_specs::AxisSpec {
                title: Some("Day".to_string()),
                format: None,
                label_angle: None,
                grid: Some(false),
            }),
            scale: Some(super::vega_specs::ScaleSpec {
                domain: Some(vec![
                    json!(1),
                    json!(2),
                    json!(3),
                    json!(4),
                    json!(5),
                    json!(6),
                    json!(7),
                ]),
                range: Some(vec![
                    json!("Mon"),
                    json!("Tue"),
                    json!("Wed"),
                    json!("Thu"),
                    json!("Fri"),
                    json!("Sat"),
                    json!("Sun"),
                ]),
                scale_type: Some("ordinal".to_string()),
            }),
            title: None,
        });

        spec.encoding.color = Some(ColorEncoding {
            field: Some("intensity".to_string()),
            encoding_type: Some("ordinal".to_string()),
            scale: Some(super::vega_specs::ScaleSpec {
                domain: Some(vec![json!(0), json!(1), json!(2)]),
                range: Some(vec![
                    json!("#ebedf0"), // No sessions - light gray
                    json!("#9be9a8"), // Single session - light green
                    json!("#40c463"), // Multiple sessions - dark green
                ]),
                scale_type: Some("ordinal".to_string()),
            }),
            legend: Some(super::vega_specs::LegendSpec {
                title: Some("Sessions".to_string()),
                orient: Some("right".to_string()),
            }),
            value: None,
        });

        spec.data_values(chart_data)
    }
}

/// Volume metrics for chart display
#[derive(Debug, Clone, Copy)]
pub enum VolumeMetric {
    Sets,
    Reps,
    TonnageKg,
}

/// PR board display modes
#[derive(Debug, Clone, Copy)]
pub enum PRDisplayMode {
    Table,
    Bar,
}

/// Chart generation helper that integrates with cached metrics
pub struct ChartGenerator;

impl ChartGenerator {
    /// Generate E1RM chart from cached data with filtering
    pub fn generate_e1rm_chart(
        cache_data: &[Value], // JSON data from cache
        exercise_filter: Option<&str>,
        date_range: Option<(NaiveDate, NaiveDate)>,
    ) -> Result<VegaLiteSpec, Box<dyn std::error::Error>> {
        // Parse cached data
        let mut e1rm_data = Vec::new();
        for item in cache_data {
            let exercise = item["exercise"].as_str().ok_or("Missing exercise")?;
            let date_str = item["date"].as_str().ok_or("Missing date")?;
            let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")?;
            let e1rm = item["e1rm_kg"].as_f64().ok_or("Missing e1rm_kg")?;

            // Apply date filter if specified
            if let Some((start, end)) = date_range {
                if date < start || date > end {
                    continue;
                }
            }

            e1rm_data.push((exercise.to_string(), date, e1rm));
        }

        Ok(BuiltinCharts::e1rm_over_time(
            &e1rm_data,
            exercise_filter,
            None,
        ))
    }

    /// Generate all charts for a comprehensive stats view
    pub fn generate_all_charts(
        e1rm_data: &[(String, NaiveDate, f64)],
        volume_data: &[(String, NaiveDate, u32, u32, f64)],
        pr_data: &[(String, String, NaiveDate, f64, Option<u32>)],
        session_dates: &[NaiveDate],
        date_range: (NaiveDate, NaiveDate),
    ) -> Vec<(String, VegaLiteSpec)> {
        let mut charts = Vec::new();

        // E1RM chart (top exercises only for overview)
        let top_exercises = DataTransforms::get_unique_exercises(e1rm_data);
        if !top_exercises.is_empty() {
            let chart = BuiltinCharts::e1rm_over_time(e1rm_data, None, None);
            charts.push(("e1rm_overview".to_string(), chart));
        }

        // Weekly volume chart
        let volume_chart = BuiltinCharts::weekly_volume_by_bodypart(
            volume_data,
            None,
            VolumeMetric::TonnageKg,
            None,
        );
        charts.push(("weekly_volume".to_string(), volume_chart));

        // PR board
        let pr_chart = BuiltinCharts::pr_board(pr_data, None, PRDisplayMode::Bar, None);
        charts.push(("pr_board".to_string(), pr_chart));

        // Session frequency heatmap
        let heatmap = BuiltinCharts::session_frequency_heatmap(
            session_dates,
            date_range.0,
            date_range.1,
            None,
        );
        charts.push(("session_frequency".to_string(), heatmap));

        charts
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_e1rm_chart_generation() {
        let data = vec![
            (
                "BP.BB.FLAT".to_string(),
                NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
                120.0,
            ),
            (
                "BP.BB.FLAT".to_string(),
                NaiveDate::from_ymd_opt(2025, 1, 8).unwrap(),
                125.0,
            ),
        ];

        let spec = BuiltinCharts::e1rm_over_time(&data, Some("BP.BB.FLAT"), None);
        let json = spec.to_json();

        assert!(json["title"].as_str().unwrap().contains("BP.BB.FLAT"));
        assert_eq!(json["mark"], "line");
        assert_eq!(json["encoding"]["x"]["field"], "date");
        assert_eq!(json["encoding"]["y"]["field"], "e1rm_kg");
    }

    #[test]
    fn test_volume_chart_generation() {
        let data = vec![
            (
                "BP.BB.FLAT".to_string(),
                NaiveDate::from_ymd_opt(2025, 1, 6).unwrap(),
                3,
                15,
                450.0,
            ),
            (
                "SQ.BB.BACK".to_string(),
                NaiveDate::from_ymd_opt(2025, 1, 6).unwrap(),
                4,
                20,
                800.0,
            ),
        ];

        let spec = BuiltinCharts::weekly_volume_by_bodypart(&data, None, VolumeMetric::Sets, None);
        let json = spec.to_json();

        assert_eq!(json["mark"], "bar");
        assert_eq!(json["encoding"]["y"]["field"], "sets");
        assert!(json["title"].as_str().unwrap().contains("Sets"));
    }

    #[test]
    fn test_heatmap_generation() {
        let sessions = vec![
            NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2025, 1, 3).unwrap(),
        ];

        let start = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2025, 1, 7).unwrap();

        let spec = BuiltinCharts::session_frequency_heatmap(&sessions, start, end, None);
        let json = spec.to_json();

        assert_eq!(json["mark"], "rect");
        assert_eq!(json["encoding"]["x"]["field"], "date");
        assert_eq!(json["encoding"]["color"]["field"], "intensity");
    }
}
