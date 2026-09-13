//! Antimeridian clipping via a faithful port of d3-geo's clip / clipAntimeridian
//! / clipRejoin. Cuts each polygon ring at the antimeridian and rejoins the pieces
//! along the map boundary — sorting intersections and linking them by winding —
//! so seam-crossing and pole-containing polygons (Antarctica) close correctly.
//!
//! Works in radians on longitudes normalized to the central meridian; output ring
//! points are projected through `proj`.

use crate::model::Pos;
use crate::projection::Proj;

const PI: f64 = std::f64::consts::PI;
const HALF_PI: f64 = std::f64::consts::FRAC_PI_2;
const QUARTER_PI: f64 = std::f64::consts::FRAC_PI_4;
const TAU: f64 = std::f64::consts::TAU;
const EPSILON: f64 = 1e-6;
const EPSILON2: f64 = 1e-12;

type P = [f64; 2]; // [lambda, phi] radians

fn clip_am_intersect(lambda0: f64, phi0: f64, lambda1: f64, phi1: f64) -> f64 {
    let sin_l = (lambda0 - lambda1).sin();
    if sin_l.abs() > EPSILON {
        let cos_phi0 = phi0.cos();
        let cos_phi1 = phi1.cos();
        ((phi0.sin() * cos_phi1 * lambda1.sin() - phi1.sin() * cos_phi0 * lambda0.sin())
            / (cos_phi0 * cos_phi1 * sin_l))
            .atan()
    } else {
        (phi0 + phi1) / 2.0
    }
}

/// A destination that accumulates line segments (rings) of [lambda, phi] points.
#[derive(Default)]
struct Sink {
    segments: Vec<Vec<P>>,
    line: Vec<P>,
    open: bool,
}
impl Sink {
    fn line_start(&mut self) {
        if self.open {
            self.line_end();
        }
        self.line = Vec::new();
        self.open = true;
    }
    fn point(&mut self, lambda: f64, phi: f64) {
        self.line.push([lambda, phi]);
    }
    fn line_end(&mut self) {
        if !self.line.is_empty() {
            self.segments.push(std::mem::take(&mut self.line));
        }
        self.open = false;
    }
}

/// Clip one ring at the antimeridian, returning its segments and whether it
/// crossed (was cut). A non-crossing ring yields a single closed segment.
fn clip_am_line(ring: &[P]) -> (Vec<Vec<P>>, bool) {
    let mut sink = Sink::default();
    sink.line_start();
    let (mut lambda0, mut phi0, mut sign0) = (f64::NAN, f64::NAN, f64::NAN);
    let mut crossed = false;

    let feed = |sink: &mut Sink, lambda1_in: f64, phi1: f64, l0: &mut f64, p0: &mut f64, s0: &mut f64, crossed: &mut bool| {
        let mut lambda1 = lambda1_in;
        let sign1 = if lambda1 > 0.0 { PI } else { -PI };
        let delta = (lambda1 - *l0).abs();
        if (delta - PI).abs() < EPSILON {
            // Crossing a pole.
            *p0 = if (*p0 + phi1) / 2.0 > 0.0 { HALF_PI } else { -HALF_PI };
            sink.point(*l0, *p0);
            sink.point(*s0, *p0);
            sink.line_end();
            sink.line_start();
            sink.point(sign1, *p0);
            sink.point(lambda1, *p0);
            *crossed = true;
        } else if *s0 != sign1 && delta >= PI {
            // Crossing the antimeridian.
            if (*l0 - *s0).abs() < EPSILON {
                *l0 -= *s0 * EPSILON;
            }
            if (lambda1 - sign1).abs() < EPSILON {
                lambda1 -= sign1 * EPSILON;
            }
            *p0 = clip_am_intersect(*l0, *p0, lambda1, phi1);
            sink.point(*s0, *p0);
            sink.line_end();
            sink.line_start();
            sink.point(sign1, *p0);
            *crossed = true;
        }
        sink.point(lambda1, phi1);
        *l0 = lambda1;
        *p0 = phi1;
        *s0 = sign1;
    };

    for &[l, p] in ring.iter().chain(std::iter::once(&ring[0])) {
        feed(&mut sink, l, p, &mut lambda0, &mut phi0, &mut sign0, &mut crossed);
    }
    sink.line_end();

    let mut segs = sink.segments;
    // Join the trailing and leading segments (they are contiguous on the ring).
    if crossed && segs.len() > 1 {
        let last = segs.pop().unwrap();
        let first = segs.remove(0);
        let mut joined = last;
        joined.extend(first);
        segs.push(joined);
    }
    (segs, crossed)
}

// --- Spherical point-in-polygon (d3-geo geoPolygonContains), for pole detection ---

fn cartesian(sph: P) -> [f64; 3] {
    let (l, p) = (sph[0], sph[1]);
    let cp = p.cos();
    [cp * l.cos(), cp * l.sin(), p.sin()]
}
fn cart_cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn cart_normalize(v: &mut [f64; 3]) {
    let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if l > 0.0 {
        v[0] /= l;
        v[1] /= l;
        v[2] /= l;
    }
}

fn polygon_contains(polygon: &[Vec<P>], point: P) -> bool {
    let lambda = point[0];
    let mut phi = point[1];
    let sin_phi = phi.sin();
    let normal = [lambda.sin(), -lambda.cos(), 0.0];
    let mut angle = 0.0;
    let mut winding = 0i32;
    let mut sum = 0.0;
    if sin_phi == 1.0 {
        phi = HALF_PI + EPSILON;
    } else if sin_phi == -1.0 {
        phi = -HALF_PI - EPSILON;
    }
    for ring in polygon {
        let m = ring.len();
        if m == 0 {
            continue;
        }
        let mut point0 = ring[m - 1];
        let mut lambda0 = point0[0];
        let phi0h = point0[1] / 2.0 + QUARTER_PI;
        let mut sin_phi0 = phi0h.sin();
        let mut cos_phi0 = phi0h.cos();
        for point1 in ring {
            let lambda1 = point1[0];
            let phi1h = point1[1] / 2.0 + QUARTER_PI;
            let sin_phi1 = phi1h.sin();
            let cos_phi1 = phi1h.cos();
            let delta = lambda1 - lambda0;
            let sign = if delta >= 0.0 { 1.0 } else { -1.0 };
            let abs_delta = sign * delta;
            let antimeridian = abs_delta > PI;
            let k = sin_phi0 * sin_phi1;
            sum += (k * sign * abs_delta.sin()).atan2(cos_phi0 * cos_phi1 + k * abs_delta.cos());
            angle += if antimeridian { delta + sign * TAU } else { delta };
            if antimeridian ^ (lambda0 >= lambda) ^ (lambda1 >= lambda) {
                let mut arc = cart_cross(cartesian(point0), cartesian(*point1));
                cart_normalize(&mut arc);
                let mut intersection = cart_cross(normal, arc);
                cart_normalize(&mut intersection);
                let phi_arc =
                    (if antimeridian ^ (delta >= 0.0) { -1.0 } else { 1.0 }) * intersection[2].asin();
                if phi > phi_arc || (phi == phi_arc && (arc[0] != 0.0 || arc[1] != 0.0)) {
                    winding += if antimeridian ^ (delta >= 0.0) { 1 } else { -1 };
                }
            }
            point0 = *point1;
            lambda0 = lambda1;
            sin_phi0 = sin_phi1;
            cos_phi0 = cos_phi1;
        }
    }
    let base = angle < -EPSILON || (angle < EPSILON && sum < -EPSILON2);
    base ^ (winding & 1 != 0)
}

// --- Boundary interpolation ---

fn interpolate(from: Option<P>, to: Option<P>, direction: f64, out: &mut Vec<P>) {
    match (from, to) {
        (None, _) => {
            let phi = direction * HALF_PI;
            out.push([-PI, phi]);
            out.push([0.0, phi]);
            out.push([PI, phi]);
            out.push([PI, 0.0]);
            out.push([PI, -phi]);
            out.push([0.0, -phi]);
            out.push([-PI, -phi]);
            out.push([-PI, 0.0]);
            out.push([-PI, phi]);
        }
        (Some(f), Some(t)) => {
            if (f[0] - t[0]).abs() > EPSILON {
                let lambda = if f[0] < t[0] { PI } else { -PI };
                let phi = direction * lambda / 2.0;
                out.push([-lambda, phi]);
                out.push([0.0, phi]);
                out.push([lambda, phi]);
            } else {
                out.push([t[0], t[1]]);
            }
        }
        (Some(_), None) => {}
    }
}

// --- Rejoin (d3-geo clipRejoin) ---

struct Isect {
    x: P,
    z: Option<usize>, // segment index (subject nodes)
    o: usize,         // paired node
    e: bool,          // entry
    v: bool,          // visited
    n: usize,
    p: usize,
}

fn compare_key(x: P) -> f64 {
    if x[0] < 0.0 {
        x[1] - HALF_PI - EPSILON
    } else {
        HALF_PI - x[1]
    }
}

fn link_ring(nodes: &mut [Isect], idxs: &[usize]) {
    let n = idxs.len();
    if n == 0 {
        return;
    }
    for k in 0..n {
        let a = idxs[k];
        let b = idxs[(k + 1) % n];
        nodes[a].n = b;
        nodes[b].p = a;
    }
}

fn clip_rejoin(
    segments: &[Vec<P>],
    start_inside: bool,
    out: &mut Vec<Vec<P>>,
) {
    let mut nodes: Vec<Isect> = Vec::new();
    let mut subject: Vec<usize> = Vec::new();
    let mut clip: Vec<usize> = Vec::new();

    for (si, seg) in segments.iter().enumerate() {
        if seg.len() <= 1 {
            continue;
        }
        let p0 = seg[0];
        let p1 = seg[seg.len() - 1];
        // subject start (entry), clip start (paired)
        let a = nodes.len();
        nodes.push(Isect { x: p0, z: Some(si), o: 0, e: true, v: false, n: 0, p: 0 });
        let ao = nodes.len();
        nodes.push(Isect { x: p0, z: None, o: a, e: false, v: false, n: 0, p: 0 });
        nodes[a].o = ao;
        subject.push(a);
        clip.push(ao);
        // subject end (exit), clip end (paired)
        let b = nodes.len();
        nodes.push(Isect { x: p1, z: Some(si), o: 0, e: false, v: false, n: 0, p: 0 });
        let bo = nodes.len();
        nodes.push(Isect { x: p1, z: None, o: b, e: true, v: false, n: 0, p: 0 });
        nodes[b].o = bo;
        subject.push(b);
        clip.push(bo);
    }

    if subject.is_empty() {
        return;
    }

    clip.sort_by(|&i, &j| compare_key(nodes[i].x).partial_cmp(&compare_key(nodes[j].x)).unwrap());
    link_ring(&mut nodes, &subject);
    link_ring(&mut nodes, &clip);

    // Alternate entry flags along the sorted boundary order.
    let mut inside = start_inside;
    for &c in &clip {
        inside = !inside;
        nodes[c].e = inside;
    }

    let start = subject[0];
    loop {
        // Find first unvisited from `start` along subject links.
        let mut current = start;
        while nodes[current].v {
            current = nodes[current].n;
            if current == start {
                return;
            }
        }
        let mut ring: Vec<P> = Vec::new();
        let mut is_subject = true;
        loop {
            let o = nodes[current].o;
            nodes[current].v = true;
            nodes[o].v = true;
            if nodes[current].e {
                if is_subject {
                    if let Some(si) = nodes[current].z {
                        for &pt in &segments[si] {
                            ring.push(pt);
                        }
                    }
                } else {
                    let to = nodes[nodes[current].n].x;
                    interpolate(Some(nodes[current].x), Some(to), 1.0, &mut ring);
                }
                current = nodes[current].n;
            } else {
                if is_subject {
                    let prev = nodes[current].p;
                    if let Some(si) = nodes[prev].z {
                        for &pt in segments[si].iter().rev() {
                            ring.push(pt);
                        }
                    }
                } else {
                    let to = nodes[nodes[current].p].x;
                    interpolate(Some(nodes[current].x), Some(to), -1.0, &mut ring);
                }
                current = nodes[current].p;
            }
            current = nodes[current].o;
            is_subject = !is_subject;
            if nodes[current].v {
                break;
            }
        }
        if ring.len() > 1 {
            out.push(ring);
        }
    }
}

/// Clip a polygon (its rings) at the antimeridian, appending closed projected
/// rings to `out_rings`.
pub fn clip_polygon(rings: &[Vec<Pos>], cm: f64, proj: &Proj, out_rings: &mut Vec<Vec<(f64, f64)>>) {
    // Normalize longitudes to the central meridian and convert to radians.
    let norm = |lon: f64| -> f64 {
        let mut d = (lon - cm) % 360.0;
        if d > 180.0 {
            d -= 360.0;
        } else if d <= -180.0 {
            d += 360.0;
        }
        d.to_radians()
    };
    let polygon: Vec<Vec<P>> = rings
        .iter()
        .map(|r| r.iter().map(|p| [norm(p.x), p.y.to_radians()]).collect())
        .collect();

    let mut segments: Vec<Vec<P>> = Vec::new();
    let mut result: Vec<Vec<P>> = Vec::new();
    for ring in &polygon {
        if ring.len() < 2 {
            continue;
        }
        let (segs, crossed) = clip_am_line(ring);
        if !crossed {
            // Clean ring: keep as-is (drop the duplicated closing point).
            if let Some(seg) = segs.into_iter().next() {
                let mut seg = seg;
                if seg.len() > 1 {
                    seg.pop();
                    result.push(seg);
                }
            }
        } else {
            for s in segs {
                if s.len() > 1 {
                    segments.push(s);
                }
            }
        }
    }

    let start_inside = polygon_contains(&polygon, [-PI, -HALF_PI]);
    if !segments.is_empty() {
        clip_rejoin(&segments, start_inside, &mut result);
    } else if start_inside {
        let mut ring = Vec::new();
        interpolate(None, None, 1.0, &mut ring);
        result.push(ring);
    }

    // Project the resulting lon/lat rings.
    for ring in &result {
        let projected: Vec<(f64, f64)> = ring
            .iter()
            .map(|p| proj.project(cm + p[0].to_degrees(), p[1].to_degrees()))
            .collect();
        if projected.len() > 1 {
            out_rings.push(projected);
        }
    }
}
