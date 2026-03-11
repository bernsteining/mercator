use svg::node::element::{Group, Path};

use crate::geometry::PathBuilder;
use crate::projection::Projection;
use crate::style::TissotConfig;

const CIRCLE_SEGMENTS: usize = 36;

pub fn render(proj: &dyn Projection, config: &TissotConfig, max_gap: f64) -> Group {
    let mut group = Group::new();
    let radius_rad = config.radius.to_radians();
    let sin_r = radius_rad.sin();
    let cos_r = radius_rad.cos();

    let mut lat: f64 = -config.max_lat;
    while lat <= config.max_lat {
        let mut lon: f64 = -180.0;
        while lon < 180.0 {
            let phi0 = lat.to_radians();
            let sin_phi0 = phi0.sin();
            let cos_phi0 = phi0.cos();

            let mut pb = PathBuilder::new(svg::node::element::path::Data::new(), max_gap);
            let mut any_valid = false;

            for i in 0..=CIRCLE_SEGMENTS {
                let az = std::f64::consts::TAU * i as f64 / CIRCLE_SEGMENTS as f64;

                // Point on small circle at azimuth `az` and angular distance `radius`
                let sin_lat = sin_phi0 * cos_r + cos_phi0 * sin_r * az.cos();
                let plat = sin_lat.asin();
                let dlon = (sin_r * az.sin()).atan2(cos_phi0 * cos_r - sin_phi0 * sin_r * az.cos());
                let plon = lon + dlon.to_degrees();

                let (x, y) = proj.project(plon, plat.to_degrees());
                if x.is_finite() && y.is_finite() {
                    any_valid = true;
                }
                pb.add(x, y);
            }

            if any_valid {
                pb.close_if_continuous();
                group = group.add(
                    Path::new()
                        .set("fill", &*config.fill)
                        .set("fill-opacity", config.fill_opacity)
                        .set("stroke", &*config.stroke)
                        .set("stroke-width", config.stroke_width)
                        .set("d", pb.finish()),
                );
            }

            lon += config.step;
        }
        lat += config.step;
    }

    group
}
