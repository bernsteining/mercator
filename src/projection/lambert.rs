use super::normalize_lon;

/// Latitude clamp to avoid singularities at the poles.
const LAT_CLAMP: f64 = 89.99;

pub struct Compiled {
    pub n: f64,
    pub f: f64,
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

    let n = if (phi1 - phi2).abs() < 1e-10 {
        phi1.sin()
    } else {
        (phi1.cos().ln() - phi2.cos().ln())
            / ((std::f64::consts::FRAC_PI_4 + phi2 / 2.0).tan().ln()
                - (std::f64::consts::FRAC_PI_4 + phi1 / 2.0).tan().ln())
    };

    let f = phi1.cos()
        * (std::f64::consts::FRAC_PI_4 + phi1 / 2.0)
            .tan()
            .powf(n)
        / n;
    let rho0 = f
        / (std::f64::consts::FRAC_PI_4 + phi0 / 2.0)
            .tan()
            .powf(n);

    Compiled { n, f, rho0, central_meridian }
}

pub fn project(lon: f64, lat: f64, c: &Compiled) -> (f64, f64) {
    let lat_rad = lat.clamp(-LAT_CLAMP, LAT_CLAMP).to_radians();
    let delta_lon = normalize_lon(lon - c.central_meridian);
    let theta = c.n * delta_lon.to_radians();
    let rho = c.f
        / (std::f64::consts::FRAC_PI_4 + lat_rad / 2.0)
            .tan()
            .powf(c.n);
    let x = rho * theta.sin();
    let y = c.rho0 - rho * theta.cos();
    (x, -y)
}
