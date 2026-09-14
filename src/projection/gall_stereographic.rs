use super::{normalize_lon, Projection};
use std::f64::consts::SQRT_2;

/// Gall stereographic: a compromise cylindrical projection (a Braun-family
/// stereographic cylindrical). x = λ/√2, y = (1 + √2/2)·tan(φ/2).
pub struct GallStereographic {
    pub central_meridian: f64,
}

impl Projection for GallStereographic {
    fn project(&self, lon: f64, lat: f64) -> (f64, f64) {
        let phi = lat.clamp(-89.999, 89.999).to_radians();
        let lambda = normalize_lon(lon - self.central_meridian).to_radians();
        let x = lambda / SQRT_2;
        let y = (1.0 + SQRT_2 / 2.0) * (phi / 2.0).tan();
        (x, -y)
    }

    fn antimeridian_gap(&self) -> f64 {
        2.0
    }

    fn antimeridian_center(&self) -> Option<f64> {
        Some(self.central_meridian)
    }
}
