use crate::geometry::push_f64;
use crate::style::{
    interpolate_template, LabelConfig, LabelInstance, StyleConfig,
};

const DEFAULT_LABEL_FONT_SIZE: f64 = 0.3;
const LABEL_LINE_SPACING: f64 = 1.2;

fn xml_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(c),
        }
    }
    out
}

pub fn write_label(out: &mut String, label: &LabelInstance) {
    out.push_str(r#"<text x=""#);
    push_f64(out, label.x);
    out.push_str(r#"" y=""#);
    push_f64(out, label.y);
    out.push_str(r#"" font-size=""#);
    push_f64(out, label.font_size);
    out.push_str(r#"" font-family=""#);
    out.push_str(&label.font_family);
    out.push_str(r#"" fill=""#);
    out.push_str(&label.color);
    out.push_str(r#"" text-anchor=""#);
    out.push_str(label.anchor);
    out.push_str(r#"" dominant-baseline="middle">"#);
    out.push_str(&xml_escape(&label.text));
    out.push_str("</text>");
}

pub fn build_labels(
    labels: &mut Vec<LabelInstance>,
    config: &StyleConfig,
    properties: &serde_json::Map<String, serde_json::Value>,
    cx: f64,
    cy: f64,
    is_point: bool,
) {
    let label_config = match &config.label {
        Some(lc) => lc,
        None => return,
    };

    let default_font_size = config.label_font_size.unwrap_or(DEFAULT_LABEL_FONT_SIZE);
    let default_color = config.label_color.as_deref().unwrap_or("black");
    let default_font_family = config.label_font_family.as_deref().unwrap_or("Arial");
    let anchor = if is_point { "start" } else { "middle" };

    match label_config {
        LabelConfig::Simple(template) => {
            if let Some(text) = interpolate_template(template, properties) {
                labels.push(LabelInstance {
                    x: cx,
                    y: cy,
                    text,
                    font_size: default_font_size,
                    color: default_color.to_string(),
                    font_family: default_font_family.to_string(),
                    anchor,
                });
            }
        }
        LabelConfig::Multi(lines) => {
            let total_lines = lines.len();
            for (i, line) in lines.iter().enumerate() {
                if let Some(text) = interpolate_template(&line.text, properties) {
                    let fs = line.font_size.unwrap_or(default_font_size);
                    let offset = (i as f64 - (total_lines as f64 - 1.0) / 2.0) * fs * LABEL_LINE_SPACING;
                    labels.push(LabelInstance {
                        x: cx,
                        y: cy - offset,
                        text,
                        font_size: fs,
                        color: line
                            .color
                            .as_deref()
                            .unwrap_or(default_color)
                            .to_string(),
                        font_family: line
                            .font_family
                            .as_deref()
                            .unwrap_or(default_font_family)
                            .to_string(),
                        anchor,
                    });
                }
            }
        }
    }
}
