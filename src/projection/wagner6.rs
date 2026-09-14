use super::{normalize_lon, Projection};
use std::f64::consts::PI;

/// Wagner VI: a compromise pseudocylindrical projection.
/// x = λ·√(1 − 3(φ/π)²), y = φ.
pub struct Wagner6 {
    pub central_meridian: f64,
}

impl Projection for Wagner6 {
    fn project(&self, lon: f64, lat: f64) -> (f64, f64) {
        let phi = lat.to_radians();
        let lambda = normalize_lon(lon - self.central_meridian).to_radians();
        let x = lambda * (1.0 - 3.0 * (phi / PI).powi(2)).max(0.0).sqrt();
        (x, -phi)
    }

    fn antimeridian_gap(&self) -> f64 {
        3.0
    }
}
