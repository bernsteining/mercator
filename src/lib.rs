/// Mercator: Rendering GeoJSON to SVG in a WASM plugin.

mod geometry;
mod graticule;
mod label;
mod pattern;
mod projection;
mod style;
mod tissot;
mod topojson_convert;

use wasm_minimal_protocol::*;

use geojson::GeoJson;
use std::cell::RefCell;
use std::collections::HashMap;

use geometry::{
    first_altitude, push_f64, render_geometry, BoundsAccumulator, Centroid, RenderOutput,
};
use label::{build_labels, write_label};
use pattern::{write_fill, PatternDefs};
use projection::Proj;
use style::{resolve_style, GraticuleConfig, LabelInstance, ResolvedStyle, StyleConfig};

initiate_protocol!();

thread_local! {
    static GEOJSON_CACHE: RefCell<HashMap<u64, GeoJson>> = RefCell::new(HashMap::new());
}

fn hash_bytes(data: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for &b in data {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

fn with_parsed_geojson<R>(data: &[u8], f: impl FnOnce(&GeoJson) -> R) -> Result<R, String> {
    let key = hash_bytes(data);
    GEOJSON_CACHE.with(|cache| {
        {
            let map = cache.borrow();
            if let Some(cached) = map.get(&key) {
                return Ok(f(cached));
            }
        }
        let content = std::str::from_utf8(data).map_err(|e| e.to_string())?;
        let parsed = match content.parse::<GeoJson>() {
            Ok(gj) => gj,
            Err(_) => topojson_convert::try_topojson(content)?,
        };
        cache.borrow_mut().insert(key, parsed);
        let map = cache.borrow();
        Ok(f(map.get(&key).unwrap()))
    })
}

const DEFAULT_VIEWBOX_PADDING: f64 = 0.15;
const GRATICULE_WIDTH_SCALE: f64 = 1000.0;

fn render_feature(
    svg: &mut String,
    feat: &geojson::Feature,
    labels: &mut Vec<LabelInstance>,
    config: &StyleConfig,
    patterns: &mut PatternDefs,
    out: &mut RenderOutput,
    proj: &Proj,
    bounds: &mut BoundsAccumulator,
) {
    let geom = match feat.geometry.as_ref() {
        Some(g) => g,
        None => return,
    };

    let empty = serde_json::Map::new();
    let base_props = feat.properties.as_ref().unwrap_or(&empty);
    let altitude = first_altitude(&geom.value);
    let needs_clone = feat.id.is_some() || altitude.is_some();

    let owned_props;
    let properties = if needs_clone {
        let mut props = base_props.clone();
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
        if let Some(alt) = altitude {
            props.entry("altitude".to_string()).or_insert(
                serde_json::Number::from_f64(alt)
                    .map(serde_json::Value::Number)
                    .unwrap_or(serde_json::Value::Null),
            );
        }
        owned_props = props;
        &owned_props
    } else {
        base_props
    };
    let style = resolve_style(config, Some(properties));

    out.clear();
    let mut centroid = Centroid::new();
    render_geometry(out, &geom.value, proj, bounds, &mut centroid);
    write_geometry(svg, out, &style, patterns);

    if let Some((mut cx, cy)) = centroid.get() {
        let is_point = matches!(geom.value, geojson::Value::Point(_) | geojson::Value::MultiPoint(_));
        if is_point {
            let r = config.point_radius.unwrap_or(config.stroke_width * 5.0);
            cx += r * 1.5;
        }
        build_labels(labels, config, &properties, cx, cy, is_point);
    }
}

fn write_geometry(
    svg: &mut String,
    out: &RenderOutput,
    style: &ResolvedStyle<'_>,
    patterns: &mut PatternDefs,
) {
    if !out.polygon_data.is_empty() {
        svg.push_str(r#"<path fill=""#);
        write_fill(svg, style, patterns);
        svg.push_str(r#"" fill-opacity=""#);
        push_f64(svg, style.fill_opacity);
        svg.push_str(r#"" stroke=""#);
        svg.push_str(&style.stroke);
        svg.push_str(r#"" stroke-width=""#);
        push_f64(svg, style.stroke_width);
        svg.push_str(r#"" d=""#);
        svg.push_str(&out.polygon_data);
        svg.push_str(r#""/>"#);
    }

    if !out.line_data.is_empty() {
        svg.push_str(r#"<path fill="none" stroke=""#);
        svg.push_str(&style.stroke);
        svg.push_str(r#"" stroke-width=""#);
        push_f64(svg, style.stroke_width);
        svg.push_str(r#"" d=""#);
        svg.push_str(&out.line_data);
        svg.push_str(r#""/>"#);
    }

    let point_fill = style.point_color.as_deref().unwrap_or(&style.fill);
    if point_fill != "none" {
        for (x, y) in &out.points {
            if x.is_nan() || y.is_nan() {
                continue;
            }
            svg.push_str(r#"<circle cx=""#);
            push_f64(svg, *x);
            svg.push_str(r#"" cy=""#);
            push_f64(svg, *y);
            svg.push_str(r#"" r=""#);
            push_f64(svg, style.point_radius);
            svg.push_str(r#"" fill=""#);
            svg.push_str(point_fill);
            svg.push_str(r#"" fill-opacity=""#);
            push_f64(svg, style.fill_opacity);
            svg.push_str(r#"" stroke=""#);
            svg.push_str(&style.stroke);
            svg.push_str(r#"" stroke-width=""#);
            push_f64(svg, style.stroke_width);
            svg.push_str(r#""/>"#);
        }
    }
}

#[wasm_func]
pub fn geo(geojson: &[u8], config: &[u8]) -> Result<Vec<u8>, String> {
    let mut conf: StyleConfig = serde_json::from_slice(config).unwrap_or_default();

    let proj = projection::from_config(conf.projection.take());
    let max_gap = proj.antimeridian_gap();

    let mut labels: Vec<LabelInstance> = Vec::new();
    let mut patterns = PatternDefs::new();
    let mut geo_buf = String::new();
    let mut bounds = BoundsAccumulator::new();
    let mut out = RenderOutput { max_gap, ..Default::default() };

    // Single pass: render features with on-the-fly projection, accumulating bounds + centroids
    with_parsed_geojson(geojson, |geojson| {
        match geojson {
            GeoJson::FeatureCollection(fc) => {
                for feat in &fc.features {
                    render_feature(&mut geo_buf, feat, &mut labels, &conf, &mut patterns, &mut out, &proj, &mut bounds);
                }
            }
            GeoJson::Feature(ref feat) => {
                render_feature(&mut geo_buf, feat, &mut labels, &conf, &mut patterns, &mut out, &proj, &mut bounds);
            }
            GeoJson::Geometry(ref geom) => {
                let style = resolve_style(&conf, None);
                out.clear();
                let mut centroid = Centroid::new();
                render_geometry(&mut out, &geom.value, &proj, &mut bounds, &mut centroid);
                write_geometry(&mut geo_buf, &out, &style, &mut patterns);
            }
        }
    })?;

    let viewbox = conf.viewbox.unwrap_or_else(|| {
        let padding = conf.viewbox_padding.unwrap_or(DEFAULT_VIEWBOX_PADDING);
        bounds.viewbox(padding)
    });

    let mut svg = String::with_capacity(32768);
    svg.push_str(r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox=""#);
    push_f64(&mut svg, viewbox.0);
    svg.push(' ');
    push_f64(&mut svg, viewbox.1);
    svg.push(' ');
    push_f64(&mut svg, viewbox.2);
    svg.push(' ');
    push_f64(&mut svg, viewbox.3);
    svg.push_str(r#"">"#);

    // Defs (patterns) must come before geometry references
    if patterns.has_patterns() {
        patterns.write_defs(&mut svg);
    }

    // Graticule
    if let Some(ref grat) = conf.graticule {
        let vb_diag = (viewbox.2.powi(2) + viewbox.3.powi(2)).sqrt();
        let scaled_grat = GraticuleConfig {
            step: grat.step,
            color: grat.color.clone(),
            width: grat.width * vb_diag / GRATICULE_WIDTH_SCALE,
            opacity: grat.opacity,
        };
        svg.push_str(&graticule::render(&proj, &scaled_grat, max_gap));
    }

    // Feature geometry
    svg.push_str(&geo_buf);

    // Tissot indicatrices
    if let Some(ref tis) = conf.tissot {
        let vb_diag = (viewbox.2.powi(2) + viewbox.3.powi(2)).sqrt();
        let scaled = style::TissotConfig {
            step: tis.step,
            radius: tis.radius,
            fill: tis.fill.clone(),
            fill_opacity: tis.fill_opacity,
            stroke: tis.stroke.clone(),
            stroke_width: tis.stroke_width * vb_diag / GRATICULE_WIDTH_SCALE,
            max_lat: tis.max_lat,
        };
        svg.push_str(&tissot::render(&proj, &scaled, max_gap));
    }

    // Labels
    for label in &labels {
        write_label(&mut svg, label);
    }

    svg.push_str("</svg>");
    Ok(svg.into_bytes())
}
