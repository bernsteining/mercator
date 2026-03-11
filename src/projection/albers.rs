use super::{normalize_lon, Projection};

const LAT_CLAMP: f64 = 89.99;

pub(super) struct Compiled {
    n: f64,
    c: f64,
    rho0: f64,
    central_meridian: f64,
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

    Compiled { n, c, rho0, central_meridian }
}

impl Projection for Compiled {
    fn project(&self, lon: f64, lat: f64) -> (f64, f64) {
        let lat_rad = lat.clamp(-LAT_CLAMP, LAT_CLAMP).to_radians();
        let delta_lon = normalize_lon(lon - self.central_meridian);
        let theta = self.n * delta_lon.to_radians();
        let q = self.c - 2.0 * self.n * lat_rad.sin();
        let rho = if q > 0.0 && self.n.abs() > 1e-10 {
            q.sqrt() / self.n
        } else {
            0.0
        };
        let x = rho * theta.sin();
        let y = self.rho0 - rho * theta.cos();
        (x, -y)
    }
}
