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
    /// Precomputed sin/cos of lat, lon, and cm to avoid recomputing per point.
    sin_lat: f64,
    cos_lat: f64,
    lon: f64,
    cm: f64,
}

/// Precomputed pole with sin/cos cached.
struct Pole {
    sin_lat: f64,
    cos_lat: f64,
    lon: f64,
    tht: f64,
}

pub(super) struct Compiled {
    pole: Pole,
    facets: [Facet; 6],
    sqrt3: f64,
    x_half: f64,
    y_half: f64,
}

pub fn compile() -> Compiled {
    let sqrt3 = 3.0_f64.sqrt();
    let asin_third = (1.0_f64 / 3.0).asin();
    let pi_3 = PI / 3.0;

    let pole_lat = 77.0_f64.to_radians();
    let pole_lon = 143.0_f64.to_radians();
    let pole_tht = 17.0_f64.to_radians();

    let make_facet = |x: f64, y: f64, lat: f64, lon: f64, cm: f64| -> Facet {
        Facet {
            x,
            y,
            plane_rotation: 0.0,
            sin_lat: lat.sin(),
            cos_lat: lat.cos(),
            lon,
            cm,
        }
    };

    Compiled {
        pole: Pole {
            sin_lat: pole_lat.sin(),
            cos_lat: pole_lat.cos(),
            lon: pole_lon,
            tht: pole_tht,
        },
        sqrt3,
        x_half: 2.0 * sqrt3,
        y_half: 1.5,
        facets: [
            make_facet(-2.0 * sqrt3 - X_OFFSET,  1.5, -asin_third,  PI,   PI),
            make_facet(     -sqrt3  - X_OFFSET,  -1.5, -asin_third, -pi_3, pi_3),
            make_facet(             -X_OFFSET,    1.5,  FRAC_PI_2,   0.0,  PI),
            make_facet(      sqrt3  - X_OFFSET,  -1.5, -asin_third,  pi_3, -pi_3),
            make_facet( 2.0 * sqrt3 - X_OFFSET,  1.5, -asin_third,  PI,   PI),
            make_facet( 3.0 * sqrt3 - X_OFFSET, -1.5, -asin_third, -pi_3, pi_3),
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

        let (lat_g, lon_g) = oblique_pole(lat, lon, &self.pole);

        let mut best_lat = f64::NEG_INFINITY;
        let mut best_lon = 0.0;
        let mut best_idx = 0;

        for (i, f) in self.facets.iter().enumerate() {
            let (lat_r, lon_r) = oblique_facet(lat_g, lon_g, f);
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

#[inline]
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

/// Oblique rotation using the precomputed main pole (sin/cos cached).
#[inline]
fn oblique_pole(lat_f: f64, lon_f: f64, pole: &Pole) -> (f64, f64) {
    let sin_lat0 = pole.sin_lat;
    let cos_lat0 = pole.cos_lat;
    let lon0 = pole.lon;
    let tht0 = pole.tht;

    let lat1 = if (sin_lat0 - 1.0).abs() < 1e-10 {
        // lat0 ≈ π/2
        lat_f
    } else if (sin_lat0 + 1.0).abs() < 1e-10 {
        // lat0 ≈ -π/2
        -lat_f
    } else {
        (sin_lat0 * lat_f.sin() + cos_lat0 * lat_f.cos() * (lon0 - lon_f).cos())
            .clamp(-1.0, 1.0)
            .asin()
    };

    let mut lon1 = if (sin_lat0 - 1.0).abs() < 1e-10 {
        lon_f - lon0
    } else if (sin_lat0 + 1.0).abs() < 1e-10 {
        lon0 - lon_f - PI
    } else {
        let cos_lat1 = lat1.cos();
        if cos_lat1.abs() < 1e-15 {
            0.0
        } else {
            let val = (cos_lat0 * lat_f.sin()
                - sin_lat0 * lat_f.cos() * (lon0 - lon_f).cos())
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

/// Oblique rotation using a facet's precomputed sin/cos.
#[inline]
fn oblique_facet(lat_f: f64, lon_f: f64, facet: &Facet) -> (f64, f64) {
    let sin_lat0 = facet.sin_lat;
    let cos_lat0 = facet.cos_lat;
    let lon0 = facet.lon;
    let tht0 = facet.cm;

    let lat1 = if (cos_lat0).abs() < 1e-10 {
        if sin_lat0 > 0.0 { lat_f } else { -lat_f }
    } else {
        (sin_lat0 * lat_f.sin() + cos_lat0 * lat_f.cos() * (lon0 - lon_f).cos())
            .clamp(-1.0, 1.0)
            .asin()
    };

    let mut lon1 = if (cos_lat0).abs() < 1e-10 {
        if sin_lat0 > 0.0 {
            lon_f - lon0
        } else {
            lon0 - lon_f - PI
        }
    } else {
        let cos_lat1 = lat1.cos();
        if cos_lat1.abs() < 1e-15 {
            0.0
        } else {
            let val = (cos_lat0 * lat_f.sin()
                - sin_lat0 * lat_f.cos() * (lon0 - lon_f).cos())
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
