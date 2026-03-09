use geo::algorithm::centroid::Centroid;
use geojson::{GeoJson, Value};
use svg::node::element::path::Data;

#[derive(Default)]
pub struct RenderOutput {
    pub polygon_data: Data,
    pub line_data: Data,
    pub points: Vec<(f64, f64)>,
}

/// Extract altitude from a GeoJSON coordinate (3rd element), returning 0.0 if absent.
pub fn coord_altitude(coord: &[f64]) -> Option<f64> {
    coord.get(2).copied()
}

/// Extract the first altitude found in a geometry value.
pub fn first_altitude(value: &Value) -> Option<f64> {
    match value {
        Value::Point(c) => coord_altitude(c),
        Value::MultiPoint(cs) | Value::LineString(cs) => {
            cs.iter().find_map(|c| coord_altitude(c))
        }
        Value::Polygon(rs) | Value::MultiLineString(rs) => {
            rs.iter().flatten().find_map(|c| coord_altitude(c))
        }
        Value::MultiPolygon(ps) => {
            ps.iter().flatten().flatten().find_map(|c| coord_altitude(c))
        }
        Value::GeometryCollection(gs) => {
            gs.iter().find_map(|g| first_altitude(&g.value))
        }
    }
}

pub fn geometry_centroid(value: &Value) -> Option<(f64, f64)> {
    let geom: geo::Geometry<f64> = value.clone().try_into().ok()?;
    let c = geom.centroid()?;
    Some((c.x(), c.y()))
}

pub fn render_geometry(out: &mut RenderOutput, value: &Value) {
    match value {
        Value::Point(ref coord) => {
            if coord.len() >= 2 {
                out.points.push((coord[0], coord[1]));
            }
        }
        Value::MultiPoint(ref coords) => {
            for coord in coords {
                if coord.len() >= 2 {
                    out.points.push((coord[0], coord[1]));
                }
            }
        }
        Value::LineString(ref coords) => {
            out.line_data = draw_line(std::mem::take(&mut out.line_data), coords);
        }
        Value::MultiLineString(ref lines) => {
            for coords in lines {
                out.line_data = draw_line(std::mem::take(&mut out.line_data), coords);
            }
        }
        Value::Polygon(ref poly) => {
            out.polygon_data = draw_polygon(std::mem::take(&mut out.polygon_data), poly);
        }
        Value::MultiPolygon(ref polys) => {
            out.polygon_data = polys
                .iter()
                .fold(std::mem::take(&mut out.polygon_data), |d, poly| {
                    draw_polygon(d, poly)
                });
        }
        Value::GeometryCollection(ref geoms) => {
            for geom in geoms {
                render_geometry(out, &geom.value);
            }
        }
    }
}

pub fn draw_polygon(data: Data, coords: &[Vec<Vec<f64>>]) -> Data {
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

pub fn draw_line(mut data: Data, coords: &[Vec<f64>]) -> Data {
    let mut points = coords.iter();
    if let Some(p0) = points.next() {
        if p0.len() >= 2 {
            data = data.move_to((p0[0], -p0[1]));
            for p in points {
                if p.len() >= 2 {
                    data = data.line_to((p[0], -p[1]));
                }
            }
        }
    }
    data
}

fn try_bbox(bbox: &Option<geojson::Bbox>) -> Option<[f64; 4]> {
    let b = bbox.as_ref()?;
    if b.len() >= 4 {
        Some([b[0], b[1], b[2], b[3]])
    } else {
        None
    }
}

pub fn compute_viewbox(geojson: &GeoJson, padding: f64) -> (f64, f64, f64, f64) {
    let bbox_from_geojson = match geojson {
        GeoJson::FeatureCollection(fc) => try_bbox(&fc.bbox),
        GeoJson::Feature(f) => try_bbox(&f.bbox),
        GeoJson::Geometry(g) => try_bbox(&g.bbox),
    };

    let (min_x, min_y, max_x, max_y) = if let Some([min_x, min_y, max_x, max_y]) = bbox_from_geojson
    {
        (min_x, min_y, max_x, max_y)
    } else {
        let mut bounds = [
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::INFINITY,
            f64::NEG_INFINITY,
        ];

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
                Value::LineString(cs) | Value::MultiPoint(cs) => {
                    cs.iter().for_each(|c| update(c))
                }
                Value::Polygon(rs) | Value::MultiLineString(rs) => {
                    rs.iter().flatten().for_each(|c| update(c))
                }
                Value::MultiPolygon(ps) => ps.iter().flatten().flatten().for_each(|c| update(c)),
                Value::GeometryCollection(gs) => {
                    gs.iter().for_each(|g| process_value(&g.value, update))
                }
            }
        }

        match geojson {
            GeoJson::FeatureCollection(fc) => {
                fc.features
                    .iter()
                    .filter_map(|f| f.geometry.as_ref())
                    .for_each(|g| process_value(&g.value, &mut update));
            }
            GeoJson::Feature(f) => {
                if let Some(g) = f.geometry.as_ref() {
                    process_value(&g.value, &mut update);
                }
            }
            GeoJson::Geometry(g) => process_value(&g.value, &mut update),
        }

        if bounds[0] == f64::INFINITY {
            return (0.0, 0.0, 100.0, 100.0);
        }

        (bounds[0], bounds[2], bounds[1], bounds[3])
    };

    let (w, h) = (max_x - min_x, max_y - min_y);
    let (px, py) = (w * padding, h * padding);
    let (fw, fh) = ((w + 2.0 * px).max(1.0), (h + 2.0 * py).max(1.0));

    (min_x - px, -(max_y + py), fw, fh)
}
