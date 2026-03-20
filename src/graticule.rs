use crate::geometry::{push_f64, PathBuilder};
use crate::projection::Proj;
use crate::style::GraticuleConfig;

const GRATICULE_SEGMENTS: usize = 90;

pub fn render(proj: &Proj, grat: &GraticuleConfig, max_gap: f64) -> String {
    let mut svg = String::new();
    let step = grat.step;

    // All graticule lines share the same style — use a single <path> element
    let mut d = String::new();

    // Meridians (longitude lines)
    let lat_step = 180.0 / GRATICULE_SEGMENTS as f64;
    let mut lon = -180.0;
    while lon <= 180.0 {
        let mut pb = PathBuilder::new(std::mem::take(&mut d), max_gap);
        for i in 0..=GRATICULE_SEGMENTS {
            let lat = -90.0 + lat_step * i as f64;
            let (x, y) = proj.project(lon, lat);
            pb.add(x, y);
        }
        d = pb.finish();
        lon += step;
    }

    // Parallels (latitude lines)
    let lon_step = 360.0 / (GRATICULE_SEGMENTS * 2) as f64;
    let mut lat = -90.0;
    while lat <= 90.0 {
        let mut pb = PathBuilder::new(std::mem::take(&mut d), max_gap);
        for i in 0..=GRATICULE_SEGMENTS * 2 {
            let lng = -180.0 + lon_step * i as f64;
            let (x, y) = proj.project(lng, lat);
            pb.add(x, y);
        }
        d = pb.finish();
        lat += step;
    }

    if !d.is_empty() {
        svg.push_str(r#"<path fill="none" stroke=""#);
        svg.push_str(&grat.color);
        svg.push_str(r#"" stroke-width=""#);
        push_f64(&mut svg, grat.width);
        svg.push_str(r#"" stroke-opacity=""#);
        push_f64(&mut svg, grat.opacity);
        svg.push_str(r#"" d=""#);
        svg.push_str(&d);
        svg.push_str(r#""/>"#);
    }

    svg
}
