use super::{normalize_lon, Projection};
use std::f64::consts::SQRT_2;

/// Gall–Peters: the cylindrical equal-area projection with standard parallels at
/// 45°. x = λ·cos45° = λ/√2, y = sin φ / cos45° = √2·sin φ.
pub struct GallPeters {
    pub central_meridian: f64,
}

impl Projection for GallPeters {
    fn project(&self, lon: f64, lat: f64) -> (f64, f64) {
        let phi = lat.to_radians();
        let lambda = normalize_lon(lon - self.central_meridian).to_radians();
        (lambda / SQRT_2, -SQRT_2 * phi.sin())
    }

    fn antimeridian_gap(&self) -> f64 {
        2.0
    }

    fn antimeridian_center(&self) -> Option<f64> {
        Some(self.central_meridian)
    }
}
