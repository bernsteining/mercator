use super::normalize_lon;

/// Maximum latitude before Mercator distortion becomes infinite.
const LAT_CLAMP: f64 = 85.05;

pub fn project(lon: f64, lat: f64, central_meridian: f64) -> (f64, f64) {
    let lat_clamped = lat.clamp(-LAT_CLAMP, LAT_CLAMP);
    let lat_rad = lat_clamped.to_radians();
    let y = (std::f64::consts::FRAC_PI_4 + lat_rad / 2.0).tan().ln()
        * (180.0 / std::f64::consts::PI);
    (normalize_lon(lon - central_meridian), -y)
}

pub fn antimeridian_gap() -> f64 {
    180.0
}
