//! Density contours from projected points: rasterize points into a grid, blur it
//! to a smooth density surface, then trace iso-lines with marching squares.
//! A lightweight take on d3-contourDensity (iso-lines rather than filled bands).

/// A scalar density grid over projected space.
pub struct Grid {
    w: usize,
    h: usize,
    x0: f64,
    y0: f64,
    cell: f64,
    v: Vec<f64>,
}

impl Grid {
    #[inline]
    fn at(&self, i: usize, j: usize) -> f64 {
        self.v[j * self.w + i]
    }
    pub fn max(&self) -> f64 {
        self.v.iter().cloned().fold(0.0, f64::max)
    }
}

/// Rasterize `points` into a grid covering `[x0,y0]..[x1,y1]` (plus a margin so
/// contours close), then box-blur `blur` times to approximate a Gaussian KDE.
pub fn density(points: &[(f64, f64)], rect: (f64, f64, f64, f64), cell: f64, blur: usize) -> Grid {
    let (rx0, ry0, rx1, ry1) = rect;
    let cell = if cell > 0.0 { cell } else { ((rx1 - rx0).max(ry1 - ry0) / 120.0).max(1e-6) };
    let margin = cell * (blur as f64 + 2.0);
    let x0 = rx0 - margin;
    let y0 = ry0 - margin;
    let w = (((rx1 + margin - x0) / cell).ceil() as usize + 1).max(2);
    let h = (((ry1 + margin - y0) / cell).ceil() as usize + 1).max(2);
    let mut v = vec![0.0f64; w * h];
    for &(px, py) in points {
        if !px.is_finite() || !py.is_finite() {
            continue;
        }
        let i = ((px - x0) / cell).round();
        let j = ((py - y0) / cell).round();
        if i >= 0.0 && (i as usize) < w && j >= 0.0 && (j as usize) < h {
            v[j as usize * w + i as usize] += 1.0;
        }
    }
    for _ in 0..blur {
        box_blur(&mut v, w, h);
    }
    Grid { w, h, x0, y0, cell, v }
}

/// One separable 3-tap box-blur pass (horizontal then vertical, edge-clamped).
fn box_blur(v: &mut [f64], w: usize, h: usize) {
    let src = v.to_vec();
    for j in 0..h {
        for i in 0..w {
            let a = src[j * w + i.saturating_sub(1)];
            let b = src[j * w + i];
            let c = src[j * w + (i + 1).min(w - 1)];
            v[j * w + i] = (a + b + c) / 3.0;
        }
    }
    let src = v.to_vec();
    for j in 0..h {
        for i in 0..w {
            let a = src[j.saturating_sub(1) * w + i];
            let b = src[j * w + i];
            let c = src[(j + 1).min(h - 1) * w + i];
            v[j * w + i] = (a + b + c) / 3.0;
        }
    }
}

/// Append the iso-line segments at value `t` (marching squares) as `(x1,y1,x2,y2)`.
pub fn iso_segments(g: &Grid, t: f64, out: &mut Vec<(f64, f64, f64, f64)>) {
    let lerp = |va: f64, vb: f64| -> f64 {
        let d = vb - va;
        if d.abs() < 1e-12 { 0.5 } else { ((t - va) / d).clamp(0.0, 1.0) }
    };
    for j in 0..g.h - 1 {
        for i in 0..g.w - 1 {
            let (tl, tr, br, bl) = (g.at(i, j), g.at(i + 1, j), g.at(i + 1, j + 1), g.at(i, j + 1));
            let mut idx = 0u8;
            if tl >= t { idx |= 8; }
            if tr >= t { idx |= 4; }
            if br >= t { idx |= 2; }
            if bl >= t { idx |= 1; }
            if idx == 0 || idx == 15 {
                continue;
            }
            let (cx0, cy0) = (g.x0 + i as f64 * g.cell, g.y0 + j as f64 * g.cell);
            let (cx1, cy1) = (cx0 + g.cell, cy0 + g.cell);
            let top = (cx0 + lerp(tl, tr) * g.cell, cy0);
            let bottom = (cx0 + lerp(bl, br) * g.cell, cy1);
            let left = (cx0, cy0 + lerp(tl, bl) * g.cell);
            let right = (cx1, cy0 + lerp(tr, br) * g.cell);
            let mut seg = |a: (f64, f64), b: (f64, f64)| out.push((a.0, a.1, b.0, b.1));
            match idx {
                1 | 14 => seg(left, bottom),
                2 | 13 => seg(bottom, right),
                3 | 12 => seg(left, right),
                4 | 11 => seg(top, right),
                6 | 9 => seg(top, bottom),
                7 | 8 => seg(left, top),
                5 => { seg(left, top); seg(bottom, right); }
                10 => { seg(left, bottom); seg(top, right); }
                _ => {}
            }
        }
    }
}
