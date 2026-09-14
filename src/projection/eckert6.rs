use super::{normalize_lon, Projection};
use std::f64::consts::PI;

/// Eckert VI: an equal-area pseudocylindrical projection with sinusoidal meridians.
/// The auxiliary angle θ solves `θ + sin θ = (1 + π/2)·sin φ` (Newton–Raphson).
pub struct Eckert6 {
    pub central_meridian: f64,
}

fn theta(phi: f64) -> f64 {
    let target = (1.0 + PI / 2.0) * phi.sin();
    let mut t = phi;
    for _ in 0..12 {
        let dt = (t + t.sin() - target) / (1.0 + t.cos());
        t -= dt;
        if dt.abs() < 1e-10 {
            break;
        }
    }
    t
}

impl Projection for Eckert6 {
    fn project(&self, lon: f64, lat: f64) -> (f64, f64) {
        let phi = lat.to_radians();
        let lambda = normalize_lon(lon - self.central_meridian).to_radians();
        let t = theta(phi);
        let k = (2.0 + PI).sqrt();
        (lambda * (1.0 + t.cos()) / k, -2.0 * t / k)
    }

    fn antimeridian_gap(&self) -> f64 {
        2.0
    }
}
