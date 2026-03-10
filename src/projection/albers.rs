use super::normalize_lon;

/// Latitude clamp to avoid singularities at the poles.
const LAT_CLAMP: f64 = 89.99;

pub struct Compiled {
    pub n: f64,
    pub c: f64,
    pub rho0: f64,
    pub central_meridian: f64,
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
    let rho0 = (c - 2.0 * n * phi0.sin()).sqrt() / n;

    Compiled { n, c, rho0, central_meridian }
}

pub fn project(lon: f64, lat: f64, comp: &Compiled) -> (f64, f64) {
    let lat_rad = lat.clamp(-LAT_CLAMP, LAT_CLAMP).to_radians();
    let delta_lon = normalize_lon(lon - comp.central_meridian);
    let theta = comp.n * delta_lon.to_radians();
    let q = comp.c - 2.0 * comp.n * lat_rad.sin();
    let rho = if q > 0.0 { q.sqrt() / comp.n } else { 0.0 };
    let x = rho * theta.sin();
    let y = comp.rho0 - rho * theta.cos();
    (x, -y)
}
