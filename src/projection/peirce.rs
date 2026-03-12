use super::elliptic::{elliptic_f, elliptic_fi};
use super::Projection;
use std::f64::consts::{FRAC_1_SQRT_2, FRAC_PI_2, FRAC_PI_4, PI, SQRT_2};

pub(super) struct Compiled {
    center_lon: f64,
    k_prime: f64,
    k_sq: f64,
    k_complete: f64,
    dx: f64,
}

pub fn compile(center_lon: f64) -> Compiled {
    let k_prime = (SQRT_2 - 1.0) / (SQRT_2 + 1.0);
    let k_sq = 1.0 - k_prime * k_prime;
    let k_complete = elliptic_f(FRAC_PI_2, k_sq);

    let gx_pos = guyou_forward(FRAC_PI_2 - 1e-6, 0.0, k_prime, k_sq, k_complete);
    let gx_neg = guyou_forward(-FRAC_PI_2 + 1e-6, 0.0, k_prime, k_sq, k_complete);
    let dx = gx_pos.0 - gx_neg.0;

    Compiled { center_lon, k_prime, k_sq, k_complete, dx }
}

impl Projection for Compiled {
    fn antimeridian_gap(&self) -> f64 {
        1.0
    }

    fn project(&self, lon: f64, lat: f64) -> (f64, f64) {
        let lambda = (lon - self.center_lon).to_radians();
        let phi = lat.to_radians();

        let (rot_lam, rot_phi) = pre_rotate(lambda, phi);
        let (x, y) = quincuncial(rot_lam, rot_phi, self);

        (x, -y)
    }
}

#[inline]
fn pre_rotate(lambda: f64, phi: f64) -> (f64, f64) {
    let sin_phi = phi.sin();
    let cos_phi = phi.cos();
    let cos_lam = lambda.cos();
    let sin_lam = lambda.sin();

    let x3 = (sin_phi + cos_phi * cos_lam) * FRAC_1_SQRT_2;
    let y3 = (cos_phi * cos_lam - sin_phi) * FRAC_1_SQRT_2;
    let z3 = cos_phi * sin_lam;

    (y3.atan2(x3), z3.clamp(-1.0, 1.0).asin())
}

#[inline]
fn quincuncial(lambda: f64, phi: f64, c: &Compiled) -> (f64, f64) {
    let front = lambda.abs() < FRAC_PI_2;
    let lam = if front {
        lambda
    } else if lambda > 0.0 {
        lambda - PI
    } else {
        lambda + PI
    };

    let (gx, gy) = guyou_forward(lam, phi, c.k_prime, c.k_sq, c.k_complete);

    let x = (gx - gy) * FRAC_1_SQRT_2;
    let y = (gx + gy) * FRAC_1_SQRT_2;

    if front {
        (x, y)
    } else {
        let d = c.dx * FRAC_1_SQRT_2;
        let s = if (x > 0.0) ^ (y > 0.0) { -1.0 } else { 1.0 };
        (s * x - y.signum() * d, s * y - x.signum() * d)
    }
}

#[inline]
fn guyou_forward(lambda: f64, phi: f64, k_prime: f64, k_sq: f64, k_complete: f64) -> (f64, f64) {
    let psi = (FRAC_PI_4 + phi.abs() / 2.0).tan().ln();
    let r = (-psi).exp() / k_prime.sqrt();
    let at = complex_atan(r * (-lambda).cos(), r * (-lambda).sin());
    let t = elliptic_fi(at.0, at.1, k_sq);
    let sign_phi = if phi >= 0.0 { 1.0 } else { -1.0 };
    (-t.1, sign_phi * (0.5 * k_complete - t.0))
}

#[inline]
fn complex_atan(x: f64, y: f64) -> (f64, f64) {
    let x2 = x * x;
    let y_1 = y + 1.0;
    let t = 1.0 - x2 - y * y;
    (
        0.5 * (if x >= 0.0 { FRAC_PI_2 } else { -FRAC_PI_2 } - t.atan2(2.0 * x)),
        -0.25 * (t * t + 4.0 * x2).ln() + 0.5 * (y_1 * y_1 + x2).ln(),
    )
}
