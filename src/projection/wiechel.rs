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

        let r_orth_sq = xo * xo + yo * yo;
        if r_orth_sq < 1e-20 {
            return (0.0, 0.0);
        }
        let r_orth = r_orth_sq.sqrt();

        let one_minus_cos_c = 1.0 - p.cos_c;
        let r_lambert = (2.0 * one_minus_cos_c).sqrt();

        // Use angle addition: cos/sin(theta + swirl) where cos_theta = xo/r_orth, sin_theta = yo/r_orth
        // This avoids atan2() by using the unit vector directly
        let swirl = -one_minus_cos_c;
        let cos_s = swirl.cos();
        let sin_s = swirl.sin();
        let inv_r = 1.0 / r_orth;
        let ct = xo * inv_r; // cos(theta)
        let st = yo * inv_r; // sin(theta)
        let x = r_lambert * (ct * cos_s - st * sin_s);
        let y = r_lambert * (st * cos_s + ct * sin_s);

        (x, -y)
    }
}
