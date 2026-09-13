use super::{normalize_lon, Projection};

/// Aitoff: a lenticular compromise projection (the basis of Winkel tripel).
/// α = acos(cos φ · cos(λ/2)); x = 2·cos φ·sin(λ/2)/sinc α, y = sin φ/sinc α.
pub struct Aitoff {
    pub central_meridian: f64,
}

/// sin(x)/x, guarded at 0.
fn sinci(x: f64) -> f64 {
    if x == 0.0 {
        1.0
    } else {
        x.sin() / x
    }
}

impl Projection for Aitoff {
    fn project(&self, lon: f64, lat: f64) -> (f64, f64) {
        let phi = lat.to_radians();
        let lambda = normalize_lon(lon - self.central_meridian).to_radians();
        let cos_phi = phi.cos();
        let alpha = (cos_phi * (lambda / 2.0).cos()).clamp(-1.0, 1.0).acos();
        let sc = sinci(alpha);
        let x = 2.0 * cos_phi * (lambda / 2.0).sin() / sc;
        let y = phi.sin() / sc;
        (x, -y)
    }

    fn antimeridian_gap(&self) -> f64 {
        2.0
    }
}
