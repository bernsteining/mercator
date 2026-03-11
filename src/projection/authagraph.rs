/// AuthaGraph projection (Kunimune's open-source approximation).
///
/// Maps the sphere onto a tetrahedron, then unfolds it into a rectangle.
/// Reference: <https://github.com/jkunimune/Map-Projections>

use super::Projection;
use std::f64::consts::{FRAC_PI_2, PI};

const K: f64 = 0.68;
const X_OFFSET: f64 = 0.6096;

struct Facet {
    x: f64,
    y: f64,
    plane_rotation: f64,
    lat: f64,
    lon: f64,
    cm: f64,
}

pub(super) struct Compiled {
    pole: [f64; 3],
    facets: [Facet; 6],
    sqrt3: f64,
    x_half: f64,
    y_half: f64,
}

pub fn compile() -> Compiled {
    let sqrt3 = 3.0_f64.sqrt();
    let asin_third = (1.0_f64 / 3.0).asin();
    let pi_3 = PI / 3.0;

    Compiled {
        pole: [
            77.0_f64.to_radians(),
            143.0_f64.to_radians(),
            17.0_f64.to_radians(),
        ],
        sqrt3,
        x_half: 2.0 * sqrt3,
        y_half: 1.5,
        facets: [
            Facet { x: -2.0 * sqrt3 - X_OFFSET, y:  1.5, plane_rotation: 0.0, lat: -asin_third, lon:  PI,   cm:  PI },
            Facet { x:      -sqrt3  - X_OFFSET,  y: -1.5, plane_rotation: 0.0, lat: -asin_third, lon: -pi_3, cm:  pi_3 },
            Facet { x:              -X_OFFSET,    y:  1.5, plane_rotation: 0.0, lat:  FRAC_PI_2,  lon:  0.0,  cm:  PI },
            Facet { x:       sqrt3  - X_OFFSET,  y: -1.5, plane_rotation: 0.0, lat: -asin_third, lon:  pi_3, cm: -pi_3 },
            Facet { x:  2.0 * sqrt3 - X_OFFSET,  y:  1.5, plane_rotation: 0.0, lat: -asin_third, lon:  PI,   cm:  PI },
            Facet { x:  3.0 * sqrt3 - X_OFFSET,  y: -1.5, plane_rotation: 0.0, lat: -asin_third, lon: -pi_3, cm:  pi_3 },
        ],
    }
}

impl Projection for Compiled {
    fn antimeridian_gap(&self) -> f64 {
        1.5
    }

    fn project(&self, lon_deg: f64, lat_deg: f64) -> (f64, f64) {
        let lat = lat_deg.to_radians();
        let lon = lon_deg.to_radians();

        let (lat_g, lon_g) = oblique(lat, lon, &self.pole);

        let mut best_lat = f64::NEG_INFINITY;
        let mut best_lon = 0.0;
        let mut best_idx = 0;

        for (i, f) in self.facets.iter().enumerate() {
            let pole = [f.lat, f.lon, f.cm];
            let (lat_r, lon_r) = oblique(lat_g, lon_g, &pole);
            if lat_r > best_lat {
                best_lat = lat_r;
                best_lon = lon_r;
                best_idx = i;
            }
        }

        let sector = 2.0 * PI / 3.0;
        let lon_r0 = ((best_lon + PI / 3.0) / sector).floor() * sector;

        let (r, tht) = face_project(best_lat, best_lon - lon_r0, self.sqrt3);

        let facet = &self.facets[best_idx];
        let th = tht + facet.plane_rotation + lon_r0 * 0.5;
        let mut x = r * th.cos() + facet.x;
        let mut y = r * th.sin() + facet.y;

        if y < -self.y_half || y > self.y_half {
            x = 2.0 * facet.x - x;
            y = 2.0 * facet.y - y;
        }
        if x < -self.x_half || x > self.x_half {
            let w = 2.0 * self.x_half;
            x = (x + self.x_half).rem_euclid(w) - self.x_half;
        }

        (x, -y)
    }
}

fn face_project(lat: f64, lon: f64, sqrt3: f64) -> (f64, f64) {
    let sin_lon = lon.sin();
    let cos_lon = lon.cos();
    let correction = (sin_lon / sqrt3).clamp(-1.0, 1.0).asin();
    let tht = ((lon - correction) / PI * 12.0_f64.sqrt()).atan();
    let denom = (2.0_f64.sqrt() / cos_lon).atan();
    let p = if denom.abs() < 1e-15 {
        0.0
    } else {
        ((FRAC_PI_2 - lat) / denom).max(0.0)
    };
    let cos_tht = tht.cos();
    let r = if cos_tht.abs() < 1e-15 {
        0.0
    } else {
        p.powf(K) * sqrt3 / cos_tht
    };
    (r, tht)
}

fn oblique(lat_f: f64, lon_f: f64, pole: &[f64; 3]) -> (f64, f64) {
    let (lat0, lon0, tht0) = (pole[0], pole[1], pole[2]);

    let lat1 = if (lat0 - FRAC_PI_2).abs() < 1e-10 {
        lat_f
    } else if (lat0 + FRAC_PI_2).abs() < 1e-10 {
        -lat_f
    } else {
        (lat0.sin() * lat_f.sin() + lat0.cos() * lat_f.cos() * (lon0 - lon_f).cos())
            .clamp(-1.0, 1.0)
            .asin()
    };

    let mut lon1 = if (lat0 - FRAC_PI_2).abs() < 1e-10 {
        lon_f - lon0
    } else if (lat0 + FRAC_PI_2).abs() < 1e-10 {
        lon0 - lon_f - PI
    } else {
        let cos_lat1 = lat1.cos();
        if cos_lat1.abs() < 1e-15 {
            0.0
        } else {
            let val = (lat0.cos() * lat_f.sin()
                - lat0.sin() * lat_f.cos() * (lon0 - lon_f).cos())
                / cos_lat1;
            let mut l = val.clamp(-1.0, 1.0).acos() - PI;
            if (lon_f - lon0).sin() > 0.0 {
                l = -l;
            }
            l
        }
    };

    lon1 -= tht0;
    if lon1.abs() > PI {
        lon1 = (lon1 + PI).rem_euclid(2.0 * PI) - PI;
    }
    if lon1 >= PI - 1e-7 {
        lon1 = -PI;
    }

    (lat1, lon1)
}
