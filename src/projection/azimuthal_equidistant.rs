use super::azimuthal::{AzimuthalCompiled, intermediates};
use super::Projection;

pub struct AzimuthalEquidistant(pub AzimuthalCompiled);

impl Projection for AzimuthalEquidistant {
    fn project(&self, lon: f64, lat: f64) -> (f64, f64) {
        let p = intermediates(lon, lat, &self.0);

        let cos_c = p.cos_c.clamp(-1.0, 1.0);
        let ang = cos_c.acos();

        if ang < 1e-10 {
            return (0.0, 0.0);
        }

        let sin_ang = ang.sin();
        if sin_ang.abs() < 1e-10 {
            return (f64::NAN, f64::NAN);
        }

        let k = ang / sin_ang;

        let x = k * p.cos_phi * p.sin_dl;
        let y = k * (self.0.cos_center * p.sin_phi - self.0.sin_center * p.cos_phi * p.cos_dl);

        (x, -y)
    }
}
