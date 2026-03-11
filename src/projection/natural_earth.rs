use super::{normalize_lon, Projection};

const A1: f64 = 0.8707;
const A2: f64 = -0.131979;
const A3: f64 = -0.013791;
const A4: f64 = 0.003971;
const A5: f64 = -0.001529;

const B1: f64 = 1.007226;
const B2: f64 = 0.015085;
const B3: f64 = -0.044475;
const B4: f64 = 0.028874;
const B5: f64 = -0.005916;

pub struct NaturalEarth {
    pub central_meridian: f64,
}

impl Projection for NaturalEarth {
    fn project(&self, lon: f64, lat: f64) -> (f64, f64) {
        let phi = lat.clamp(-90.0, 90.0).to_radians();
        let lambda = normalize_lon(lon - self.central_meridian).to_radians();
        let phi2 = phi * phi;
        let phi4 = phi2 * phi2;
        let x = lambda * (A1 + A2 * phi2 + phi4 * (A3 + phi4 * (A4 * phi2 + A5 * phi4)));
        let y = phi * (B1 + phi2 * (B2 + phi4 * (B3 + B4 * phi2 + B5 * phi4)));
        (x, -y)
    }

    fn antimeridian_gap(&self) -> f64 {
        3.0
    }
}
