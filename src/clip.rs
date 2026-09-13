//! Clip geometry to the visible hemisphere of an azimuthal projection (the globe
//! "limb"), re-stitching cut rings along the limb arc so filled polygons stay
//! closed — the correct alternative to the per-vertex NaN/gap heuristics.
//!
//! Clipping happens on the sphere (using 3D unit vectors) because in projected
//! orthographic space the near and far hemispheres overlap; visibility is decided
//! by the sign of `v · center`, and edge/limb intersections are found via the
//! intersection of the edge's great circle with the limb great circle.

use crate::model::Pos;
use crate::projection::Proj;

/// Degrees of arc per stitched segment along the limb.
const ARC_STEP_DEG: f64 = 2.0;

/// How geometry is clipped to the projection's domain.
pub enum Clip {
    /// Visible-hemisphere clip for azimuthal (globe) projections.
    Circle(ClipCircle),
    /// Antimeridian clip for cylindrical/pseudocylindrical projections.
    Antimeridian { central_meridian: f64 },
}

fn ll_to_vec(lon: f64, lat: f64) -> [f64; 3] {
    let (lo, la) = (lon.to_radians(), lat.to_radians());
    let cl = la.cos();
    [cl * lo.cos(), cl * lo.sin(), la.sin()]
}

fn vec_to_ll(v: [f64; 3]) -> (f64, f64) {
    (
        v[1].atan2(v[0]).to_degrees(),
        v[2].clamp(-1.0, 1.0).asin().to_degrees(),
    )
}

fn dot(a: &[f64; 3], b: &[f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: &[f64; 3], b: &[f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn normalize(v: [f64; 3]) -> [f64; 3] {
    let m = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if m > 0.0 {
        [v[0] / m, v[1] / m, v[2] / m]
    } else {
        v
    }
}

/// The visible-hemisphere clip of an azimuthal projection, centered on `center`
/// (a unit vector). The limb is the great circle `{ v : v · center = 0 }`.
pub struct ClipCircle {
    center: [f64; 3],
}

impl ClipCircle {
    pub fn new(center: [f64; 3]) -> Self {
        ClipCircle { center }
    }

    #[inline]
    fn visible(&self, v: &[f64; 3]) -> bool {
        dot(v, &self.center) >= 0.0
    }

    /// Project a point that lies on the limb (`v · center ≈ 0`). Nudged a hair
    /// toward the center first, so the projection doesn't hit its `cos_c < 0`
    /// backface guard and return NaN.
    fn project_limb(&self, v: [f64; 3], proj: &Proj) -> (f64, f64) {
        const EPS: f64 = 1e-6;
        let p = normalize([
            v[0] + self.center[0] * EPS,
            v[1] + self.center[1] * EPS,
            v[2] + self.center[2] * EPS,
        ]);
        let (lon, lat) = vec_to_ll(p);
        proj.project(lon, lat)
    }

    /// Unit vector where the great-circle arc `a→b` crosses the limb.
    fn intersect(&self, a: &[f64; 3], b: &[f64; 3]) -> [f64; 3] {
        // Intersection line of the two planes (edge plane normal a×b, limb plane
        // normal = center): direction (a×b)×center; the crossing is ±this.
        let d = normalize(cross(&cross(a, b), &self.center));
        let neg = [-d[0], -d[1], -d[2]];
        // Pick the candidate lying on the short arc between a and b.
        let mid = [a[0] + b[0], a[1] + b[1], a[2] + b[2]];
        if dot(&d, &mid) >= dot(&neg, &mid) {
            d
        } else {
            neg
        }
    }

    /// The projected disc center and radius for drawing/bounding the sphere.
    pub fn disc(&self, proj: &Proj) -> ((f64, f64), f64) {
        let (clon, clat) = vec_to_ll(self.center);
        let center = proj.project(clon, clat);
        // A point on the limb: perpendicular to center within the sphere.
        let up = if self.center[2].abs() < 0.9 {
            [0.0, 0.0, 1.0]
        } else {
            [1.0, 0.0, 0.0]
        };
        let limb = normalize(cross(&self.center, &up));
        let (lx, ly) = self.project_limb(limb, proj);
        let r = ((lx - center.0).powi(2) + (ly - center.1).powi(2)).sqrt();
        (center, r)
    }
}

/// Stitch an arc along the limb from `from` to `to` (both unit vectors on the
/// limb), pushing the intermediate projected points (endpoints excluded).
fn stitch_arc(out: &mut Vec<(f64, f64)>, from: [f64; 3], to: [f64; 3], clip: &ClipCircle, proj: &Proj) {
    let u = from;
    let w = normalize(cross(&clip.center, &u));
    // Angle of `to` in the (u, w) basis; take the short way around.
    let ang = dot(&to, &w).atan2(dot(&to, &u));
    let steps = ((ang.abs().to_degrees() / ARC_STEP_DEG).ceil() as usize).max(1);
    for k in 1..steps {
        let t = ang * (k as f64 / steps as f64);
        let (c, s) = (t.cos(), t.sin());
        let v = [
            u[0] * c + w[0] * s,
            u[1] * c + w[1] * s,
            u[2] * c + w[2] * s,
        ];
        out.push(clip.project_limb(v, proj));
    }
}

/// Clip one polygon ring against the hemisphere, appending the projected clipped
/// ring (closed, with limb arcs) to `out`. Returns true if anything was emitted.
pub fn clip_polygon_ring(ring: &[Pos], clip: &ClipCircle, proj: &Proj, out: &mut Vec<(f64, f64)>) -> bool {
    let n = ring.len();
    if n < 2 {
        return false;
    }
    let vecs: Vec<[f64; 3]> = ring.iter().map(|p| ll_to_vec(p.x, p.y)).collect();
    let vis: Vec<bool> = vecs.iter().map(|v| clip.visible(v)).collect();

    if vis.iter().all(|&b| b) {
        for p in ring {
            out.push(proj.project(p.x, p.y));
        }
        return true;
    }
    if vis.iter().all(|&b| !b) {
        return false;
    }

    let start = out.len();
    let mut pending_exit: Option<[f64; 3]> = None;
    let mut first_entry: Option<[f64; 3]> = None;

    for i in 0..n {
        let j = (i + 1) % n;
        let (sv, ev) = (vis[i], vis[j]);
        if ev {
            if !sv {
                // Entering the hemisphere.
                let entry = clip.intersect(&vecs[i], &vecs[j]);
                match pending_exit.take() {
                    Some(exit) => stitch_arc(out, exit, entry, clip, proj),
                    None => first_entry = Some(entry),
                }
                out.push(clip.project_limb(entry, proj));
            }
            out.push(proj.project(ring[j].x, ring[j].y));
        } else if sv {
            // Exiting the hemisphere.
            let exit = clip.intersect(&vecs[i], &vecs[j]);
            out.push(clip.project_limb(exit, proj));
            pending_exit = Some(exit);
        }
    }

    // Close across the seam (ring began outside the hemisphere).
    if let (Some(exit), Some(entry)) = (pending_exit, first_entry) {
        stitch_arc(out, exit, entry, clip, proj);
    }
    out.len() > start
}

