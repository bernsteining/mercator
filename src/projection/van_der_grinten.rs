use super::{normalize_lon, Projection};
use std::f64::consts::PI;

/// Van der Grinten I: a compromise projection that maps the whole sphere into a
/// circle. Straight port of Snyder's forward equations (sphere, unit radius).
pub struct VanDerGrinten {
    pub central_meridian: f64,
}

impl Projection for VanDerGrinten {
    fn project(&self, lon: f64, lat: f64) -> (f64, f64) {
        let lambda = normalize_lon(lon - self.central_meridian).to_radians();
        let phi = lat.to_radians();
        // Degenerate rows/columns.
        if phi.abs() < 1e-12 {
            return (lambda, 0.0);
        }
        let theta = (2.0 * phi.abs() / PI).clamp(0.0, 1.0).asin();
        if lambda.abs() < 1e-12 || (phi.abs() - PI / 2.0).abs() < 1e-12 {
            let y = PI * (theta / 2.0).tan();
            return (0.0, -y.copysign(phi));
        }
        let a = 0.5 * (PI / lambda - lambda / PI).abs();
        let sin_t = theta.sin();
        let cos_t = theta.cos();
        let g = cos_t / (sin_t + cos_t - 1.0);
        let p = g * (2.0 / sin_t - 1.0);
        let a2 = a * a;
        let p2 = p * p;
        let g2 = g * g;
        let q = a2 + g;
        let denom = p2 + a2;
        let x = PI * lambda.signum()
            * (a * (g - p2) + (a2 * (g - p2).powi(2) - (p2 + a2) * (g2 - p2)).max(0.0).sqrt())
            / denom;
        let y = PI * phi.signum()
            * (p * q - a * ((a2 + 1.0) * denom - q * q).max(0.0).sqrt())
            / denom;
        (x, -y)
    }

    fn antimeridian_gap(&self) -> f64 {
        3.0
    }
}
