use super::{normalize_lon, Projection};

/// Sinusoidal (Sanson–Flamsteed): an equal-area pseudocylindrical projection.
/// x = λ·cos φ, y = φ.
pub struct Sinusoidal {
    pub central_meridian: f64,
}

impl Projection for Sinusoidal {
    fn project(&self, lon: f64, lat: f64) -> (f64, f64) {
        let phi = lat.to_radians();
        let lambda = normalize_lon(lon - self.central_meridian).to_radians();
        (lambda * phi.cos(), -phi)
    }

    fn antimeridian_gap(&self) -> f64 {
        3.0
    }
}
