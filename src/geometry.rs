use crate::clip::{clip_polygon_ring, Clip};
use crate::clip_antimeridian::clip_polygon as clip_antimeridian_polygon;
use crate::model::{Geometry, Pos};
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
    /// Used by callers (graticule, tissot) that haven't already checked finiteness.
    #[inline]
    pub fn add(&mut self, x: f64, y: f64) {
        if !x.is_finite() || !y.is_finite() {
            self.prev = None;
            return;
        }
        self.push_point(x, y);
    }

    /// Append a point already known to be finite (gap-based M/L emission). The
    /// feature render path checks finiteness once, up front, and calls this —
    /// avoiding a second `is_finite` test per vertex.
    #[inline]
    pub fn push_point(&mut self, x: f64, y: f64) {
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

    /// Break the current sub-path (a non-finite/clipped vertex was encountered).
    #[inline]
    pub fn break_subpath(&mut self) {
        self.prev = None;
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
    /// Adaptive-resampling tolerance in projected units (0 = off). When > 0, long
    /// segments are subdivided so the projected path follows the projection's curve.
    pub precision: f64,
}

impl Default for RenderOutput {
    fn default() -> Self {
        Self {
            polygon_data: String::new(),
            line_data: String::new(),
            points: Vec::new(),
            max_gap: f64::INFINITY,
            precision: 0.0,
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

/// Project a point and update bounds (and, when labels need it, the centroid).
/// Used for Point/MultiPoint geometries.
#[inline]
fn project_and_track(
    lon: f64,
    lat: f64,
    proj: &Proj,
    bounds: &mut BoundsAccumulator,
    centroid: &mut Centroid,
    track_centroid: bool,
) -> (f64, f64) {
    let (x, y) = proj.project(lon, lat);
    if x.is_finite() && y.is_finite() {
        bounds.add(x, y);
        if track_centroid {
            centroid.add(x, y);
        }
    }
    (x, y)
}

/// Project one path vertex, check finiteness once, update accumulators, and emit.
/// A non-finite projection (e.g. an azimuthal back-hemisphere point) breaks the
/// sub-path. This is the per-vertex hot path for lines and polygons.
#[inline]
fn plot_vertex(
    pb: &mut PathBuilder,
    p: Pos,
    proj: &Proj,
    bounds: &mut BoundsAccumulator,
    centroid: &mut Centroid,
    track_centroid: bool,
) {
    let (x, y) = proj.project(p.x, p.y);
    if x.is_finite() && y.is_finite() {
        bounds.add(x, y);
        if track_centroid {
            centroid.add(x, y);
        }
        pb.push_point(x, y);
    } else {
        pb.break_subpath();
    }
}

/// Emit an already-projected point (finite → append; non-finite → break sub-path).
#[inline]
fn emit_projected(pb: &mut PathBuilder, bounds: &mut BoundsAccumulator, centroid: &mut Centroid, track_centroid: bool, x: f64, y: f64) {
    if x.is_finite() && y.is_finite() {
        bounds.add(x, y);
        if track_centroid {
            centroid.add(x, y);
        }
        pb.push_point(x, y);
    } else {
        pb.break_subpath();
    }
}

const RESAMPLE_MAX_DEPTH: u8 = 16;

/// Adaptive resampling of one geographic segment `a→b`: if the projected midpoint
/// deviates from the chord midpoint by more than `√prec2`, recurse; else emit `b`.
/// `a` is assumed already emitted; emits every point after it up to and including `b`.
#[allow(clippy::too_many_arguments)]
fn resample_segment(
    pb: &mut PathBuilder,
    bounds: &mut BoundsAccumulator,
    centroid: &mut Centroid,
    track_centroid: bool,
    proj: &Proj,
    prec2: f64,
    a: Pos,
    b: Pos,
    pa: (f64, f64),
    pbp: (f64, f64),
    depth: u8,
) {
    if depth < RESAMPLE_MAX_DEPTH && pa.0.is_finite() && pa.1.is_finite() && pbp.0.is_finite() && pbp.1.is_finite() {
        // GeoJSON segments are straight in lon/lat, so subdivide there and project.
        let m = Pos { x: (a.x + b.x) * 0.5, y: (a.y + b.y) * 0.5 };
        let pm = proj.project(m.x, m.y);
        if pm.0.is_finite() && pm.1.is_finite() {
            let dx = pm.0 - (pa.0 + pbp.0) * 0.5;
            let dy = pm.1 - (pa.1 + pbp.1) * 0.5;
            if dx * dx + dy * dy > prec2 {
                resample_segment(pb, bounds, centroid, track_centroid, proj, prec2, a, m, pa, pm, depth + 1);
                resample_segment(pb, bounds, centroid, track_centroid, proj, prec2, m, b, pm, pbp, depth + 1);
                return;
            }
        }
    }
    emit_projected(pb, bounds, centroid, track_centroid, pbp.0, pbp.1);
}

/// Plot a ring/line, adaptively resampling when `precision > 0` (else vertex-by-vertex).
fn plot_path(
    pb: &mut PathBuilder,
    coords: &[Pos],
    proj: &Proj,
    bounds: &mut BoundsAccumulator,
    centroid: &mut Centroid,
    track_centroid: bool,
    precision: f64,
) {
    if precision <= 0.0 {
        for p in coords {
            plot_vertex(pb, *p, proj, bounds, centroid, track_centroid);
        }
        return;
    }
    let prec2 = precision * precision;
    let mut prev: Option<(Pos, (f64, f64))> = None;
    for &p in coords {
        let pp = proj.project(p.x, p.y);
        match prev {
            None => emit_projected(pb, bounds, centroid, track_centroid, pp.0, pp.1),
            Some((a, pa)) => resample_segment(pb, bounds, centroid, track_centroid, proj, prec2, a, p, pa, pp, 0),
        }
        prev = Some((p, pp));
    }
}

pub fn render_geometry(
    out: &mut RenderOutput,
    value: &Geometry,
    proj: &Proj,
    bounds: &mut BoundsAccumulator,
    centroid: &mut Centroid,
    track_centroid: bool,
    clip: Option<&Clip>,
) {
    match value {
        Geometry::Point(coord) => {
            let (x, y) = project_and_track(coord.x, coord.y, proj, bounds, centroid, track_centroid);
            out.points.push((x, y));
        }
        Geometry::MultiPoint(coords) => {
            for coord in coords {
                let (x, y) = project_and_track(coord.x, coord.y, proj, bounds, centroid, track_centroid);
                out.points.push((x, y));
            }
        }
        Geometry::LineString(coords) => {
            let data = std::mem::take(&mut out.line_data);
            out.line_data = draw_line(data, coords, out.max_gap, out.precision, proj, bounds, centroid, track_centroid);
        }
        Geometry::MultiLineString(lines) => {
            for coords in lines {
                let data = std::mem::take(&mut out.line_data);
                out.line_data = draw_line(data, coords, out.max_gap, out.precision, proj, bounds, centroid, track_centroid);
            }
        }
        Geometry::Polygon(poly) => {
            let data = std::mem::take(&mut out.polygon_data);
            out.polygon_data = draw_polygon(data, poly, out.max_gap, out.precision, proj, bounds, centroid, track_centroid, clip);
        }
        Geometry::MultiPolygon(polys) => {
            let (gap, prec) = (out.max_gap, out.precision);
            out.polygon_data = polys
                .iter()
                .fold(std::mem::take(&mut out.polygon_data), |d, poly| {
                    draw_polygon(d, poly, gap, prec, proj, bounds, centroid, track_centroid, clip)
                });
        }
        Geometry::GeometryCollection(geoms) => {
            for geom in geoms {
                render_geometry(out, geom, proj, bounds, centroid, track_centroid, clip);
            }
        }
    }
}

/// Emit a closed sub-path from already-projected points, updating accumulators.
fn emit_ring(
    d: &mut String,
    pts: &[(f64, f64)],
    bounds: &mut BoundsAccumulator,
    centroid: &mut Centroid,
    track_centroid: bool,
) {
    let mut started = false;
    for &(x, y) in pts {
        if !x.is_finite() || !y.is_finite() {
            continue;
        }
        bounds.add(x, y);
        if track_centroid {
            centroid.add(x, y);
        }
        push_coord(d, if started { 'L' } else { 'M' }, x, y);
        started = true;
    }
    if started {
        d.push('Z');
    }
}

#[allow(clippy::too_many_arguments)]
#[allow(clippy::too_many_arguments)]
pub fn draw_polygon(
    data: String,
    coords: &[Vec<Pos>],
    max_gap: f64,
    precision: f64,
    proj: &Proj,
    bounds: &mut BoundsAccumulator,
    centroid: &mut Centroid,
    track_centroid: bool,
    clip: Option<&Clip>,
) -> String {
    // Clipping (globe limb or antimeridian): clip each ring and emit the re-stitched
    // projected ring(s) directly, bypassing the gap/NaN heuristics.
    if let Some(clip) = clip {
        let mut d = data;
        match clip {
            Clip::Circle(circle) => {
                let mut pts: Vec<(f64, f64)> = Vec::new();
                for ring in coords {
                    pts.clear();
                    if clip_polygon_ring(ring, circle, proj, &mut pts) {
                        emit_ring(&mut d, &pts, bounds, centroid, track_centroid);
                    }
                }
            }
            Clip::Antimeridian { central_meridian } => {
                // The whole polygon (exterior + holes) is clipped together.
                let mut rings: Vec<Vec<(f64, f64)>> = Vec::new();
                clip_antimeridian_polygon(coords, *central_meridian, proj, &mut rings);
                for r in &rings {
                    emit_ring(&mut d, r, bounds, centroid, track_centroid);
                }
            }
        }
        return d;
    }

    let mut pb = PathBuilder::new(data, max_gap);
    for ring in coords {
        plot_path(&mut pb, ring, proj, bounds, centroid, track_centroid, precision);
        pb.close_if_continuous();
    }
    pb.finish()
}

pub fn draw_line(
    data: String,
    coords: &[Pos],
    max_gap: f64,
    precision: f64,
    proj: &Proj,
    bounds: &mut BoundsAccumulator,
    centroid: &mut Centroid,
    track_centroid: bool,
) -> String {
    let mut pb = PathBuilder::new(data, max_gap);
    plot_path(&mut pb, coords, proj, bounds, centroid, track_centroid, precision);
    pb.finish()
}
