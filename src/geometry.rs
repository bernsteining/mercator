use geojson::Value;

use crate::projection::Proj;

/// Push f64 as string using ryu (bypasses std::fmt machinery).
#[inline]
pub fn push_f64(buf: &mut String, val: f64) {
    buf.push_str(ryu::Buffer::new().format(val));
}

/// Push "Cx,y" (where C is a command char like M or L) using ryu for fast f64→str.
#[inline]
fn push_coord(buf: &mut String, cmd: char, x: f64, y: f64) {
    let mut b = ryu::Buffer::new();
    buf.push(cmd);
    buf.push_str(b.format(x));
    buf.push(',');
    buf.push_str(b.format(y));
}

/// Accumulates bounding box from projected coordinates.
pub struct BoundsAccumulator {
    min_x: f64,
    max_x: f64,
    min_y: f64,
    max_y: f64,
}

impl BoundsAccumulator {
    pub fn new() -> Self {
        Self {
            min_x: f64::INFINITY,
            max_x: f64::NEG_INFINITY,
            min_y: f64::INFINITY,
            max_y: f64::NEG_INFINITY,
        }
    }

    #[inline]
    pub fn add(&mut self, x: f64, y: f64) {
        self.min_x = self.min_x.min(x);
        self.max_x = self.max_x.max(x);
        self.min_y = self.min_y.min(y);
        self.max_y = self.max_y.max(y);
    }

    pub fn viewbox(&self, padding: f64) -> (f64, f64, f64, f64) {
        if self.min_x == f64::INFINITY {
            return (0.0, 0.0, 100.0, 100.0);
        }
        let (w, h) = (self.max_x - self.min_x, self.max_y - self.min_y);
        let p = w.max(h) * padding;
        let (fw, fh) = ((w + 2.0 * p).max(1.0), (h + 2.0 * p).max(1.0));
        (self.min_x - p, self.min_y - p, fw, fh)
    }
}

/// Accumulates centroid from projected coordinates (per-feature).
pub struct Centroid {
    sum_x: f64,
    sum_y: f64,
    count: u32,
}

impl Centroid {
    pub fn new() -> Self {
        Self { sum_x: 0.0, sum_y: 0.0, count: 0 }
    }

    #[inline]
    fn add(&mut self, x: f64, y: f64) {
        self.sum_x += x;
        self.sum_y += y;
        self.count += 1;
    }

    pub fn get(&self) -> Option<(f64, f64)> {
        if self.count == 0 {
            None
        } else {
            Some((self.sum_x / self.count as f64, self.sum_y / self.count as f64))
        }
    }
}

/// Accumulates SVG path commands while handling NaN coordinates and antimeridian gaps.
pub struct PathBuilder {
    data: String,
    prev: Option<(f64, f64)>,
    first: Option<(f64, f64)>,
    max_gap: f64,
}

impl PathBuilder {
    pub fn new(data: String, max_gap: f64) -> Self {
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
                push_coord(&mut self.data, 'M', x, y);
            } else {
                push_coord(&mut self.data, 'L', x, y);
            }
        } else {
            push_coord(&mut self.data, 'M', x, y);
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
                self.data.push('Z');
            }
        }
        self.prev = None;
        self.first = None;
    }

    pub fn finish(self) -> String {
        self.data
    }
}

pub struct RenderOutput {
    pub polygon_data: String,
    pub line_data: String,
    pub points: Vec<(f64, f64)>,
    /// Maximum x-gap between consecutive vertices before treating as antimeridian crossing.
    pub max_gap: f64,
}

impl Default for RenderOutput {
    fn default() -> Self {
        Self {
            polygon_data: String::new(),
            line_data: String::new(),
            points: Vec::new(),
            max_gap: f64::INFINITY,
        }
    }
}

impl RenderOutput {
    /// Clear data for reuse, retaining allocated capacity.
    pub fn clear(&mut self) {
        self.polygon_data.clear();
        self.line_data.clear();
        self.points.clear();
    }
}

pub fn first_altitude(value: &Value) -> Option<f64> {
    fn walk(value: &Value) -> Option<f64> {
        match value {
            Value::Point(c) => c.get(2).copied(),
            Value::MultiPoint(cs) | Value::LineString(cs) => {
                cs.iter().find_map(|c| c.get(2).copied())
            }
            Value::Polygon(rs) | Value::MultiLineString(rs) => {
                rs.iter().flatten().find_map(|c| c.get(2).copied())
            }
            Value::MultiPolygon(ps) => {
                ps.iter().flatten().flatten().find_map(|c| c.get(2).copied())
            }
            Value::GeometryCollection(gs) => gs.iter().find_map(|g| walk(&g.value)),
        }
    }
    walk(value)
}

/// Project a coordinate and update bounds + centroid accumulators.
#[inline]
fn project_and_track(
    lon: f64,
    lat: f64,
    proj: &Proj,
    bounds: &mut BoundsAccumulator,
    centroid: &mut Centroid,
) -> (f64, f64) {
    let (x, y) = proj.project(lon, lat);
    if x.is_finite() && y.is_finite() {
        bounds.add(x, y);
        centroid.add(x, y);
    }
    (x, y)
}

pub fn render_geometry(
    out: &mut RenderOutput,
    value: &Value,
    proj: &Proj,
    bounds: &mut BoundsAccumulator,
    centroid: &mut Centroid,
) {
    match value {
        Value::Point(ref coord) => {
            if coord.len() >= 2 {
                let (x, y) = project_and_track(coord[0], coord[1], proj, bounds, centroid);
                out.points.push((x, y));
            }
        }
        Value::MultiPoint(ref coords) => {
            for coord in coords {
                if coord.len() >= 2 {
                    let (x, y) = project_and_track(coord[0], coord[1], proj, bounds, centroid);
                    out.points.push((x, y));
                }
            }
        }
        Value::LineString(ref coords) => {
            let data = std::mem::take(&mut out.line_data);
            out.line_data = draw_line(data, coords, out.max_gap, proj, bounds, centroid);
        }
        Value::MultiLineString(ref lines) => {
            for coords in lines {
                let data = std::mem::take(&mut out.line_data);
                out.line_data = draw_line(data, coords, out.max_gap, proj, bounds, centroid);
            }
        }
        Value::Polygon(ref poly) => {
            let data = std::mem::take(&mut out.polygon_data);
            out.polygon_data = draw_polygon(data, poly, out.max_gap, proj, bounds, centroid);
        }
        Value::MultiPolygon(ref polys) => {
            let gap = out.max_gap;
            out.polygon_data = polys
                .iter()
                .fold(std::mem::take(&mut out.polygon_data), |d, poly| {
                    draw_polygon(d, poly, gap, proj, bounds, centroid)
                });
        }
        Value::GeometryCollection(ref geoms) => {
            for geom in geoms {
                render_geometry(out, &geom.value, proj, bounds, centroid);
            }
        }
    }
}

pub fn draw_polygon(
    data: String,
    coords: &[Vec<Vec<f64>>],
    max_gap: f64,
    proj: &Proj,
    bounds: &mut BoundsAccumulator,
    centroid: &mut Centroid,
) -> String {
    let mut pb = PathBuilder::new(data, max_gap);
    for ring in coords {
        for p in ring {
            if p.len() >= 2 {
                let (x, y) = project_and_track(p[0], p[1], proj, bounds, centroid);
                pb.add(x, y);
            }
        }
        pb.close_if_continuous();
    }
    pb.finish()
}

pub fn draw_line(
    data: String,
    coords: &[Vec<f64>],
    max_gap: f64,
    proj: &Proj,
    bounds: &mut BoundsAccumulator,
    centroid: &mut Centroid,
) -> String {
    let mut pb = PathBuilder::new(data, max_gap);
    for p in coords {
        if p.len() >= 2 {
            let (x, y) = project_and_track(p[0], p[1], proj, bounds, centroid);
            pb.add(x, y);
        }
    }
    pb.finish()
}
