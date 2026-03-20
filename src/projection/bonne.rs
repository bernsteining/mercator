use super::{prepare_lon_lat, Projection};

pub struct Bonne {
    pub central_meridian: f64,
    phi1: f64,
    cot_phi1: f64,
    degenerate: bool,
}

impl Bonne {
    pub fn new(central_meridian: f64, standard_parallel: f64) -> Self {
        let phi1 = standard_parallel.to_radians();
        let sin_phi1 = phi1.sin();
        let degenerate = sin_phi1.abs() < 1e-10;
        let cot_phi1 = if degenerate { 0.0 } else { phi1.cos() / sin_phi1 };
        Self { central_meridian, phi1, cot_phi1, degenerate }
    }
}

impl Projection for Bonne {
    fn project(&self, lon: f64, lat: f64) -> (f64, f64) {
        let (lambda, phi) = prepare_lon_lat(lon, lat, self.central_meridian);

        if self.degenerate {
            let x = lambda * phi.cos();
            return (x, -phi);
        }
        let rho = self.cot_phi1 + self.phi1 - phi;

        if rho.abs() < 1e-10 {
            return (0.0, 0.0);
        }

        let e = lambda * phi.cos() / rho;
        let x = rho * e.sin();
        let y = self.cot_phi1 - rho * e.cos();

        (x, -y)
    }

    fn antimeridian_gap(&self) -> f64 {
        0.3
    }
}
