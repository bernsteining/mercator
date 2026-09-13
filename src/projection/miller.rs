use super::{normalize_lon, Projection};

/// Miller cylindrical: a compromise projection that tempers Mercator's polar
/// exaggeration. x = λ, y = 1.25·asinh(tan(0.8·φ)) = 1.25·ln(tan(π/4 + 0.4φ)).
pub struct Miller {
    pub central_meridian: f64,
}

impl Projection for Miller {
    fn project(&self, lon: f64, lat: f64) -> (f64, f64) {
        let phi = lat.clamp(-89.999, 89.999).to_radians();
        let lambda = normalize_lon(lon - self.central_meridian).to_radians();
        let y = 1.25 * (0.8 * phi).tan().asinh();
        (lambda, -y)
    }

    fn antimeridian_gap(&self) -> f64 {
        3.0
    }

    fn antimeridian_center(&self) -> Option<f64> {
        Some(self.central_meridian)
    }
}
