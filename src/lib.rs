/// Mercator: Rendering GeoJSON to SVG in a WASM plugin.

mod clip;
mod clip_antimeridian;
mod geometry;
mod graticule;
mod label;
mod model;
mod pattern;
mod projection;
mod scale;
mod style;
mod tissot;
mod topojson;

use wasm_minimal_protocol::*;

use std::cell::RefCell;
use std::collections::HashMap;

use model::{Feature, FeatureId, GeoJson, Geometry};
use scale::{render_legend, ColorScale, SizeScale};

use clip::{Clip, ClipCircle};
use geometry::{push_f64, render_geometry, BoundsAccumulator, Centroid, RenderOutput};
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
        let parsed = match model::parse(data) {
            Ok(gj) => gj,
            Err(_) => {
                let content = std::str::from_utf8(data).map_err(|e| e.to_string())?;
                topojson::try_topojson(content)?
            }
        };
        cache.borrow_mut().insert(key, parsed);
        let map = cache.borrow();
        Ok(f(map.get(&key).unwrap()))
    })
}

const DEFAULT_VIEWBOX_PADDING: f64 = 0.15;
const GRATICULE_WIDTH_SCALE: f64 = 1000.0;

#[allow(clippy::too_many_arguments)]
fn render_feature(
    svg: &mut String,
    feat: &Feature,
    labels: &mut Vec<LabelInstance>,
    config: &StyleConfig,
    scale: Option<&ColorScale>,
    size_scale: Option<&SizeScale>,
    clip: Option<&Clip>,
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

    let owned_props;
    let properties = if feat.id.is_some() {
        let mut props = base_props.clone();
        match feat.id.as_ref() {
            Some(FeatureId::String(s)) => {
                props.entry("id".to_string()).or_insert(serde_json::Value::String(s.clone()));
            }
            Some(FeatureId::Number(n)) => {
                props.entry("id".to_string()).or_insert(serde_json::Value::Number(n.clone()));
            }
            None => {}
        }
        owned_props = props;
        &owned_props
    } else {
        base_props
    };
    let mut style = resolve_style(config, Some(properties));
    // A data-driven fill_scale overrides the (possibly templated) solid fill.
    if let Some(scale) = scale {
        style.fill = std::borrow::Cow::Owned(scale.color_for(properties));
    }
    // A point_radius_scale sizes this feature's symbols by a numeric property.
    if let Some(size_scale) = size_scale {
        style.point_radius = size_scale.radius_for(properties);
    }

    // The centroid is only used to place labels; skip accumulating it otherwise.
    let track_centroid = config.label.is_some();

    out.clear();
    let mut centroid = Centroid::new();
    render_geometry(out, geom, proj, bounds, &mut centroid, track_centroid, clip);
    write_geometry(svg, out, &style, patterns);

    if let Some((mut cx, cy)) = centroid.get() {
        let is_point = matches!(geom, Geometry::Point(_) | Geometry::MultiPoint(_));
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
            write_marker(svg, &style.point_shape, *x, *y, style.point_radius, point_fill, style.fill_opacity, &style.stroke, style.stroke_width);
        }
    }
}

/// Emit one point marker of the given `shape` centered at `(x, y)` with size `r`.
/// Non-circle shapes are drawn as a `<path>` inscribed in the radius-`r` circle.
#[allow(clippy::too_many_arguments)]
fn write_marker(svg: &mut String, shape: &str, x: f64, y: f64, r: f64, fill: &str, fill_opacity: f64, stroke: &str, stroke_width: f64) {
    let common = |svg: &mut String| {
        svg.push_str(r#" fill=""#);
        svg.push_str(fill);
        svg.push_str(r#"" fill-opacity=""#);
        push_f64(svg, fill_opacity);
        svg.push_str(r#"" stroke=""#);
        svg.push_str(stroke);
        svg.push_str(r#"" stroke-width=""#);
        push_f64(svg, stroke_width);
        svg.push_str(r#""/>"#);
    };
    // Vertex list for polygon markers (fraction of r, relative to center).
    let poly: Option<&[(f64, f64)]> = match shape {
        "square" => Some(&[(-0.886, -0.886), (0.886, -0.886), (0.886, 0.886), (-0.886, 0.886)]),
        "diamond" => Some(&[(0.0, -1.0), (1.0, 0.0), (0.0, 1.0), (-1.0, 0.0)]),
        "triangle" => Some(&[(0.0, -1.0), (0.866, 0.5), (-0.866, 0.5)]),
        "cross" | "plus" => Some(&[
            (-0.33, -1.0), (0.33, -1.0), (0.33, -0.33), (1.0, -0.33), (1.0, 0.33),
            (0.33, 0.33), (0.33, 1.0), (-0.33, 1.0), (-0.33, 0.33), (-1.0, 0.33),
            (-1.0, -0.33), (-0.33, -0.33),
        ]),
        "star" => Some(&STAR),
        _ => None, // circle
    };
    match poly {
        None => {
            svg.push_str(r#"<circle cx=""#);
            push_f64(svg, x);
            svg.push_str(r#"" cy=""#);
            push_f64(svg, y);
            svg.push_str(r#"" r=""#);
            push_f64(svg, r);
            common(svg);
        }
        Some(verts) => {
            svg.push_str(r#"<path d=""#);
            for (i, (dx, dy)) in verts.iter().enumerate() {
                svg.push(if i == 0 { 'M' } else { 'L' });
                push_f64(svg, x + dx * r);
                svg.push(' ');
                push_f64(svg, y + dy * r);
                svg.push(' ');
            }
            svg.push('Z');
            svg.push('"');
            common(svg);
        }
    }
}

/// 5-pointed star vertices (outer radius 1, inner 0.5), pointing up.
static STAR: [(f64, f64); 10] = {
    // Precomputed cos/sin at 36° steps from the top (-90°), alternating radii.
    [
        (0.0, -1.0), (0.2939, -0.4045), (0.9511, -0.3090), (0.4755, 0.1545),
        (0.5878, 0.8090), (0.0, 0.5), (-0.5878, 0.8090), (-0.4755, 0.1545),
        (-0.9511, -0.3090), (-0.2939, -0.4045),
    ]
};

#[wasm_func]
pub fn geo(geojson: &[u8], config: &[u8]) -> Result<Vec<u8>, String> {
    // An empty config means "use all defaults"; anything else must parse cleanly.
    // Surfacing the error (rather than silently falling back to defaults) makes a
    // typo in one field visible instead of quietly wiping every style.
    let mut conf: StyleConfig = if config.is_empty() {
        StyleConfig::default()
    } else {
        serde_json::from_slice(config).map_err(|e| format!("invalid config: {e}"))?
    };

    let proj = projection::from_config(conf.projection.take());
    let max_gap = proj.antimeridian_gap();

    // Azimuthal (globe) projections always clip geometry to the visible hemisphere
    // — that's a correctness fix, independent of styling. The optional `sphere`
    // config only adds the ocean disc behind the land. For cylindrical projections,
    // `antimeridian: true` opts into antimeridian clipping.
    let clip: Option<Clip> = if let Some(c) = proj.clip_center() {
        Some(Clip::Circle(ClipCircle::new(c)))
    } else if conf.antimeridian {
        proj.antimeridian_center()
            .map(|central_meridian| Clip::Antimeridian { central_meridian })
    } else {
        None
    };

    let mut labels: Vec<LabelInstance> = Vec::new();
    let mut patterns = PatternDefs::new();
    let mut geo_buf = String::new();
    let mut bounds = BoundsAccumulator::new();
    let mut out = RenderOutput { max_gap, ..Default::default() };

    // Single pass: render features with on-the-fly projection, accumulating bounds + centroids.
    // The color scale (choropleth) is built first — it may pre-scan properties for its domain —
    // then returned so the legend can be drawn once the viewbox is known.
    let scale = with_parsed_geojson(geojson, |geojson| {
        let scale = conf
            .fill_scale
            .as_ref()
            .and_then(|fs| ColorScale::build(fs, geojson));
        let size = conf
            .point_radius_scale
            .as_ref()
            .and_then(|rs| SizeScale::build(rs, geojson));
        match geojson {
            GeoJson::FeatureCollection(features) => {
                for feat in features {
                    render_feature(&mut geo_buf, feat, &mut labels, &conf, scale.as_ref(), size.as_ref(), clip.as_ref(), &mut patterns, &mut out, &proj, &mut bounds);
                }
            }
            GeoJson::Feature(feat) => {
                render_feature(&mut geo_buf, feat, &mut labels, &conf, scale.as_ref(), size.as_ref(), clip.as_ref(), &mut patterns, &mut out, &proj, &mut bounds);
            }
            GeoJson::Geometry(geom) => {
                let style = resolve_style(&conf, None);
                out.clear();
                let mut centroid = Centroid::new();
                render_geometry(&mut out, geom, &proj, &mut bounds, &mut centroid, false, clip.as_ref());
                write_geometry(&mut geo_buf, &out, &style, &mut patterns);
            }
        }
        scale
    })?;

    // When an ocean disc is drawn, include it in the auto viewbox so it isn't
    // clipped. Without a disc the viewbox follows the (already limb-clipped)
    // geometry, so a partial-globe map isn't padded out to the full hemisphere.
    if let (Some(Clip::Circle(circle)), Some(_)) = (&clip, &conf.sphere) {
        let ((cx, cy), r) = circle.disc(&proj);
        bounds.add(cx - r, cy - r);
        bounds.add(cx + r, cy + r);
    }

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

    // Sphere (ocean) disc, behind graticule and geometry.
    if let (Some(Clip::Circle(circle)), Some(sphere)) = (&clip, &conf.sphere) {
        let ((cx, cy), r) = circle.disc(&proj);
        svg.push_str(r#"<circle cx=""#);
        push_f64(&mut svg, cx);
        svg.push_str(r#"" cy=""#);
        push_f64(&mut svg, cy);
        svg.push_str(r#"" r=""#);
        push_f64(&mut svg, r);
        svg.push_str(r#"" fill=""#);
        svg.push_str(&sphere.fill);
        if let Some(ref stroke) = sphere.stroke {
            svg.push_str(r#"" stroke=""#);
            svg.push_str(stroke);
            svg.push_str(r#"" stroke-width=""#);
            push_f64(&mut svg, sphere.stroke_width);
        }
        svg.push_str(r#""/>"#);
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

    // Legend (choropleth key), drawn last so it sits on top.
    if let (Some(scale), Some(legend)) = (scale.as_ref(), conf.legend.as_ref()) {
        render_legend(&mut svg, scale, legend, viewbox);
    }

    svg.push_str("</svg>");
    Ok(svg.into_bytes())
}
