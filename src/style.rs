use crate::projection::ProjectionConfig;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct LabelLine {
    pub text: String,
    pub font_size: Option<f64>,
    pub color: Option<String>,
    pub font_family: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum LabelConfig {
    Simple(String),
    Multi(Vec<LabelLine>),
}

pub struct LabelInstance {
    pub x: f64,
    pub y: f64,
    pub text: String,
    pub font_size: f64,
    pub color: String,
    pub font_family: String,
    pub anchor: &'static str,
}

fn default_stroke() -> String { "black".to_string() }
fn default_stroke_width() -> f64 { 0.05 }
fn default_fill() -> String { "red".to_string() }
fn default_fill_opacity() -> f64 { 0.5 }

#[derive(Debug, Deserialize)]
pub struct StyleConfig {
    #[serde(default = "default_stroke")]
    pub stroke: String,
    #[serde(default = "default_stroke_width")]
    pub stroke_width: f64,
    #[serde(default = "default_fill")]
    pub fill: String,
    #[serde(default = "default_fill_opacity")]
    pub fill_opacity: f64,
    pub viewbox: Option<(f64, f64, f64, f64)>,
    pub viewbox_padding: Option<f64>,
    pub label_color: Option<String>,
    pub label_font_size: Option<f64>,
    pub label_font_family: Option<String>,
    pub label: Option<LabelConfig>,
    pub point_radius: Option<f64>,
    pub point_color: Option<String>,
    pub fill_pattern: Option<String>,
    pub projection: Option<ProjectionConfig>,
    pub graticule: Option<GraticuleConfig>,
}

#[derive(Debug, Deserialize)]
pub struct GraticuleConfig {
    #[serde(default = "default_graticule_step")]
    pub step: f64,
    #[serde(default = "default_graticule_color")]
    pub color: String,
    #[serde(default = "default_graticule_width")]
    pub width: f64,
    #[serde(default = "default_graticule_opacity")]
    pub opacity: f64,
}

fn default_graticule_step() -> f64 { 15.0 }
fn default_graticule_color() -> String { "#ccc".to_string() }
fn default_graticule_width() -> f64 { 0.5 }
fn default_graticule_opacity() -> f64 { 0.6 }

impl Default for StyleConfig {
    fn default() -> Self {
        Self {
            stroke: "black".to_string(),
            stroke_width: 0.05,
            fill: "red".to_string(),
            fill_opacity: 0.5,
            viewbox: None,
            viewbox_padding: None,
            label_color: Some("black".to_string()),
            label_font_size: Some(0.3),
            label_font_family: Some("Arial".to_string()),
            label: None,
            point_radius: None,
            point_color: None,
            fill_pattern: None,
            projection: None,
            graticule: None,
        }
    }
}

pub struct ResolvedStyle {
    pub stroke: String,
    pub stroke_width: f64,
    pub fill: String,
    pub fill_opacity: f64,
    pub point_radius: f64,
    pub point_color: Option<String>,
    pub fill_pattern: Option<String>,
}

pub fn resolve_style(
    config: &StyleConfig,
    properties: Option<&serde_json::Map<String, serde_json::Value>>,
) -> ResolvedStyle {
    let resolve = |template: &str, fallback: &str| -> String {
        if let Some(props) = properties {
            if template.contains('{') {
                if let Some(val) = interpolate_template(template, props) {
                    return val;
                }
            }
        }
        if template.contains('{') {
            fallback.to_string()
        } else {
            template.to_string()
        }
    };

    let fill_pattern = config.fill_pattern.as_ref().and_then(|template| {
        let resolved = resolve(template, "");
        if resolved.is_empty() { None } else { Some(resolved) }
    });

    let point_color = config.point_color.as_ref().map(|template| {
        resolve(template, "none")
    });

    ResolvedStyle {
        stroke: resolve(&config.stroke, "black"),
        stroke_width: config.stroke_width,
        fill: resolve(&config.fill, "none"),
        fill_opacity: config.fill_opacity,
        point_radius: config.point_radius.unwrap_or(config.stroke_width * 5.0),
        point_color,
        fill_pattern,
    }
}

pub fn interpolate_template(
    template: &str,
    properties: &serde_json::Map<String, serde_json::Value>,
) -> Option<String> {
    let mut result = String::new();
    let mut chars = template.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '{' {
            let mut key = String::new();
            for ch in chars.by_ref() {
                if ch == '}' {
                    break;
                }
                key.push(ch);
            }
            let val = properties.get(&key)?;
            match val {
                serde_json::Value::String(s) => result.push_str(s),
                serde_json::Value::Number(n) => result.push_str(&n.to_string()),
                serde_json::Value::Bool(b) => result.push_str(&b.to_string()),
                _ => return None,
            }
        } else {
            result.push(ch);
        }
    }

    if result.is_empty() {
        None
    } else {
        Some(result)
    }
}
