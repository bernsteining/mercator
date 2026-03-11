use std::collections::HashMap;
use svg::node::element::{Circle, Definitions, Line, Pattern};

use crate::style::ResolvedStyle;

const PATTERN_CELL_MULTIPLIER: f64 = 3.0;
const PATTERN_CELL_MIN: f64 = 0.08;
const PATTERN_LINE_WIDTH_RATIO: f64 = 0.3;
const PATTERN_DOT_RADIUS_RATIO: f64 = 0.2;
const SVG_ROUND_FACTOR: f64 = 1000.0;

pub struct PatternDefs {
    patterns: HashMap<(String, String), String>,
    pub defs: Definitions,
    counter: usize,
}

impl PatternDefs {
    pub fn new() -> Self {
        Self {
            patterns: HashMap::new(),
            defs: Definitions::new(),
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

        let pattern = match pattern_type {
            "hatched" => Pattern::new()
                .set("id", &*id)
                .set("patternUnits", "userSpaceOnUse")
                .set("width", cell)
                .set("height", cell)
                .set("patternTransform", "rotate(45)")
                .add(
                    Line::new()
                        .set("x1", 0)
                        .set("y1", 0)
                        .set("x2", 0)
                        .set("y2", cell)
                        .set("stroke", color)
                        .set("stroke-width", line_w),
                ),
            "crosshatched" => Pattern::new()
                .set("id", &*id)
                .set("patternUnits", "userSpaceOnUse")
                .set("width", cell)
                .set("height", cell)
                .set("patternTransform", "rotate(45)")
                .add(
                    Line::new()
                        .set("x1", 0)
                        .set("y1", 0)
                        .set("x2", 0)
                        .set("y2", cell)
                        .set("stroke", color)
                        .set("stroke-width", line_w),
                )
                .add(
                    Line::new()
                        .set("x1", 0)
                        .set("y1", 0)
                        .set("x2", cell)
                        .set("y2", 0)
                        .set("stroke", color)
                        .set("stroke-width", line_w),
                ),
            "dotted" => {
                let r = cell * PATTERN_DOT_RADIUS_RATIO;
                Pattern::new()
                    .set("id", &*id)
                    .set("patternUnits", "userSpaceOnUse")
                    .set("width", cell)
                    .set("height", cell)
                    .add(
                        Circle::new()
                            .set("cx", cell / 2.0)
                            .set("cy", cell / 2.0)
                            .set("r", r)
                            .set("fill", color),
                    )
            }
            _ => Pattern::new()
                .set("id", &*id)
                .set("patternUnits", "userSpaceOnUse")
                .set("width", cell)
                .set("height", cell),
        };

        self.defs = std::mem::replace(&mut self.defs, Definitions::new()).add(pattern);
        self.patterns.insert(key, id.clone());
        id
    }

    pub fn has_patterns(&self) -> bool {
        !self.patterns.is_empty()
    }
}

pub fn resolve_fill(style: &ResolvedStyle, patterns: &mut PatternDefs) -> String {
    if let Some(ref pat) = style.fill_pattern {
        let id = patterns.get_or_create(pat, &style.fill, style.stroke_width);
        format!("url(#{})", id)
    } else {
        style.fill.clone()
    }
}
