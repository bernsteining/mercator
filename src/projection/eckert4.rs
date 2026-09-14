use super::{normalize_lon, Projection};
use std::f64::consts::PI;

/// Eckert IV: an equal-area pseudocylindrical projection with a straight pole
/// line half the equator's length. The auxiliary angle θ solves
/// `θ + sin θ cos θ + 2 sin θ = (2 + π/2) sin φ` (Newton–Raphson).
pub struct Eckert4 {
    pub central_meridian: f64,
}

fn theta(phi: f64) -> f64 {
    let target = (2.0 + PI / 2.0) * phi.sin();
    let mut t = phi / 2.0;
    for _ in 0..12 {
        let (s, c) = (t.sin(), t.cos());
        let f = t + s * c + 2.0 * s - target;
        let fp = 1.0 + (2.0 * t).cos() + 2.0 * c;
        let dt = f / fp;
        t -= dt;
        if dt.abs() < 1e-10 {
            break;
        }
    }
    t
}

impl Projection for Eckert4 {
    fn project(&self, lon: f64, lat: f64) -> (f64, f64) {
        let phi = lat.to_radians();
        let lambda = normalize_lon(lon - self.central_meridian).to_radians();
        let t = theta(phi);
        let c1 = 2.0 / (PI * (4.0 + PI)).sqrt();
        let c2 = 2.0 * (PI / (4.0 + PI)).sqrt();
        (c1 * lambda * (1.0 + t.cos()), -c2 * t.sin())
    }

    fn antimeridian_gap(&self) -> f64 {
        2.0
    }
}
