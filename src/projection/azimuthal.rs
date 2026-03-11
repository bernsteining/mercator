/// Precomputed center-point data shared by all azimuthal projections.
pub struct AzimuthalCompiled {
    pub sin_center: f64,
    pub cos_center: f64,
    pub center_lon: f64,
}

pub fn compile(center_lat: f64, center_lon: f64) -> AzimuthalCompiled {
    let phi0 = center_lat.to_radians();
    AzimuthalCompiled {
        sin_center: phi0.sin(),
        cos_center: phi0.cos(),
        center_lon,
    }
}

/// Precomputed trig values for projecting a single point.
pub struct AzimuthalPoint {
    pub sin_phi: f64,
    pub cos_phi: f64,
    pub sin_dl: f64,
    pub cos_dl: f64,
    /// Cosine of angular distance from center.
    pub cos_c: f64,
}

pub fn intermediates(lon: f64, lat: f64, c: &AzimuthalCompiled) -> AzimuthalPoint {
    let phi = lat.to_radians();
    let delta_lon = (lon - c.center_lon).to_radians();

    let sin_phi = phi.sin();
    let cos_phi = phi.cos();
    let cos_dl = delta_lon.cos();
    let sin_dl = delta_lon.sin();
    let cos_c = c.sin_center * sin_phi + c.cos_center * cos_phi * cos_dl;

    AzimuthalPoint { sin_phi, cos_phi, sin_dl, cos_dl, cos_c }
}
