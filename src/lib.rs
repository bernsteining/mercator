use wasm_minimal_protocol::*;

use geojson::{GeoJson, Geometry, Value};
use svg::node::element::path::Data;
use svg::node::element::Path;
use svg::Document;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct StyleConfig {
    stroke: String,
    stroke_width: f64,
    fill: String,
    fill_opacity: f64,
    viewbox: (f64, f64, f64, f64),
}

impl Default for StyleConfig {
    fn default() -> Self {
        Self {
            stroke: "black".to_string(),
            stroke_width: 0.05,
            fill: "blue".to_string(),
            fill_opacity: 0.5,
            viewbox: (10., -70., 15., 15.),
        }
    }
}

fn doc_from_config(data: Data, config: StyleConfig) -> Document {
    Document::new().set("viewBox", config.viewbox).add(
        Path::new()
            .set("fill", config.fill)
            .set("fill-opacity", config.fill_opacity)
            .set("stroke", config.stroke)
            .set("stroke-width", config.stroke_width)
            .set("d", data),
    )
}

initiate_protocol!();

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

#[wasm_func]
pub fn geo(geojson: &[u8], config: &[u8]) -> Result<Vec<u8>, String> {
    let conf: StyleConfig = {
        match serde_json::from_slice(config) {
            Ok(conf) => conf,
            _ => StyleConfig::default(),
        }
    };

    let content = String::from_utf8(geojson.to_vec()).map_err(|e| e.to_string())?;
    let geojson = content.parse::<GeoJson>().map_err(|e| e.to_string())?;
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
