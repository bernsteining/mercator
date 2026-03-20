use std::collections::HashMap;

use crate::geometry::push_f64;
use crate::style::ResolvedStyle;

const PATTERN_CELL_MULTIPLIER: f64 = 3.0;
const PATTERN_CELL_MIN: f64 = 0.08;
const PATTERN_LINE_WIDTH_RATIO: f64 = 0.3;
const PATTERN_DOT_RADIUS_RATIO: f64 = 0.2;
const SVG_ROUND_FACTOR: f64 = 1000.0;

pub struct PatternDefs {
    patterns: HashMap<(String, String), String>,
    buf: String,
    counter: usize,
}

impl PatternDefs {
    pub fn new() -> Self {
        Self {
            patterns: HashMap::new(),
            buf: String::new(),
            counter: 0,
        }
    }

    pub fn get_or_create(&mut self, pattern_type: &str, color: &str, stroke_width: f64) -> String {
        let key = (pattern_type.to_string(), color.to_string());
        if let Some(id) = self.patterns.get(&key) {
            return id.clone();
        }

        let id = format!("pat-{}", self.counter);
        self.counter += 1;

        let cell = ((stroke_width * PATTERN_CELL_MULTIPLIER).max(PATTERN_CELL_MIN) * SVG_ROUND_FACTOR).round() / SVG_ROUND_FACTOR;
        let line_w = (cell * PATTERN_LINE_WIDTH_RATIO * SVG_ROUND_FACTOR).round() / SVG_ROUND_FACTOR;

        // Helper: write pattern header
        let buf = &mut self.buf;
        buf.push_str(r#"<pattern id=""#);
        buf.push_str(&id);
        buf.push_str(r#"" patternUnits="userSpaceOnUse" width=""#);
        push_f64(buf, cell);
        buf.push_str(r#"" height=""#);
        push_f64(buf, cell);

        match pattern_type {
            "hatched" => {
                buf.push_str(r#"" patternTransform="rotate(45)"><line x1="0" y1="0" x2="0" y2=""#);
                push_f64(buf, cell);
                buf.push_str(r#"" stroke=""#);
                buf.push_str(color);
                buf.push_str(r#"" stroke-width=""#);
                push_f64(buf, line_w);
                buf.push_str(r#""/></pattern>"#);
            }
            "crosshatched" => {
                buf.push_str(r#"" patternTransform="rotate(45)"><line x1="0" y1="0" x2="0" y2=""#);
                push_f64(buf, cell);
                buf.push_str(r#"" stroke=""#);
                buf.push_str(color);
                buf.push_str(r#"" stroke-width=""#);
                push_f64(buf, line_w);
                buf.push_str(r#""/><line x1="0" y1="0" x2=""#);
                push_f64(buf, cell);
                buf.push_str(r#"" y2="0" stroke=""#);
                buf.push_str(color);
                buf.push_str(r#"" stroke-width=""#);
                push_f64(buf, line_w);
                buf.push_str(r#""/></pattern>"#);
            }
            "dotted" => {
                let r = cell * PATTERN_DOT_RADIUS_RATIO;
                let half = cell / 2.0;
                buf.push_str(r#""><circle cx=""#);
                push_f64(buf, half);
                buf.push_str(r#"" cy=""#);
                push_f64(buf, half);
                buf.push_str(r#"" r=""#);
                push_f64(buf, r);
                buf.push_str(r#"" fill=""#);
                buf.push_str(color);
                buf.push_str(r#""/></pattern>"#);
            }
            _ => {
                buf.push_str(r#""/>"#);
            }
        }

        self.patterns.insert(key, id.clone());
        id
    }

    pub fn has_patterns(&self) -> bool {
        !self.buf.is_empty()
    }

    pub fn write_defs(&self, out: &mut String) {
        out.push_str("<defs>");
        out.push_str(&self.buf);
        out.push_str("</defs>");
    }
}

/// Write the fill attribute value directly to the SVG string (no intermediate allocation).
pub fn write_fill(svg: &mut String, style: &ResolvedStyle<'_>, patterns: &mut PatternDefs) {
    if let Some(ref pat) = style.fill_pattern {
        let id = patterns.get_or_create(pat, &style.fill, style.stroke_width);
        svg.push_str("url(#");
        svg.push_str(&id);
        svg.push(')');
    } else {
        svg.push_str(&style.fill);
    }
}
