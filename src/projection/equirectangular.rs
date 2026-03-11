use super::{normalize_lon, Projection};

pub struct Equirectangular {
    pub central_meridian: f64,
}

impl Projection for Equirectangular {
    fn project(&self, lon: f64, lat: f64) -> (f64, f64) {
        (normalize_lon(lon - self.central_meridian), -lat)
    }

    fn antimeridian_gap(&self) -> f64 {
        180.0
    }
}
