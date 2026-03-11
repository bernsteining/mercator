use super::{prepare_lon_lat, Projection};

pub struct Cassini {
    pub central_meridian: f64,
}

impl Projection for Cassini {
    fn project(&self, lon: f64, lat: f64) -> (f64, f64) {
        let (lambda, phi) = prepare_lon_lat(lon, lat, self.central_meridian);
        let x = (phi.cos() * lambda.sin()).clamp(-1.0, 1.0).asin();
        let y = phi.sin().atan2(phi.cos() * lambda.cos());
        (x, -y)
    }

    fn antimeridian_gap(&self) -> f64 {
        2.0
    }
}
