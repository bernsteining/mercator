use super::{prepare_lon_lat, Projection};

pub struct Polyconic {
    pub central_meridian: f64,
}

impl Projection for Polyconic {
    fn project(&self, lon: f64, lat: f64) -> (f64, f64) {
        let (lambda, phi) = prepare_lon_lat(lon, lat, self.central_meridian);

        if phi.abs() < 1e-10 {
            return (lambda, 0.0);
        }

        let sin_phi = phi.sin();
        let cot_phi = phi.cos() / sin_phi;
        let e = lambda * sin_phi;
        let x = cot_phi * e.sin();
        let y = phi + cot_phi * (1.0 - e.cos());

        (x, -y)
    }

    fn antimeridian_gap(&self) -> f64 {
        0.3
    }
}
