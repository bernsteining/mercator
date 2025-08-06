use wasm_minimal_protocol::*;

use geojson::{GeoJson, Geometry, Value};
use svg::node::element::path::Data;
use svg::node::element::Path;
use svg::Document;

use serde::Deserialize;

initiate_protocol!();

#[derive(Debug, Deserialize)]
struct StyleConfig {
    stroke: String,
    stroke_width: f64,
    fill: String,
    fill_opacity: f64,
    viewbox: Option<(f64, f64, f64, f64)>,
}

impl Default for StyleConfig {
    fn default() -> Self {
        Self {
            stroke: "black".to_string(),
            stroke_width: 0.05,
            fill: "red".to_string(),
            fill_opacity: 0.5,
            viewbox: None,
        }
    }
}

fn doc_from_config(data: Data, config: StyleConfig) -> Document {
    let viewbox = config.viewbox.unwrap_or((0.0, 0.0, 100.0, 100.0));
    Document::new().set("viewBox", viewbox).add(
        Path::new()
            .set("fill", config.fill)
            .set("fill-opacity", config.fill_opacity)
            .set("stroke", config.stroke)
            .set("stroke-width", config.stroke_width)
            .set("d", data),
    )
}

fn draw_polygon(data: Data, coords: &[Vec<Vec<f64>>]) -> Data {
    coords.iter().fold(data, |mut d, ring| {
        let mut points = ring.iter();
        if let Some(p0) = points.next() {
            d = d.move_to((p0[0], -p0[1]));
            for p in points {
                d = d.line_to((p[0], -p[1]));
            }
            d.close()
        } else {
            d
        }
    })
}

pub fn compute_viewbox(geojson: &GeoJson) -> (f64, f64, f64, f64) {
    let mut bounds = [f64::INFINITY, f64::NEG_INFINITY, f64::INFINITY, f64::NEG_INFINITY]; // [min_x, max_x, min_y, max_y]
    
    let mut update = |coord: &[f64]| {
        if coord.len() >= 2 {
            bounds[0] = bounds[0].min(coord[0]);
            bounds[1] = bounds[1].max(coord[0]);
            bounds[2] = bounds[2].min(coord[1]);
            bounds[3] = bounds[3].max(coord[1]);
        }
    };
    
    fn process_value(value: &Value, update: &mut dyn FnMut(&[f64])) {
        match value {
            Value::Point(c) => update(c),
            Value::LineString(cs) | Value::MultiPoint(cs) => cs.iter().for_each(|c| update(c)),
            Value::Polygon(rs) | Value::MultiLineString(rs) => rs.iter().flatten().for_each(|c| update(c)),
            Value::MultiPolygon(ps) => ps.iter().flatten().flatten().for_each(|c| update(c)),
            Value::GeometryCollection(gs) => gs.iter().for_each(|g| process_value(&g.value, update)),
        }
    }
    
    match geojson {
        GeoJson::FeatureCollection(fc) => {
            fc.features.iter()
                .filter_map(|f| f.geometry.as_ref())
                .for_each(|g| process_value(&g.value, &mut update));
        }
        GeoJson::Geometry(g) => process_value(&g.value, &mut update),
        _ => {}
    }
    
    if bounds[0] == f64::INFINITY { return (0.0, 0.0, 100.0, 100.0); }
    
    let (w, h) = (bounds[1] - bounds[0], bounds[3] - bounds[2]);
    let (px, py) = (w * 0.1, h * 0.1);
    let (fw, fh) = ((w + 2.0 * px).max(1.0), (h + 2.0 * py).max(1.0));
    
    (bounds[0] - px, -(bounds[3] + py), fw, fh)
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
    let geojson = content.parse::<GeoJson>().map_err(|e| e.to_string())?;

    if conf.viewbox.is_none() {
        conf.viewbox = Some(compute_viewbox(&geojson));
    }

    let mut data = Data::new();

    if let GeoJson::FeatureCollection(fc) = geojson {
        for feat in fc.features {
            if let Some(Geometry { value, .. }) = feat.geometry {
                data = match value {
                    Value::Polygon(ref poly) => draw_polygon(data, poly),
                    Value::MultiPolygon(ref polys) => {
                        polys.iter().fold(data, |d, poly| draw_polygon(d, poly))
                    }
                    _ => data,
                };
            }
        }
    }

    let doc = doc_from_config(data, conf);
    let mut buf = Vec::new();
    svg::write(&mut buf, &doc).map_err(|e| e.to_string())?;
    Ok(buf)
}
