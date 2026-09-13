use super::azimuthal::{AzimuthalCompiled, intermediates};
use super::Projection;

pub struct Orthographic(pub AzimuthalCompiled);

impl Projection for Orthographic {
    fn project(&self, lon: f64, lat: f64) -> (f64, f64) {
        let p = intermediates(lon, lat, &self.0);

        if p.cos_c < 0.0 {
            return (f64::NAN, f64::NAN);
        }

        let x = p.cos_phi * p.sin_dl;
        let y = self.0.cos_center * p.sin_phi - self.0.sin_center * p.cos_phi * p.cos_dl;

        (x, -y)
    }

    fn clip_center(&self) -> Option<[f64; 3]> {
        let c = &self.0;
        let lon = c.center_lon.to_radians();
        Some([c.cos_center * lon.cos(), c.cos_center * lon.sin(), c.sin_center])
    }
}
