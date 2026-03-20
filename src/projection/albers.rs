use super::{normalize_lon, Projection};
use std::f64::consts::PI;

const LAT_CLAMP: f64 = 89.99;

pub(crate) struct Compiled {
    n: f64,
    two_n: f64,
    inv_n: f64,
    c: f64,
    rho0: f64,
    central_meridian: f64,
    gap: f64,
}

pub fn compile(
    standard_parallel_1: f64,
    standard_parallel_2: f64,
    central_meridian: f64,
    latitude_of_origin: f64,
) -> Compiled {
    let phi1 = standard_parallel_1.to_radians();
    let phi2 = standard_parallel_2.to_radians();
    let phi0 = latitude_of_origin.to_radians();

    let n = (phi1.sin() + phi2.sin()) / 2.0;
    let c = phi1.cos().powi(2) + 2.0 * n * phi1.sin();
    let rho0 = if n.abs() < 1e-10 {
        0.0
    } else {
        (c - 2.0 * n * phi0.sin()).max(0.0).sqrt() / n
    };

    let inv_n = if n.abs() < 1e-10 { 0.0 } else { 1.0 / n };
    // Antimeridian jump at equator: 2 * rho_eq * |sin(n*π)|; use half as gap threshold
    let rho_eq = if n.abs() < 1e-10 { 0.0 } else { c.max(0.0).sqrt() / n };
    let gap = rho_eq.abs() * (n * PI).sin().abs();
    Compiled { n, two_n: 2.0 * n, inv_n, c, rho0, central_meridian, gap }
}

impl Projection for Compiled {
    fn antimeridian_gap(&self) -> f64 {
        self.gap
    }

    fn project(&self, lon: f64, lat: f64) -> (f64, f64) {
        let lat_rad = lat.clamp(-LAT_CLAMP, LAT_CLAMP).to_radians();
        let delta_lon = normalize_lon(lon - self.central_meridian);
        let theta = self.n * delta_lon.to_radians();
        let q = self.c - self.two_n * lat_rad.sin();
        let rho = if q > 0.0 {
            q.sqrt() * self.inv_n
        } else {
            0.0
        };
        let x = rho * theta.sin();
        let y = self.rho0 - rho * theta.cos();
        (x, -y)
    }
}
