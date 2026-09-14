/// Mercator: Rendering GeoJSON to SVG in a WASM plugin.

mod clip;
mod clip_antimeridian;
mod contour;
mod geometry;
mod graticule;
mod hexbin;
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
use scale::{render_legend, render_size_legend, scheme_colors, ColorScale, SizeScale};

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
    hex_points: Option<&mut Vec<(f64, f64)>>,
    dorling_out: Option<&mut Vec<(f64, f64, f64, String)>>,
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
    // Skip features that don't match the filter.
    if let Some(filter) = &config.filter {
        if !filter.matches(properties) {
            return;
        }
    }

    let mut style = resolve_style(config, Some(properties));
    // A data-driven fill_scale overrides the (possibly templated) solid fill.
    if let Some(scale) = scale {
        style.fill = std::borrow::Cow::Owned(scale.color_for(properties));
    }
    // A point_radius_scale sizes this feature's symbols by a numeric property.
    if let Some(size_scale) = size_scale {
        style.point_radius = size_scale.radius_for(properties);
    }

    // The centroid places labels and Dorling circles; skip it otherwise.
    let track_centroid = config.label.is_some() || config.dorling.is_some();

    out.clear();
    let mut centroid = Centroid::new();
    render_geometry(out, geom, proj, bounds, &mut centroid, track_centroid, clip);
    if let Some(dl) = dorling_out {
        // Dorling: collect a circle at the centroid; the geometry itself isn't drawn.
        if let (Some(dcfg), Some((cx, cy))) = (&config.dorling, centroid.get()) {
            let v = properties.get(&dcfg.property).and_then(serde_json::Value::as_f64).unwrap_or(0.0);
            dl.push((cx, cy, v, style.fill.to_string()));
        }
    } else {
        // With hexbin on, points are aggregated into the grid rather than drawn.
        write_geometry(svg, out, &style, patterns, hex_points.is_none());
        if let Some(hp) = hex_points {
            hp.extend(out.points.iter().copied());
        }
    }

    if let Some((mut cx, cy)) = centroid.get() {
        let is_point = matches!(geom, Geometry::Point(_) | Geometry::MultiPoint(_));
        if is_point {
            let r = config.point_radius.unwrap_or(config.stroke_width.resolve(Some(properties)) * 5.0);
            cx += r * 1.5;
        }
        build_labels(labels, config, &properties, cx, cy, is_point);
    }
}

/// Append a `stroke-dasharray` attribute fragment when the style sets a dash.
fn write_dash(svg: &mut String, style: &ResolvedStyle<'_>) {
    if let Some(dash) = &style.stroke_dash {
        if !dash.is_empty() {
            svg.push_str(r#"" stroke-dasharray=""#);
            svg.push_str(dash);
        }
    }
}

fn write_geometry(
    svg: &mut String,
    out: &RenderOutput,
    style: &ResolvedStyle<'_>,
    patterns: &mut PatternDefs,
    draw_points: bool,
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
        write_dash(svg, style);
        svg.push_str(r#"" d=""#);
        svg.push_str(&out.polygon_data);
        svg.push_str(r#""/>"#);
    }

    if !out.line_data.is_empty() {
        svg.push_str(r#"<path fill="none" stroke=""#);
        svg.push_str(&style.stroke);
        svg.push_str(r#"" stroke-width=""#);
        push_f64(svg, style.stroke_width);
        write_dash(svg, style);
        svg.push_str(r#"" d=""#);
        svg.push_str(&out.line_data);
        svg.push_str(r#""/>"#);
    }

    let point_fill = style.point_color.as_deref().unwrap_or(&style.fill);
    if draw_points && point_fill != "none" {
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
            svg.push('"');
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

/// Lay out a Dorling cartogram: size each collected `(cx, cy, value, fill)` circle
/// (√-scaled so area ∝ value), then relax overlaps with gravity toward the
/// original centroid + pairwise repulsion. Returns final `(x, y, r, fill)`.
fn compute_dorling(pts: Vec<(f64, f64, f64, String)>, dc: &style::DorlingConfig) -> Vec<(f64, f64, f64, String)> {
    let n = pts.len();
    if n == 0 {
        return Vec::new();
    }
    let (lo, hi) = dc.domain.unwrap_or_else(|| {
        let mut lo = f64::INFINITY;
        let mut hi = f64::NEG_INFINITY;
        for p in &pts {
            lo = lo.min(p.2);
            hi = hi.max(p.2);
        }
        (lo, hi)
    });
    let radius = |v: f64| {
        let t = if hi > lo { ((v - lo) / (hi - lo)).clamp(0.0, 1.0) } else { 0.0 };
        dc.min_radius + (dc.max_radius - dc.min_radius) * t.sqrt()
    };
    let mut x: Vec<f64> = pts.iter().map(|p| p.0).collect();
    let mut y: Vec<f64> = pts.iter().map(|p| p.1).collect();
    let (ox, oy) = (x.clone(), y.clone());
    let r: Vec<f64> = pts.iter().map(|p| radius(p.2)).collect();
    for _ in 0..dc.iterations {
        for i in 0..n {
            x[i] += (ox[i] - x[i]) * 0.08;
            y[i] += (oy[i] - y[i]) * 0.08;
        }
        for i in 0..n {
            for j in (i + 1)..n {
                let dx = x[j] - x[i];
                let dy = y[j] - y[i];
                let d = (dx * dx + dy * dy).sqrt();
                let target = r[i] + r[j];
                if d > 1e-9 && d < target {
                    let push = (target - d) / d * 0.5;
                    let (mx, my) = (dx * push, dy * push);
                    x[i] -= mx;
                    y[i] -= my;
                    x[j] += mx;
                    y[j] += my;
                } else if d <= 1e-9 {
                    x[j] += target * 0.5;
                }
            }
        }
    }
    (0..n).map(|i| (x[i], y[i], r[i], pts[i].3.clone())).collect()
}

/// Sample the sphere's boundary (the ±180° meridian and the poles) and project it
/// — the outline/frame of a non-azimuthal projection (ellipse, lens, or rectangle).
/// Non-finite points (e.g. Mercator's infinite poles) are skipped.
fn sphere_outline_points(proj: &Proj) -> Vec<(f64, f64)> {
    const E: f64 = 1e-3;
    let n = 180;
    let mut pts = Vec::with_capacity(4 * (n + 1));
    let mut push = |lon: f64, lat: f64| {
        let (x, y) = proj.project(lon, lat);
        if x.is_finite() && y.is_finite() {
            pts.push((x, y));
        }
    };
    for i in 0..=n { push(-180.0 + E + (360.0 - 2.0 * E) * i as f64 / n as f64, 90.0 - E); }
    for i in 0..=n { push(180.0 - E, 90.0 - 180.0 * i as f64 / n as f64); }
    for i in 0..=n { push(180.0 - E - (360.0 - 2.0 * E) * i as f64 / n as f64, -90.0 + E); }
    for i in 0..=n { push(-180.0 + E, -90.0 + 180.0 * i as f64 / n as f64); }
    pts
}

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

    let rot = conf.rotate.as_ref().map(|v| {
        [
            v.first().copied().unwrap_or(0.0),
            v.get(1).copied().unwrap_or(0.0),
            v.get(2).copied().unwrap_or(0.0),
        ]
    });
    let proj = projection::from_config(conf.projection.take(), rot);
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

    // For non-azimuthal projections a `sphere` config draws the projection's frame
    // (ellipse / lens / rectangle) as an ocean background, sampled from the sphere
    // boundary. (Azimuthal projections use the disc above instead.)
    let sphere_frame: Option<Vec<(f64, f64)>> = if conf.sphere.is_some() && proj.clip_center().is_none() {
        Some(sphere_outline_points(&proj))
    } else {
        None
    };

    let mut labels: Vec<LabelInstance> = Vec::new();
    let mut patterns = PatternDefs::new();
    let mut geo_buf = String::new();
    let mut bounds = BoundsAccumulator::new();
    let mut out = RenderOutput { max_gap, precision: conf.precision.unwrap_or(0.0), ..Default::default() };
    // Hexbin and contour both aggregate the projected points here (not drawn).
    let want_hex = conf.hexbin.is_some() || conf.contour.is_some();
    let mut hex_points: Vec<(f64, f64)> = Vec::new();
    // When a Dorling cartogram is on, features become circles collected here.
    let want_dorling = conf.dorling.is_some();
    let mut dorling_pts: Vec<(f64, f64, f64, String)> = Vec::new();

    // Single pass: render features with on-the-fly projection, accumulating bounds + centroids.
    // The color scale (choropleth) is built first — it may pre-scan properties for its domain —
    // then returned so the legend can be drawn once the viewbox is known.
    let (scale, size_scale) = with_parsed_geojson(geojson, |geojson| {
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
                    let hp = if want_hex { Some(&mut hex_points) } else { None };
                    let dp = if want_dorling { Some(&mut dorling_pts) } else { None };
                    render_feature(&mut geo_buf, feat, &mut labels, &conf, scale.as_ref(), size.as_ref(), clip.as_ref(), &mut patterns, &mut out, &proj, &mut bounds, hp, dp);
                }
            }
            GeoJson::Feature(feat) => {
                let hp = if want_hex { Some(&mut hex_points) } else { None };
                let dp = if want_dorling { Some(&mut dorling_pts) } else { None };
                render_feature(&mut geo_buf, feat, &mut labels, &conf, scale.as_ref(), size.as_ref(), clip.as_ref(), &mut patterns, &mut out, &proj, &mut bounds, hp, dp);
            }
            GeoJson::Geometry(geom) => {
                let style = resolve_style(&conf, None);
                out.clear();
                let mut centroid = Centroid::new();
                render_geometry(&mut out, geom, &proj, &mut bounds, &mut centroid, false, clip.as_ref());
                write_geometry(&mut geo_buf, &out, &style, &mut patterns, !want_hex);
                if want_hex {
                    hex_points.extend(out.points.iter().copied());
                }
            }
        }
        (scale, size)
    })?;

    // When an ocean disc is drawn, include it in the auto viewbox so it isn't
    // clipped. Without a disc the viewbox follows the (already limb-clipped)
    // geometry, so a partial-globe map isn't padded out to the full hemisphere.
    if let (Some(Clip::Circle(circle)), Some(_)) = (&clip, &conf.sphere) {
        let ((cx, cy), r) = circle.disc(&proj);
        bounds.add(cx - r, cy - r);
        bounds.add(cx + r, cy + r);
    }
    if let Some(frame) = &sphere_frame {
        for &(x, y) in frame {
            bounds.add(x, y);
        }
    }

    // Lay out the Dorling cartogram (if any) before the viewbox so its circles fit.
    let dorling_circles = match &conf.dorling {
        Some(dc) => compute_dorling(std::mem::take(&mut dorling_pts), dc),
        None => Vec::new(),
    };
    for &(x, y, r, _) in &dorling_circles {
        bounds.add(x - r, y - r);
        bounds.add(x + r, y + r);
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

    // clip_angle (azimuthal small-circle) / clip_extent (rectangle): clip the map
    // content to a shape via an SVG clipPath wrapping everything but the legend.
    let clipping = if let (Some(a), Some(Clip::Circle(circle))) = (conf.clip_angle, &clip) {
        let ((cx, cy), r) = circle.disc_at(&proj, a.to_radians());
        svg.push_str(r#"<clipPath id="mclip"><circle cx=""#);
        push_f64(&mut svg, cx);
        svg.push_str(r#"" cy=""#);
        push_f64(&mut svg, cy);
        svg.push_str(r#"" r=""#);
        push_f64(&mut svg, r);
        svg.push_str(r#""/></clipPath>"#);
        true
    } else if let Some([x0, y0, x1, y1]) = conf.clip_extent {
        svg.push_str(r#"<clipPath id="mclip"><rect x=""#);
        push_f64(&mut svg, x0);
        svg.push_str(r#"" y=""#);
        push_f64(&mut svg, y0);
        svg.push_str(r#"" width=""#);
        push_f64(&mut svg, x1 - x0);
        svg.push_str(r#"" height=""#);
        push_f64(&mut svg, y1 - y0);
        svg.push_str(r#""/></clipPath>"#);
        true
    } else {
        false
    };
    if clipping {
        svg.push_str(r##"<g clip-path="url(#mclip)">"##);
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

    // Sphere (ocean) frame for non-azimuthal projections, behind graticule/geometry.
    if let (Some(frame), Some(sphere)) = (&sphere_frame, &conf.sphere) {
        if frame.len() > 2 {
            svg.push_str(r#"<path fill=""#);
            svg.push_str(&sphere.fill);
            if let Some(ref stroke) = sphere.stroke {
                svg.push_str(r#"" stroke=""#);
                svg.push_str(stroke);
                svg.push_str(r#"" stroke-width=""#);
                push_f64(&mut svg, sphere.stroke_width);
            }
            svg.push_str(r#"" d=""#);
            for (i, (x, y)) in frame.iter().enumerate() {
                svg.push(if i == 0 { 'M' } else { 'L' });
                push_f64(&mut svg, *x);
                svg.push(' ');
                push_f64(&mut svg, *y);
                svg.push(' ');
            }
            svg.push_str(r#"Z"/>"#);
        }
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

    // Hexbin: aggregate the collected points into a hex grid, colored by count.
    if let Some(hx) = &conf.hexbin {
        let bins = hexbin::bin(&hex_points, hx.radius);
        let max = bins.iter().map(|b| b.count).max().unwrap_or(1).max(1);
        let colors = scheme_colors(&hx.scheme, hx.n).unwrap_or_else(|| vec!["#cccccc".to_string()]);
        let n = colors.len();
        for b in &bins {
            let t = if max > 1 { (b.count - 1) as f64 / (max - 1) as f64 } else { 0.0 };
            let idx = ((t * n as f64) as usize).min(n - 1);
            svg.push_str(r#"<path d=""#);
            for (i, (x, y)) in hexbin::hexagon(b.cx, b.cy, hx.radius).iter().enumerate() {
                svg.push(if i == 0 { 'M' } else { 'L' });
                push_f64(&mut svg, *x);
                svg.push(' ');
                push_f64(&mut svg, *y);
                svg.push(' ');
            }
            svg.push_str(r#"Z" fill=""#);
            svg.push_str(&colors[idx]);
            if let Some(ref stroke) = hx.stroke {
                svg.push_str(r#"" stroke=""#);
                svg.push_str(stroke);
                svg.push_str(r#"" stroke-width=""#);
                push_f64(&mut svg, hx.stroke_width);
            }
            svg.push_str(r#""/>"#);
        }
    }

    // Density contours (iso-lines of a blurred point KDE), colored by level.
    if let Some(cc) = &conf.contour {
        let rect = (viewbox.0, viewbox.1, viewbox.0 + viewbox.2, viewbox.1 + viewbox.3);
        let grid = contour::density(&hex_points, rect, cc.cell_size.unwrap_or(0.0), cc.bandwidth);
        let max = grid.max();
        if max > 0.0 {
            let colors = scheme_colors(&cc.scheme, cc.n).unwrap_or_else(|| vec!["#3182bd".to_string()]);
            let mut segs: Vec<(f64, f64, f64, f64)> = Vec::new();
            for k in 1..=cc.n {
                let t = max * k as f64 / (cc.n as f64 + 1.0);
                segs.clear();
                contour::iso_segments(&grid, t, &mut segs);
                if segs.is_empty() {
                    continue;
                }
                svg.push_str(r#"<path fill="none" stroke=""#);
                svg.push_str(&colors[(k - 1).min(colors.len() - 1)]);
                svg.push_str(r#"" stroke-width=""#);
                push_f64(&mut svg, cc.stroke_width);
                svg.push_str(r#"" d=""#);
                for (x1, y1, x2, y2) in &segs {
                    svg.push('M');
                    push_f64(&mut svg, *x1);
                    svg.push(' ');
                    push_f64(&mut svg, *y1);
                    svg.push('L');
                    push_f64(&mut svg, *x2);
                    svg.push(' ');
                    push_f64(&mut svg, *y2);
                    svg.push(' ');
                }
                svg.push_str(r#""/>"#);
            }
        }
    }

    // Dorling cartogram circles.
    if let Some(dc) = &conf.dorling {
        for (x, y, r, fill) in &dorling_circles {
            svg.push_str(r#"<circle cx=""#);
            push_f64(&mut svg, *x);
            svg.push_str(r#"" cy=""#);
            push_f64(&mut svg, *y);
            svg.push_str(r#"" r=""#);
            push_f64(&mut svg, *r);
            svg.push_str(r#"" fill=""#);
            svg.push_str(fill);
            if let Some(ref stroke) = dc.stroke {
                svg.push_str(r#"" stroke=""#);
                svg.push_str(stroke);
                svg.push_str(r#"" stroke-width=""#);
                push_f64(&mut svg, dc.stroke_width);
            }
            svg.push_str(r#""/>"#);
        }
    }

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

    // Labels — optionally cull ones whose box overlaps an already-placed label.
    if conf.label_collide {
        let mut placed: Vec<(f64, f64, f64, f64)> = Vec::new();
        for label in &labels {
            let w = label.text.chars().count() as f64 * label.font_size * 0.55;
            let h = label.font_size;
            let x0 = if label.anchor == "middle" { label.x - w / 2.0 } else { label.x };
            let (x1, y0, y1) = (x0 + w, label.y - h / 2.0, label.y + h / 2.0);
            let hit = placed.iter().any(|&(px0, py0, px1, py1)| x0 < px1 && x1 > px0 && y0 < py1 && y1 > py0);
            if !hit {
                placed.push((x0, y0, x1, y1));
                write_label(&mut svg, label);
            }
        }
    } else {
        for label in &labels {
            write_label(&mut svg, label);
        }
    }

    // Close the clip group (legends/keys are drawn unclipped, on top).
    if clipping {
        svg.push_str("</g>");
    }

    // Legend (choropleth key), drawn last so it sits on top.
    if let (Some(scale), Some(legend)) = (scale.as_ref(), conf.legend.as_ref()) {
        render_legend(&mut svg, scale, legend, viewbox);
    }
    // Size key (nested circles) for a proportional-symbol map.
    if let (Some(size), Some(legend)) = (size_scale.as_ref(), conf.size_legend.as_ref()) {
        render_size_legend(&mut svg, size, legend, viewbox);
    }

    svg.push_str("</svg>");
    Ok(svg.into_bytes())
}

/// Compute the projected geometry bounds for one dataset under `config`'s
/// projection (+ clip / sphere disc), returned as JSON `[x, y, w, h]` with zero
/// padding. The `render-layers` Typst helper unions these across layers to pick
/// one shared viewbox so every layer lines up.
#[wasm_func]
pub fn bounds(geojson: &[u8], config: &[u8]) -> Result<Vec<u8>, String> {
    let mut conf: StyleConfig = if config.is_empty() {
        StyleConfig::default()
    } else {
        serde_json::from_slice(config).map_err(|e| format!("invalid config: {e}"))?
    };
    let rot = conf.rotate.as_ref().map(|v| {
        [
            v.first().copied().unwrap_or(0.0),
            v.get(1).copied().unwrap_or(0.0),
            v.get(2).copied().unwrap_or(0.0),
        ]
    });
    let proj = projection::from_config(conf.projection.take(), rot);
    let max_gap = proj.antimeridian_gap();
    let clip: Option<Clip> = if let Some(c) = proj.clip_center() {
        Some(Clip::Circle(ClipCircle::new(c)))
    } else if conf.antimeridian {
        proj.antimeridian_center().map(|central_meridian| Clip::Antimeridian { central_meridian })
    } else {
        None
    };

    let mut bounds = BoundsAccumulator::new();
    let mut out = RenderOutput { max_gap, precision: conf.precision.unwrap_or(0.0), ..Default::default() };
    let mut centroid = Centroid::new();

    with_parsed_geojson(geojson, |gj| {
        let mut accumulate = |g: &Geometry, out: &mut RenderOutput, b: &mut BoundsAccumulator| {
            out.clear();
            render_geometry(out, g, &proj, b, &mut centroid, false, clip.as_ref());
        };
        match gj {
            GeoJson::FeatureCollection(features) => {
                for f in features {
                    if let Some(g) = &f.geometry {
                        accumulate(g, &mut out, &mut bounds);
                    }
                }
            }
            GeoJson::Feature(f) => {
                if let Some(g) = &f.geometry {
                    accumulate(g, &mut out, &mut bounds);
                }
            }
            GeoJson::Geometry(g) => accumulate(g, &mut out, &mut bounds),
        }
    })?;

    // Match geo()'s auto-viewbox: include the ocean disc / frame when drawn.
    if let (Some(Clip::Circle(circle)), Some(_)) = (&clip, &conf.sphere) {
        let ((cx, cy), r) = circle.disc(&proj);
        bounds.add(cx - r, cy - r);
        bounds.add(cx + r, cy + r);
    }
    if conf.sphere.is_some() && proj.clip_center().is_none() {
        for (x, y) in sphere_outline_points(&proj) {
            bounds.add(x, y);
        }
    }

    let (x, y, w, h) = bounds.viewbox(0.0);
    Ok(format!("[{},{},{},{}]", x, y, w, h).into_bytes())
}

/// Projected centroid of every feature, as a JSON array of `[x, y]` (or `null`
/// for a feature without geometry), under `config`'s projection. Lets a Typst
/// caller place custom markers/labels at feature centers (d3 geoPath.centroid).
#[wasm_func]
pub fn centroids(geojson: &[u8], config: &[u8]) -> Result<Vec<u8>, String> {
    let mut conf: StyleConfig = if config.is_empty() {
        StyleConfig::default()
    } else {
        serde_json::from_slice(config).map_err(|e| format!("invalid config: {e}"))?
    };
    let rot = conf.rotate.as_ref().map(|v| {
        [v.first().copied().unwrap_or(0.0), v.get(1).copied().unwrap_or(0.0), v.get(2).copied().unwrap_or(0.0)]
    });
    let proj = projection::from_config(conf.projection.take(), rot);
    let max_gap = proj.antimeridian_gap();
    let clip: Option<Clip> = proj.clip_center().map(|c| Clip::Circle(ClipCircle::new(c)));

    let mut out = RenderOutput { max_gap, ..Default::default() };
    let mut result = String::from("[");
    with_parsed_geojson(geojson, |gj| {
        let one = |geom: Option<&Geometry>, result: &mut String, out: &mut RenderOutput| {
            if !result.ends_with('[') {
                result.push(',');
            }
            match geom {
                Some(g) => {
                    out.clear();
                    let mut c = Centroid::new();
                    let mut b = BoundsAccumulator::new();
                    render_geometry(out, g, &proj, &mut b, &mut c, true, clip.as_ref());
                    match c.get() {
                        Some((x, y)) => result.push_str(&format!("[{},{}]", x, y)),
                        None => result.push_str("null"),
                    }
                }
                None => result.push_str("null"),
            }
        };
        match gj {
            GeoJson::FeatureCollection(features) => {
                for f in features {
                    one(f.geometry.as_ref(), &mut result, &mut out);
                }
            }
            GeoJson::Feature(f) => one(f.geometry.as_ref(), &mut result, &mut out),
            GeoJson::Geometry(g) => one(Some(g), &mut result, &mut out),
        }
    })?;
    result.push(']');
    Ok(result.into_bytes())
}
