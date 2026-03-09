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
}

#[derive(Debug, Deserialize)]
pub struct StyleConfig {
    pub stroke: String,
    pub stroke_width: f64,
    pub fill: String,
    pub fill_opacity: f64,
    pub viewbox: Option<(f64, f64, f64, f64)>,
    pub viewbox_padding: Option<f64>,
    pub label_color: Option<String>,
    pub label_font_size: Option<f64>,
    pub label_font_family: Option<String>,
    pub label: Option<LabelConfig>,
    pub point_radius: Option<f64>,
    pub fill_pattern: Option<String>,
}

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
            fill_pattern: None,
        }
    }
}

pub struct ResolvedStyle {
    pub stroke: String,
    pub stroke_width: f64,
    pub fill: String,
    pub fill_opacity: f64,
    pub point_radius: f64,
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

    ResolvedStyle {
        stroke: resolve(&config.stroke, "black"),
        stroke_width: config.stroke_width,
        fill: resolve(&config.fill, "none"),
        fill_opacity: config.fill_opacity,
        point_radius: config.point_radius.unwrap_or(config.stroke_width * 5.0),
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
