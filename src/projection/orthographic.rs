pub struct Compiled {
    pub sin_center: f64,
    pub cos_center: f64,
    pub center_lon: f64,
}

pub fn compile(center_lat: f64, center_lon: f64) -> Compiled {
    let phi0 = center_lat.to_radians();
    Compiled {
        sin_center: phi0.sin(),
        cos_center: phi0.cos(),
        center_lon,
    }
}

pub fn project(lon: f64, lat: f64, c: &Compiled) -> (f64, f64) {
    let lat_rad = lat.to_radians();
    let delta_lon = (lon - c.center_lon).to_radians();
    let cos_c =
        c.sin_center * lat_rad.sin() + c.cos_center * lat_rad.cos() * delta_lon.cos();
    if cos_c < 0.0 {
        return (f64::NAN, f64::NAN);
    }
    let x = lat_rad.cos() * delta_lon.sin();
    let y = c.cos_center * lat_rad.sin() - c.sin_center * lat_rad.cos() * delta_lon.cos();
    (x, -y)
}
