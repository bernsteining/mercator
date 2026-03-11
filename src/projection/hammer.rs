use super::{prepare_lon_lat, Projection};

pub struct Hammer {
    pub central_meridian: f64,
}

impl Projection for Hammer {
    fn project(&self, lon: f64, lat: f64) -> (f64, f64) {
        let (lambda, phi) = prepare_lon_lat(lon, lat, self.central_meridian);

        let cos_phi = phi.cos();
        let half_lambda = lambda / 2.0;
        let z = (1.0 + cos_phi * half_lambda.cos()).max(0.0).sqrt();

        if z.abs() < 1e-10 {
            return (0.0, 0.0);
        }

        let x = 2.0 * std::f64::consts::SQRT_2 * cos_phi * half_lambda.sin() / z;
        let y = std::f64::consts::SQRT_2 * phi.sin() / z;

        (x, -y)
    }

    fn antimeridian_gap(&self) -> f64 {
        3.0
    }
}
