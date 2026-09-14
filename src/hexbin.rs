//! Hexagonal binning of projected points — a faithful port of d3-hexbin's grid.
//! Bins are keyed on a pointy-top hex lattice with horizontal spacing `r·√3` and
//! vertical spacing `1.5·r`; each returned bin carries its center and point count.

use std::collections::HashMap;

/// A populated hex cell: center `(cx, cy)` (projected units) and point count.
pub struct HexBin {
    pub cx: f64,
    pub cy: f64,
    pub count: u32,
}

/// Bin `points` (already projected) into hexagons of the given `radius`.
pub fn bin(points: &[(f64, f64)], radius: f64) -> Vec<HexBin> {
    if radius <= 0.0 {
        return Vec::new();
    }
    let dx = radius * 2.0 * (std::f64::consts::PI / 3.0).sin(); // r·√3
    let dy = radius * 1.5;
    let mut cells: HashMap<(i64, i64), u32> = HashMap::new();

    for &(x, y) in points {
        if !x.is_finite() || !y.is_finite() {
            continue;
        }
        let py = y / dy;
        let mut pj = py.round();
        let odd = (pj as i64).rem_euclid(2) == 1;
        let px = x / dx - if odd { 0.5 } else { 0.0 };
        let mut pi = px.round();
        let py1 = py - pj;
        // If near a cell edge, test the nearer diagonal neighbor (d3-hexbin).
        if py1.abs() * 3.0 > 1.0 {
            let px1 = px - pi;
            let pi2 = pi + if px < pi { -0.5 } else { 0.5 };
            let pj2 = pj + if py < pj { -1.0 } else { 1.0 };
            let px2 = px - pi2;
            let py2 = py - pj2;
            if px1 * px1 + py1 * py1 > px2 * px2 + py2 * py2 {
                pi = pi2 + if (pj as i64).rem_euclid(2) == 1 { 0.5 } else { -0.5 };
                pj = pj2;
            }
        }
        *cells.entry((pi.round() as i64, pj as i64)).or_insert(0) += 1;
    }

    cells
        .into_iter()
        .map(|((i, j), count)| {
            let odd = j.rem_euclid(2) == 1;
            let cx = (i as f64 + if odd { 0.5 } else { 0.0 }) * dx;
            let cy = j as f64 * dy;
            HexBin { cx, cy, count }
        })
        .collect()
}

/// The six vertices of a pointy-top hexagon of `radius` centered at `(cx, cy)`.
pub fn hexagon(cx: f64, cy: f64, radius: f64) -> [(f64, f64); 6] {
    let mut v = [(0.0, 0.0); 6];
    for (k, item) in v.iter_mut().enumerate() {
        let a = std::f64::consts::PI / 3.0 * k as f64;
        *item = (cx + a.sin() * radius, cy - a.cos() * radius);
    }
    v
}
