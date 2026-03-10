/// Mercator: Rendering GeoJSON to SVG in a WASM plugin.

mod geometry;
mod projection;
mod style;

use wasm_minimal_protocol::*;

use geojson::GeoJson;
use svg::node::element::{Circle, Definitions, Group, Line, Path, Pattern, Text};
use svg::Document;

use std::collections::HashMap;

use geometry::{
    compute_viewbox, first_altitude, geometry_centroid, render_geometry, PathBuilder, RenderOutput,
};
use projection::{project_geojson, CompiledProjection};
use style::{
    interpolate_template, resolve_style, GraticuleConfig, LabelConfig, LabelInstance,
    ResolvedStyle, StyleConfig,
};

initiate_protocol!();

// Pattern fill constants
const PATTERN_CELL_MULTIPLIER: f64 = 3.0;
const PATTERN_CELL_MIN: f64 = 0.08;
const PATTERN_LINE_WIDTH_RATIO: f64 = 0.3;
const PATTERN_DOT_RADIUS_RATIO: f64 = 0.2;

// Label layout constants
const DEFAULT_LABEL_FONT_SIZE: f64 = 0.3;
const LABEL_LINE_SPACING: f64 = 1.2;

// Viewbox defaults
const DEFAULT_VIEWBOX_PADDING: f64 = 0.1;
const FALLBACK_VIEWBOX: (f64, f64, f64, f64) = (0.0, 0.0, 100.0, 100.0);

// Graticule rendering
const GRATICULE_SEGMENTS: usize = 90;
const GRATICULE_WIDTH_SCALE: f64 = 1000.0;

// SVG rounding precision (3 decimal places)
const SVG_ROUND_FACTOR: f64 = 1000.0;

struct PatternDefs {
    patterns: HashMap<(String, String), String>,
    defs: Definitions,
    counter: usize,
}

impl PatternDefs {
    fn new() -> Self {
        Self {
            patterns: HashMap::new(),
            defs: Definitions::new(),
            counter: 0,
        }
    }

    fn get_or_create(&mut self, pattern_type: &str, color: &str, stroke_width: f64) -> String {
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

    fn has_patterns(&self) -> bool {
        !self.patterns.is_empty()
    }
}

fn add_label(doc: Document, label: &LabelInstance) -> Document {
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

fn render_feature(
    group: Group,
    feat: &geojson::Feature,
    labels: &mut Vec<LabelInstance>,
    config: &StyleConfig,
    patterns: &mut PatternDefs,
    max_gap: f64,
) -> Group {
    let geom = match feat.geometry.as_ref() {
        Some(g) => g,
        None => return group,
    };

    // Build augmented properties including Feature id and coordinate altitude
    let properties = {
        let mut props = feat.properties.clone().unwrap_or_default();
        if let Some(ref id) = feat.id {
            match id {
                geojson::feature::Id::String(s) => {
                    props.entry("id".to_string()).or_insert(serde_json::Value::String(s.clone()));
                }
                geojson::feature::Id::Number(n) => {
                    props.entry("id".to_string()).or_insert(serde_json::Value::Number(n.clone()));
                }
            }
        }
        if let Some(alt) = first_altitude(&geom.value) {
            props.entry("altitude".to_string()).or_insert(
                serde_json::Number::from_f64(alt)
                    .map(serde_json::Value::Number)
                    .unwrap_or(serde_json::Value::Null),
            );
        }
        props
    };
    let properties_ref = Some(&properties);
    let style = resolve_style(config, properties_ref);

    let mut out = RenderOutput { max_gap, ..Default::default() };
    render_geometry(&mut out, &geom.value);
    let group = add_geometry_to_group(group, out, &style, patterns);

    let label_config = match &config.label {
        Some(lc) => lc,
        None => return group,
    };

    let (mut cx, cy) = match geometry_centroid(&geom.value) {
        Some(c) => c,
        None => return group,
    };

    let is_point = matches!(geom.value, geojson::Value::Point(_) | geojson::Value::MultiPoint(_));
    if is_point {
        let r = config.point_radius.unwrap_or(config.stroke_width * 5.0);
        cx += r * 1.5;
    }

    let default_font_size = config.label_font_size.unwrap_or(DEFAULT_LABEL_FONT_SIZE);
    let default_color = config.label_color.as_deref().unwrap_or("black");
    let default_font_family = config.label_font_family.as_deref().unwrap_or("Arial");

    match label_config {
        LabelConfig::Simple(template) => {
            if let Some(text) = interpolate_template(template, &properties) {
                labels.push(LabelInstance {
                    x: cx,
                    y: cy,
                    text,
                    font_size: default_font_size,
                    color: default_color.to_string(),
                    font_family: default_font_family.to_string(),
                    anchor: if is_point { "start" } else { "middle" },
                });
            }
        }
        LabelConfig::Multi(lines) => {
            let total_lines = lines.len();
            for (i, line) in lines.iter().enumerate() {
                if let Some(text) = interpolate_template(&line.text, &properties) {
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
                        anchor: if is_point { "start" } else { "middle" },
                    });
                }
            }
        }
    }

    group
}

fn resolve_fill(style: &ResolvedStyle, patterns: &mut PatternDefs) -> String {
    if let Some(ref pat) = style.fill_pattern {
        let id = patterns.get_or_create(pat, &style.fill, style.stroke_width);
        format!("url(#{})", id)
    } else {
        style.fill.clone()
    }
}

fn add_geometry_to_group(
    mut group: Group,
    out: RenderOutput,
    style: &ResolvedStyle,
    patterns: &mut PatternDefs,
) -> Group {
    if !out.polygon_data.is_empty() {
        let fill = resolve_fill(style, patterns);
        group = group.add(
            Path::new()
                .set("fill", &*fill)
                .set("fill-opacity", style.fill_opacity)
                .set("stroke", &*style.stroke)
                .set("stroke-width", style.stroke_width)
                .set("d", out.polygon_data),
        );
    }

    if !out.line_data.is_empty() {
        group = group.add(
            Path::new()
                .set("fill", "none")
                .set("stroke", &*style.stroke)
                .set("stroke-width", style.stroke_width)
                .set("d", out.line_data),
        );
    }

    let point_fill = style.point_color.as_deref().unwrap_or(&style.fill);
    if point_fill != "none" {
        for (x, y) in &out.points {
            if x.is_nan() || y.is_nan() {
                continue;
            }
            group = group.add(
                Circle::new()
                    .set("cx", *x)
                    .set("cy", *y)
                    .set("r", style.point_radius)
                    .set("fill", point_fill)
                    .set("fill-opacity", style.fill_opacity)
                    .set("stroke", &*style.stroke)
                    .set("stroke-width", style.stroke_width),
            );
        }
    }

    group
}

fn try_topojson(content: &str) -> Result<GeoJson, String> {
    let topo: topojson::Topology =
        serde_json::from_str(content).map_err(|e| format!("Not valid GeoJSON or TopoJSON: {e}"))?;

    // Pick the object with the most geometries to avoid merging
    // duplicate layers (e.g. "countries" + "land" in world-atlas files).
    let best = topo
        .objects
        .iter()
        .max_by_key(|ng| match &ng.geometry.value {
            topojson::Value::GeometryCollection(geoms) => geoms.len(),
            _ => 1,
        })
        .ok_or_else(|| "TopoJSON has no named objects".to_string())?;

    let fc = topojson::to_geojson(&topo, &best.name).map_err(|e| e.to_string())?;
    Ok(GeoJson::FeatureCollection(geojson::FeatureCollection {
        bbox: None,
        features: fc.features,
        foreign_members: None,
    }))
}

fn render_graticule(proj: &CompiledProjection, grat: &GraticuleConfig, max_gap: f64) -> Group {
    let mut group = Group::new();
    let step = grat.step;
    let segments = GRATICULE_SEGMENTS;

    let make_path = |data: svg::node::element::path::Data| -> Path {
        Path::new()
            .set("fill", "none")
            .set("stroke", &*grat.color)
            .set("stroke-width", grat.width)
            .set("stroke-opacity", grat.opacity)
            .set("d", data)
    };

    // Meridians (longitude lines)
    let mut lon = -180.0;
    while lon <= 180.0 {
        let mut pb = PathBuilder::new(svg::node::element::path::Data::new(), max_gap);
        for i in 0..=segments {
            let lat = -90.0 + (180.0 * i as f64 / segments as f64);
            let (x, y) = proj.project(lon, lat);
            pb.add(x, y);
        }
        group = group.add(make_path(pb.finish()));
        lon += step;
    }

    // Parallels (latitude lines)
    let mut lat = -90.0;
    while lat <= 90.0 {
        let mut pb = PathBuilder::new(svg::node::element::path::Data::new(), max_gap);
        for i in 0..=segments * 2 {
            let lng = -180.0 + (360.0 * i as f64 / (segments * 2) as f64);
            let (x, y) = proj.project(lng, lat);
            pb.add(x, y);
        }
        group = group.add(make_path(pb.finish()));
        lat += step;
    }

    group
}

#[wasm_func]
pub fn geo(geojson: &[u8], config: &[u8]) -> Result<Vec<u8>, String> {
    let mut conf: StyleConfig = {
        match serde_json::from_slice(config) {
            Ok(conf) => conf,
            _ => StyleConfig::default(),
        }
    };

    let content = String::from_utf8(geojson.to_vec()).map_err(|e| e.to_string())?;
    let mut geojson = match content.parse::<GeoJson>() {
        Ok(gj) => gj,
        Err(_) => try_topojson(&content)?,
    };

    let proj = CompiledProjection::from_config(conf.projection.take());
    project_geojson(&mut geojson, &proj);

    if conf.viewbox.is_none() {
        let padding = conf.viewbox_padding.unwrap_or(DEFAULT_VIEWBOX_PADDING);
        conf.viewbox = Some(compute_viewbox(&geojson, padding));
    }

    let viewbox = conf.viewbox.unwrap_or(FALLBACK_VIEWBOX);
    let max_gap = proj.antimeridian_gap();
    let mut doc = Document::new().set("viewBox", viewbox);
    let mut group = Group::new();
    let mut labels: Vec<LabelInstance> = Vec::new();
    let mut patterns = PatternDefs::new();

    match geojson {
        GeoJson::FeatureCollection(fc) => {
            for feat in &fc.features {
                group = render_feature(group, feat, &mut labels, &conf, &mut patterns, max_gap);
            }
        }
        GeoJson::Feature(ref feat) => {
            group = render_feature(group, feat, &mut labels, &conf, &mut patterns, max_gap);
        }
        GeoJson::Geometry(ref geom) => {
            let style = resolve_style(&conf, None);
            let mut out = RenderOutput { max_gap, ..Default::default() };
            render_geometry(&mut out, &geom.value);
            group = add_geometry_to_group(group, out, &style, &mut patterns);
        }
    }

    // Add defs before geometry so pattern references resolve correctly
    if patterns.has_patterns() {
        doc = doc.add(patterns.defs);
    }

    // Graticule behind geometry
    if let Some(ref grat) = conf.graticule {
        let vb_diag = (viewbox.2.powi(2) + viewbox.3.powi(2)).sqrt();
        let scaled_width = grat.width * vb_diag / GRATICULE_WIDTH_SCALE;
        let scaled_grat = GraticuleConfig {
            step: grat.step,
            color: grat.color.clone(),
            width: scaled_width,
            opacity: grat.opacity,
        };
        doc = doc.add(render_graticule(&proj, &scaled_grat, max_gap));
    }

    doc = doc.add(group);

    for label in &labels {
        doc = add_label(doc, label);
    }
    let mut buf = Vec::new();
    svg::write(&mut buf, &doc).map_err(|e| e.to_string())?;
    Ok(buf)
}
