use super::{normalize_lon, Projection};
use std::f64::consts::{PI, SQRT_2};

/// Mollweide (Babinet): an equal-area pseudocylindrical projection whose meridians
/// are ellipse arcs, mapping the sphere into a 2:1 ellipse. The auxiliary angle θ
/// solves `2θ + sin 2θ = π sin φ` (Newton–Raphson).
pub struct Mollweide {
    pub central_meridian: f64,
}

fn theta(phi: f64) -> f64 {
    // Poles are exact; elsewhere iterate to convergence.
    if (phi.abs() - PI / 2.0).abs() < 1e-9 {
        return phi;
    }
    let mut t = phi;
    let target = PI * phi.sin();
    for _ in 0..12 {
        let dt = (2.0 * t + (2.0 * t).sin() - target) / (2.0 + 2.0 * (2.0 * t).cos());
        t -= dt;
        if dt.abs() < 1e-10 {
            break;
        }
    }
    t
}

impl Projection for Mollweide {
    fn project(&self, lon: f64, lat: f64) -> (f64, f64) {
        let phi = lat.to_radians();
        let lambda = normalize_lon(lon - self.central_meridian).to_radians();
        let t = theta(phi);
        let x = (2.0 * SQRT_2 / PI) * lambda * t.cos();
        let y = SQRT_2 * t.sin();
        (x, -y)
    }

    fn antimeridian_gap(&self) -> f64 {
        2.0
    }
}
