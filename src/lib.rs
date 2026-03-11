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
use svg::node::element::{Circle, Group, Path};
use svg::Document;

use geometry::{
    compute_viewbox, first_altitude, geometry_centroid, render_geometry, RenderOutput,
};
use label::{add_label, build_labels};
use pattern::{resolve_fill, PatternDefs};
use projection::project_geojson;
use style::{resolve_style, GraticuleConfig, LabelInstance, ResolvedStyle, StyleConfig};

initiate_protocol!();

const DEFAULT_VIEWBOX_PADDING: f64 = 0.15;
const FALLBACK_VIEWBOX: (f64, f64, f64, f64) = (0.0, 0.0, 100.0, 100.0);
const GRATICULE_WIDTH_SCALE: f64 = 1000.0;

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

    let mut out = RenderOutput { max_gap, ..Default::default() };
    render_geometry(&mut out, &geom.value);
    let group = add_geometry_to_group(group, out, &style, patterns);

    if let Some((mut cx, cy)) = geometry_centroid(&geom.value) {
        let is_point = matches!(geom.value, geojson::Value::Point(_) | geojson::Value::MultiPoint(_));
        if is_point {
            let r = config.point_radius.unwrap_or(config.stroke_width * 5.0);
            cx += r * 1.5;
        }
        build_labels(labels, config, &properties, cx, cy, is_point);
    }

    group
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

#[wasm_func]
pub fn geo(geojson: &[u8], config: &[u8]) -> Result<Vec<u8>, String> {
    let mut conf: StyleConfig = serde_json::from_slice(config).unwrap_or_default();

    let content = std::str::from_utf8(geojson).map_err(|e| e.to_string())?;
    let mut geojson = match content.parse::<GeoJson>() {
        Ok(gj) => gj,
        Err(_) => topojson_convert::try_topojson(content)?,
    };

    let proj = projection::from_config(conf.projection.take());
    project_geojson(&mut geojson, &*proj);

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

    if patterns.has_patterns() {
        doc = doc.add(patterns.defs);
    }

    if let Some(ref grat) = conf.graticule {
        let vb_diag = (viewbox.2.powi(2) + viewbox.3.powi(2)).sqrt();
        let scaled_grat = GraticuleConfig {
            step: grat.step,
            color: grat.color.clone(),
            width: grat.width * vb_diag / GRATICULE_WIDTH_SCALE,
            opacity: grat.opacity,
        };
        doc = doc.add(graticule::render(&*proj, &scaled_grat, max_gap));
    }

    doc = doc.add(group);

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
        doc = doc.add(tissot::render(&*proj, &scaled, max_gap));
    }

    for label in &labels {
        doc = add_label(doc, label);
    }

    let mut buf = Vec::new();
    svg::write(&mut buf, &doc).map_err(|e| e.to_string())?;
    Ok(buf)
}
