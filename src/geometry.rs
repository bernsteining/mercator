use geojson::{GeoJson, Value};
use svg::node::element::path::Data;

/// Visit every coordinate in a GeoJSON Value (read-only).
pub fn for_each_coord(value: &Value, f: &mut dyn FnMut(&[f64])) {
    match value {
        Value::Point(c) => f(c),
        Value::MultiPoint(cs) | Value::LineString(cs) => cs.iter().for_each(|c| f(c)),
        Value::Polygon(rs) | Value::MultiLineString(rs) => {
            rs.iter().flatten().for_each(|c| f(c))
        }
        Value::MultiPolygon(ps) => ps.iter().flatten().flatten().for_each(|c| f(c)),
        Value::GeometryCollection(gs) => gs.iter().for_each(|g| for_each_coord(&g.value, f)),
    }
}

/// Visit every coordinate in a GeoJSON Value (mutable).
pub fn for_each_coord_mut(value: &mut Value, f: &mut dyn FnMut(&mut Vec<f64>)) {
    match value {
        Value::Point(c) => f(c),
        Value::MultiPoint(cs) | Value::LineString(cs) => cs.iter_mut().for_each(|c| f(c)),
        Value::Polygon(rs) | Value::MultiLineString(rs) => {
            rs.iter_mut().flatten().for_each(|c| f(c))
        }
        Value::MultiPolygon(ps) => ps.iter_mut().flatten().flatten().for_each(|c| f(c)),
        Value::GeometryCollection(gs) => {
            gs.iter_mut().for_each(|g| for_each_coord_mut(&mut g.value, f))
        }
    }
}

/// Accumulates SVG path commands while handling NaN coordinates and antimeridian gaps.
pub struct PathBuilder {
    data: Data,
    prev: Option<(f64, f64)>,
    first: Option<(f64, f64)>,
    max_gap: f64,
}

impl PathBuilder {
    pub fn new(data: Data, max_gap: f64) -> Self {
        Self { data, prev: None, first: None, max_gap }
    }

    #[inline]
    fn is_gap(&self, x1: f64, y1: f64, x2: f64, y2: f64) -> bool {
        (x1 - x2).abs() > self.max_gap || (y1 - y2).abs() > self.max_gap
    }

    /// Add a point, handling NaN skipping and gap-based path breaking.
    #[inline]
    pub fn add(&mut self, x: f64, y: f64) {
        if !x.is_finite() || !y.is_finite() {
            self.prev = None;
            return;
        }
        if let Some((px, py)) = self.prev {
            if self.is_gap(x, y, px, py) {
                self.data = std::mem::take(&mut self.data).move_to((x, y));
            } else {
                self.data = std::mem::take(&mut self.data).line_to((x, y));
            }
        } else {
            self.data = std::mem::take(&mut self.data).move_to((x, y));
            if self.first.is_none() {
                self.first = Some((x, y));
            }
        }
        self.prev = Some((x, y));
    }

    /// Close the current sub-path if the last→first gap is small enough.
    pub fn close_if_continuous(&mut self) {
        if let (Some((lx, ly)), Some((fx, fy))) = (self.prev, self.first) {
            if !self.is_gap(lx, ly, fx, fy) {
                self.data = std::mem::take(&mut self.data).close();
            }
        }
        self.prev = None;
        self.first = None;
    }

    pub fn finish(self) -> Data {
        self.data
    }
}

pub struct RenderOutput {
    pub polygon_data: Data,
    pub line_data: Data,
    pub points: Vec<(f64, f64)>,
    /// Maximum x-gap between consecutive vertices before treating as antimeridian crossing.
    pub max_gap: f64,
}

impl Default for RenderOutput {
    fn default() -> Self {
        Self {
            polygon_data: Data::default(),
            line_data: Data::default(),
            points: Vec::new(),
            max_gap: f64::INFINITY,
        }
    }
}

pub fn first_altitude(value: &Value) -> Option<f64> {
    let mut result = None;
    for_each_coord(value, &mut |c| {
        if result.is_none() {
            result = c.get(2).copied();
        }
    });
    result
}

pub fn geometry_centroid(value: &Value) -> Option<(f64, f64)> {
    let mut sum_x = 0.0;
    let mut sum_y = 0.0;
    let mut count = 0u32;
    for_each_coord(value, &mut |c| {
        if c.len() >= 2 && c[0].is_finite() && c[1].is_finite() {
            sum_x += c[0];
            sum_y += c[1];
            count += 1;
        }
    });
    if count == 0 {
        None
    } else {
        Some((sum_x / count as f64, sum_y / count as f64))
    }
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
            out.line_data = draw_line(std::mem::take(&mut out.line_data), coords, out.max_gap);
        }
        Value::MultiLineString(ref lines) => {
            for coords in lines {
                out.line_data = draw_line(std::mem::take(&mut out.line_data), coords, out.max_gap);
            }
        }
        Value::Polygon(ref poly) => {
            out.polygon_data = draw_polygon(std::mem::take(&mut out.polygon_data), poly, out.max_gap);
        }
        Value::MultiPolygon(ref polys) => {
            let gap = out.max_gap;
            out.polygon_data = polys
                .iter()
                .fold(std::mem::take(&mut out.polygon_data), |d, poly| {
                    draw_polygon(d, poly, gap)
                });
        }
        Value::GeometryCollection(ref geoms) => {
            for geom in geoms {
                render_geometry(out, &geom.value);
            }
        }
    }
}

pub fn draw_polygon(data: Data, coords: &[Vec<Vec<f64>>], max_gap: f64) -> Data {
    let mut pb = PathBuilder::new(data, max_gap);
    for ring in coords {
        for p in ring {
            if p.len() >= 2 {
                pb.add(p[0], p[1]);
            }
        }
        pb.close_if_continuous();
    }
    pb.finish()
}

pub fn draw_line(data: Data, coords: &[Vec<f64>], max_gap: f64) -> Data {
    let mut pb = PathBuilder::new(data, max_gap);
    for p in coords {
        if p.len() >= 2 {
            pb.add(p[0], p[1]);
        }
    }
    pb.finish()
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
            if coord.len() >= 2 && coord[0].is_finite() && coord[1].is_finite() {
                bounds[0] = bounds[0].min(coord[0]);
                bounds[1] = bounds[1].max(coord[0]);
                bounds[2] = bounds[2].min(coord[1]);
                bounds[3] = bounds[3].max(coord[1]);
            }
        };

        match geojson {
            GeoJson::FeatureCollection(fc) => {
                fc.features
                    .iter()
                    .filter_map(|f| f.geometry.as_ref())
                    .for_each(|g| for_each_coord(&g.value, &mut update));
            }
            GeoJson::Feature(f) => {
                if let Some(g) = f.geometry.as_ref() {
                    for_each_coord(&g.value, &mut update);
                }
            }
            GeoJson::Geometry(g) => for_each_coord(&g.value, &mut update),
        }

        if bounds[0] == f64::INFINITY {
            return (0.0, 0.0, 100.0, 100.0);
        }

        (bounds[0], bounds[2], bounds[1], bounds[3])
    };

    let (w, h) = (max_x - min_x, max_y - min_y);
    let p = w.max(h) * padding;
    let (fw, fh) = ((w + 2.0 * p).max(1.0), (h + 2.0 * p).max(1.0));

    (min_x - p, min_y - p, fw, fh)
}
