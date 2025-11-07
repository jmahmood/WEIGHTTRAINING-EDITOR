use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;

/// Vega-Lite chart specification builder
#[derive(Debug, Clone)]
pub struct VegaLiteSpec {
    pub title: Option<String>,
    pub description: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub mark: MarkSpec,
    pub encoding: EncodingSpec,
    pub data: DataSpec,
    pub config: Option<Value>,
}

/// Data specification for Vega-Lite
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DataSpec {
    Values {
        values: Vec<Value>,
    },
    Url {
        url: String,
        format: Option<DataFormat>,
    },
}

/// Data format specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataFormat {
    #[serde(rename = "type")]
    pub format_type: String, // "json", "csv", etc.
    pub parse: Option<HashMap<String, String>>,
}

/// Mark specification
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MarkSpec {
    Simple(String), // "line", "bar", "point", etc.
    Detailed(MarkObject),
}

/// Detailed mark object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkObject {
    #[serde(rename = "type")]
    pub mark_type: String,
    pub color: Option<String>,
    pub size: Option<f64>,
    pub opacity: Option<f64>,
    pub stroke: Option<String>,
    #[serde(rename = "strokeWidth")]
    pub stroke_width: Option<f64>,
}

/// Encoding specification
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EncodingSpec {
    pub x: Option<AxisEncoding>,
    pub y: Option<AxisEncoding>,
    pub color: Option<ColorEncoding>,
    pub size: Option<SizeEncoding>,
    pub shape: Option<ShapeEncoding>,
}

/// Axis encoding specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AxisEncoding {
    pub field: String,
    #[serde(rename = "type")]
    pub encoding_type: String, // "quantitative", "ordinal", "temporal", "nominal"
    pub axis: Option<AxisSpec>,
    pub scale: Option<ScaleSpec>,
    pub title: Option<String>,
}

/// Axis specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AxisSpec {
    pub title: Option<String>,
    pub format: Option<String>,
    #[serde(rename = "labelAngle")]
    pub label_angle: Option<f64>,
    pub grid: Option<bool>,
}

/// Scale specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScaleSpec {
    pub domain: Option<Vec<Value>>,
    pub range: Option<Vec<Value>>,
    #[serde(rename = "type")]
    pub scale_type: Option<String>, // "linear", "log", "time", etc.
}

/// Color encoding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorEncoding {
    pub field: Option<String>,
    #[serde(rename = "type")]
    pub encoding_type: Option<String>,
    pub scale: Option<ScaleSpec>,
    pub legend: Option<LegendSpec>,
    pub value: Option<String>, // Fixed color value
}

/// Size encoding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SizeEncoding {
    pub field: Option<String>,
    #[serde(rename = "type")]
    pub encoding_type: Option<String>,
    pub scale: Option<ScaleSpec>,
    pub value: Option<f64>, // Fixed size value
}

/// Shape encoding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShapeEncoding {
    pub field: Option<String>,
    #[serde(rename = "type")]
    pub encoding_type: Option<String>,
    pub value: Option<String>, // Fixed shape value
}

/// Legend specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegendSpec {
    pub title: Option<String>,
    pub orient: Option<String>, // "left", "right", "top", "bottom"
}

impl Default for VegaLiteSpec {
    fn default() -> Self {
        Self::new()
    }
}

impl VegaLiteSpec {
    /// Create a new Vega-Lite specification
    pub fn new() -> Self {
        Self {
            title: None,
            description: None,
            width: None,
            height: None,
            mark: MarkSpec::Simple("point".to_string()),
            encoding: EncodingSpec::default(),
            data: DataSpec::Values { values: vec![] },
            config: None,
        }
    }

    /// Set the title
    pub fn title(mut self, title: String) -> Self {
        self.title = Some(title);
        self
    }

    /// Set the dimensions
    pub fn dimensions(mut self, width: u32, height: u32) -> Self {
        self.width = Some(width);
        self.height = Some(height);
        self
    }

    /// Set the mark type
    pub fn mark(mut self, mark_type: &str) -> Self {
        self.mark = MarkSpec::Simple(mark_type.to_string());
        self
    }

    /// Set detailed mark with styling
    pub fn detailed_mark(mut self, mark_type: &str, color: Option<String>) -> Self {
        self.mark = MarkSpec::Detailed(MarkObject {
            mark_type: mark_type.to_string(),
            color,
            size: None,
            opacity: None,
            stroke: None,
            stroke_width: None,
        });
        self
    }

    /// Set X axis encoding
    pub fn x_axis(mut self, field: &str, encoding_type: &str, title: Option<String>) -> Self {
        self.encoding.x = Some(AxisEncoding {
            field: field.to_string(),
            encoding_type: encoding_type.to_string(),
            axis: Some(AxisSpec {
                title,
                format: None,
                label_angle: None,
                grid: None,
            }),
            scale: None,
            title: None,
        });
        self
    }

    /// Set Y axis encoding
    pub fn y_axis(mut self, field: &str, encoding_type: &str, title: Option<String>) -> Self {
        self.encoding.y = Some(AxisEncoding {
            field: field.to_string(),
            encoding_type: encoding_type.to_string(),
            axis: Some(AxisSpec {
                title,
                format: None,
                label_angle: None,
                grid: None,
            }),
            scale: None,
            title: None,
        });
        self
    }

    /// Set color encoding
    pub fn color_field(mut self, field: &str, encoding_type: &str) -> Self {
        self.encoding.color = Some(ColorEncoding {
            field: Some(field.to_string()),
            encoding_type: Some(encoding_type.to_string()),
            scale: None,
            legend: None,
            value: None,
        });
        self
    }

    /// Set fixed color
    pub fn color_value(mut self, color: &str) -> Self {
        self.encoding.color = Some(ColorEncoding {
            field: None,
            encoding_type: None,
            scale: None,
            legend: None,
            value: Some(color.to_string()),
        });
        self
    }

    /// Set the data from JSON values
    pub fn data_values(mut self, values: Vec<Value>) -> Self {
        self.data = DataSpec::Values { values };
        self
    }

    /// Convert to JSON representation
    pub fn to_json(&self) -> Value {
        let mut spec = json!({
            "$schema": "https://vega.github.io/schema/vega-lite/v5.json",
            "mark": self.mark,
            "encoding": self.encoding,
            "data": self.data
        });

        if let Some(title) = &self.title {
            spec["title"] = json!(title);
        }

        if let Some(description) = &self.description {
            spec["description"] = json!(description);
        }

        if let Some(width) = self.width {
            spec["width"] = json!(width);
        }

        if let Some(height) = self.height {
            spec["height"] = json!(height);
        }

        if let Some(config) = &self.config {
            spec["config"] = config.clone();
        }

        spec
    }

    /// Convert to JSON string
    pub fn to_json_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self.to_json())
    }
}

/// Chart template builders for common chart types
pub struct ChartTemplates;

impl ChartTemplates {
    /// Create a line chart template for time series data
    pub fn line_chart() -> VegaLiteSpec {
        VegaLiteSpec::new()
            .dimensions(600, 400)
            .mark("line")
            .color_value("steelblue")
    }

    /// Create a bar chart template
    pub fn bar_chart() -> VegaLiteSpec {
        VegaLiteSpec::new()
            .dimensions(600, 400)
            .mark("bar")
            .color_value("steelblue")
    }

    /// Create a stacked bar chart template
    pub fn stacked_bar_chart() -> VegaLiteSpec {
        VegaLiteSpec::new().dimensions(600, 400).mark("bar")
    }

    /// Create a scatter plot template
    pub fn scatter_plot() -> VegaLiteSpec {
        VegaLiteSpec::new().dimensions(600, 400).mark("circle")
    }

    /// Create a heatmap template
    pub fn heatmap() -> VegaLiteSpec {
        VegaLiteSpec::new().dimensions(600, 400).mark("rect")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_basic_spec_creation() {
        let spec = VegaLiteSpec::new()
            .title("Test Chart".to_string())
            .dimensions(400, 300)
            .mark("line")
            .x_axis("date", "temporal", Some("Date".to_string()))
            .y_axis("value", "quantitative", Some("Value".to_string()))
            .data_values(vec![
                json!({"date": "2025-01-01", "value": 100}),
                json!({"date": "2025-01-02", "value": 120}),
            ]);

        let json_spec = spec.to_json();

        assert!(json_spec["$schema"].as_str().is_some());
        assert_eq!(json_spec["title"], "Test Chart");
        assert_eq!(json_spec["width"], 400);
        assert_eq!(json_spec["height"], 300);
        assert_eq!(json_spec["mark"], "line");

        // Check encoding
        assert!(json_spec["encoding"]["x"].is_object());
        assert!(json_spec["encoding"]["y"].is_object());
        assert_eq!(json_spec["encoding"]["x"]["field"], "date");
        assert_eq!(json_spec["encoding"]["x"]["type"], "temporal");

        // Check data
        assert!(json_spec["data"]["values"].is_array());
        assert_eq!(json_spec["data"]["values"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_chart_templates() {
        let line_chart = ChartTemplates::line_chart();
        let json = line_chart.to_json();
        assert_eq!(json["mark"], "line");
        assert_eq!(json["width"], 600);
        assert_eq!(json["height"], 400);

        let bar_chart = ChartTemplates::bar_chart();
        let json = bar_chart.to_json();
        assert_eq!(json["mark"], "bar");
    }
}
