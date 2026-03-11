use super::azimuthal::{AzimuthalCompiled, intermediates};
use super::Projection;

pub struct LambertAzimuthal(pub AzimuthalCompiled);

impl Projection for LambertAzimuthal {
    fn project(&self, lon: f64, lat: f64) -> (f64, f64) {
        let p = intermediates(lon, lat, &self.0);

        if p.cos_c < -0.999 {
            return (f64::NAN, f64::NAN);
        }

        let k = (2.0 / (1.0 + p.cos_c)).sqrt();
        let x = k * p.cos_phi * p.sin_dl;
        let y = k * (self.0.cos_center * p.sin_phi - self.0.sin_center * p.cos_phi * p.cos_dl);

        (x, -y)
    }
}
