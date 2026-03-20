use crate::geometry::{push_f64, PathBuilder};
use crate::projection::Proj;
use crate::style::TissotConfig;

const CIRCLE_SEGMENTS: usize = 36;

pub fn render(proj: &Proj, config: &TissotConfig, max_gap: f64) -> String {
    let mut svg = String::new();
    let radius_rad = config.radius.to_radians();
    let sin_r = radius_rad.sin();
    let cos_r = radius_rad.cos();

    // Precompute per-segment trig values — identical for every circle
    let mut seg_sin_r_sin = [0.0f64; CIRCLE_SEGMENTS + 1];
    let mut seg_sin_r_cos = [0.0f64; CIRCLE_SEGMENTS + 1];
    for i in 0..=CIRCLE_SEGMENTS {
        let az = std::f64::consts::TAU * i as f64 / CIRCLE_SEGMENTS as f64;
        seg_sin_r_sin[i] = sin_r * az.sin();
        seg_sin_r_cos[i] = sin_r * az.cos();
    }

    // All tissot circles share the same style — collect into a single <path>
    let mut d = String::new();

    let mut lat: f64 = -config.max_lat;
    while lat <= config.max_lat {
        let mut lon: f64 = -180.0;
        while lon < 180.0 {
            let phi0 = lat.to_radians();
            let sin_phi0 = phi0.sin();
            let cos_phi0 = phi0.cos();

            let mut pb = PathBuilder::new(String::new(), max_gap);
            let mut any_valid = false;

            for i in 0..=CIRCLE_SEGMENTS {
                let sr_cos = seg_sin_r_cos[i];
                let sr_sin = seg_sin_r_sin[i];

                let sin_lat = sin_phi0 * cos_r + cos_phi0 * sr_cos;
                let plat = sin_lat.asin();
                let dlon = sr_sin.atan2(cos_phi0 * cos_r - sin_phi0 * sr_cos);
                let plon = lon + dlon.to_degrees();

                let (x, y) = proj.project(plon, plat.to_degrees());
                if x.is_finite() && y.is_finite() {
                    any_valid = true;
                }
                pb.add(x, y);
            }

            if any_valid {
                pb.close_if_continuous();
                d.push_str(&pb.finish());
            }

            lon += config.step;
        }
        lat += config.step;
    }

    if !d.is_empty() {
        svg.push_str(r#"<path fill=""#);
        svg.push_str(&config.fill);
        svg.push_str(r#"" fill-opacity=""#);
        push_f64(&mut svg, config.fill_opacity);
        svg.push_str(r#"" stroke=""#);
        svg.push_str(&config.stroke);
        svg.push_str(r#"" stroke-width=""#);
        push_f64(&mut svg, config.stroke_width);
        svg.push_str(r#"" d=""#);
        svg.push_str(&d);
        svg.push_str(r#""/>"#);
    }

    svg
}
