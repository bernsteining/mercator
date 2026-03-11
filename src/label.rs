use svg::node::element::Text;
use svg::Document;

use crate::style::{
    interpolate_template, LabelConfig, LabelInstance, StyleConfig,
};

const DEFAULT_LABEL_FONT_SIZE: f64 = 0.3;
const LABEL_LINE_SPACING: f64 = 1.2;

pub fn add_label(doc: Document, label: &LabelInstance) -> Document {
    doc.add(
        Text::new(&*label.text)
            .set("x", label.x)
            .set("y", label.y)
            .set("font-size", label.font_size)
            .set("font-family", &*label.font_family)
            .set("fill", &*label.color)
            .set("text-anchor", label.anchor)
            .set("dominant-baseline", "middle"),
    )
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
