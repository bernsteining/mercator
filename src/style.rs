use std::borrow::Cow;

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
fn default_fill() -> String { "white".to_string() }
fn default_fill_opacity() -> f64 { 1.0 }

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
    pub tissot: Option<TissotConfig>,
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

#[derive(Debug, Deserialize)]
pub struct TissotConfig {
    #[serde(default = "default_tissot_step")]
    pub step: f64,
    #[serde(default = "default_tissot_radius")]
    pub radius: f64,
    #[serde(default = "default_tissot_fill")]
    pub fill: String,
    #[serde(default = "default_tissot_fill_opacity")]
    pub fill_opacity: f64,
    #[serde(default = "default_tissot_stroke")]
    pub stroke: String,
    #[serde(default = "default_tissot_stroke_width")]
    pub stroke_width: f64,
    #[serde(default = "default_tissot_max_lat")]
    pub max_lat: f64,
}

fn default_tissot_step() -> f64 { 30.0 }
fn default_tissot_radius() -> f64 { 5.0 }
fn default_tissot_fill() -> String { "red".to_string() }
fn default_tissot_fill_opacity() -> f64 { 0.3 }
fn default_tissot_stroke() -> String { "red".to_string() }
fn default_tissot_stroke_width() -> f64 { 0.5 }
fn default_tissot_max_lat() -> f64 { 60.0 }

impl Default for StyleConfig {
    fn default() -> Self {
        Self {
            stroke: "black".to_string(),
            stroke_width: 0.05,
            fill: "white".to_string(),
            fill_opacity: 1.0,
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
            tissot: None,
        }
    }
}

pub struct ResolvedStyle<'a> {
    pub stroke: Cow<'a, str>,
    pub stroke_width: f64,
    pub fill: Cow<'a, str>,
    pub fill_opacity: f64,
    pub point_radius: f64,
    pub point_color: Option<Cow<'a, str>>,
    pub fill_pattern: Option<Cow<'a, str>>,
}

fn resolve_field<'a>(
    template: &'a str,
    fallback: &'a str,
    properties: Option<&serde_json::Map<String, serde_json::Value>>,
) -> Cow<'a, str> {
    if !template.contains('{') {
        return Cow::Borrowed(template);
    }
    if let Some(props) = properties {
        if let Some(val) = interpolate_template(template, props) {
            return Cow::Owned(val);
        }
    }
    Cow::Borrowed(fallback)
}

pub fn resolve_style<'a>(
    config: &'a StyleConfig,
    properties: Option<&serde_json::Map<String, serde_json::Value>>,
) -> ResolvedStyle<'a> {
    let fill_pattern = config.fill_pattern.as_deref().and_then(|template| {
        let resolved = resolve_field(template, "", properties);
        if resolved.is_empty() { None } else { Some(resolved) }
    });

    let point_color = config.point_color.as_deref().map(|template| {
        resolve_field(template, "none", properties)
    });

    ResolvedStyle {
        stroke: resolve_field(&config.stroke, "black", properties),
        stroke_width: config.stroke_width,
        fill: resolve_field(&config.fill, "none", properties),
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
    let mut result = String::with_capacity(template.len());
    let bytes = template.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] == b'{' {
            i += 1;
            let key_start = i;
            while i < bytes.len() && bytes[i] != b'}' {
                i += 1;
            }
            let key = &template[key_start..i];
            let val = properties.get(key)?;
            match val {
                serde_json::Value::String(s) => result.push_str(s),
                serde_json::Value::Number(n) => {
                    use std::fmt::Write;
                    let _ = write!(result, "{}", n);
                }
                serde_json::Value::Bool(b) => {
                    result.push_str(if *b { "true" } else { "false" });
                }
                _ => return None,
            }
            if i < bytes.len() {
                i += 1; // skip '}'
            }
        } else {
            result.push(bytes[i] as char);
            i += 1;
        }
    }

    if result.is_empty() {
        None
    } else {
        Some(result)
    }
}
