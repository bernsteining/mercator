use super::{prepare_lon_lat, Projection};

pub struct WinkelTripel {
    pub central_meridian: f64,
    /// Precomputed cos(standard_parallel) — avoids recomputing acos(2/π) + cos per point.
    pub cos_phi1: f64,
}

impl WinkelTripel {
    pub fn new(central_meridian: f64) -> Self {
        let phi1 = (2.0 / std::f64::consts::PI).acos();
        Self { central_meridian, cos_phi1: phi1.cos() }
    }
}

impl Projection for WinkelTripel {
    fn project(&self, lon: f64, lat: f64) -> (f64, f64) {
        let (lambda, phi) = prepare_lon_lat(lon, lat, self.central_meridian);

        let cos_phi = phi.cos();

        let x_equi = lambda * self.cos_phi1;
        let y_equi = phi;

        let alpha = (cos_phi * (lambda / 2.0).cos()).clamp(-1.0, 1.0).acos();
        let (x_aitoff, y_aitoff) = if alpha.abs() < 1e-10 {
            (0.0, 0.0)
        } else {
            let sinc_alpha = alpha.sin() / alpha;
            (
                2.0 * cos_phi * (lambda / 2.0).sin() / sinc_alpha,
                phi.sin() / sinc_alpha,
            )
        };

        let x = (x_equi + x_aitoff) / 2.0;
        let y = (y_equi + y_aitoff) / 2.0;

        (x, -y)
    }

    fn antimeridian_gap(&self) -> f64 {
        3.0
    }
}
