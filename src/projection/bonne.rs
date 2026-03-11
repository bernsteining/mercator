use super::{prepare_lon_lat, Projection};

pub struct Bonne {
    pub central_meridian: f64,
    pub standard_parallel: f64,
}

impl Projection for Bonne {
    fn project(&self, lon: f64, lat: f64) -> (f64, f64) {
        let (lambda, phi) = prepare_lon_lat(lon, lat, self.central_meridian);
        let phi1 = self.standard_parallel.to_radians();

        let sin_phi1 = phi1.sin();
        if sin_phi1.abs() < 1e-10 {
            // Degenerate: standard_parallel ≈ 0 → sinusoidal projection
            let x = lambda * phi.cos();
            return (x, -phi);
        }
        let cot_phi1 = phi1.cos() / sin_phi1;
        let rho = cot_phi1 + phi1 - phi;

        if rho.abs() < 1e-10 {
            return (0.0, 0.0);
        }

        let e = lambda * phi.cos() / rho;
        let x = rho * e.sin();
        let y = cot_phi1 - rho * e.cos();

        (x, -y)
    }

    fn antimeridian_gap(&self) -> f64 {
        0.3
    }
}
