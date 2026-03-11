use super::azimuthal::{AzimuthalCompiled, intermediates};
use super::Projection;

pub struct Wiechel(pub AzimuthalCompiled);

impl Projection for Wiechel {
    fn project(&self, lon: f64, lat: f64) -> (f64, f64) {
        let p = intermediates(lon, lat, &self.0);

        if p.cos_c < 0.0 {
            return (f64::NAN, f64::NAN);
        }

        let xo = p.cos_phi * p.sin_dl;
        let yo = self.0.cos_center * p.sin_phi - self.0.sin_center * p.cos_phi * p.cos_dl;

        let r_orth = (xo * xo + yo * yo).sqrt();
        if r_orth < 1e-10 {
            return (0.0, 0.0);
        }
        let theta = yo.atan2(xo);

        let r_lambert = (2.0 * (1.0 - p.cos_c)).sqrt();

        let swirl = -(1.0 - p.cos_c);
        let theta_w = theta + swirl;

        let x = r_lambert * theta_w.cos();
        let y = r_lambert * theta_w.sin();

        (x, -y)
    }
}
