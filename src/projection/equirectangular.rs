pub fn project(lon: f64, lat: f64) -> (f64, f64) {
    (lon, -lat)
}

pub fn antimeridian_gap() -> f64 {
    180.0
}
