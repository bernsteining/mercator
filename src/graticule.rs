use svg::node::element::{Group, Path};

use crate::geometry::PathBuilder;
use crate::projection::Projection;
use crate::style::GraticuleConfig;

const GRATICULE_SEGMENTS: usize = 360;

pub fn render(proj: &dyn Projection, grat: &GraticuleConfig, max_gap: f64) -> Group {
    let mut group = Group::new();
    let step = grat.step;

    let make_path = |data: svg::node::element::path::Data| -> Path {
        Path::new()
            .set("fill", "none")
            .set("stroke", &*grat.color)
            .set("stroke-width", grat.width)
            .set("stroke-opacity", grat.opacity)
            .set("d", data)
    };

    // Meridians (longitude lines)
    let mut lon = -180.0;
    while lon <= 180.0 {
        let mut pb = PathBuilder::new(svg::node::element::path::Data::new(), max_gap);
        for i in 0..=GRATICULE_SEGMENTS {
            let lat = -90.0 + (180.0 * i as f64 / GRATICULE_SEGMENTS as f64);
            let (x, y) = proj.project(lon, lat);
            pb.add(x, y);
        }
        group = group.add(make_path(pb.finish()));
        lon += step;
    }

    // Parallels (latitude lines)
    let mut lat = -90.0;
    while lat <= 90.0 {
        let mut pb = PathBuilder::new(svg::node::element::path::Data::new(), max_gap);
        for i in 0..=GRATICULE_SEGMENTS * 2 {
            let lng = -180.0 + (360.0 * i as f64 / (GRATICULE_SEGMENTS * 2) as f64);
            let (x, y) = proj.project(lng, lat);
            pb.add(x, y);
        }
        group = group.add(make_path(pb.finish()));
        lat += step;
    }

    group
}
