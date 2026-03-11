use std::f64::consts::{FRAC_PI_4, PI};

const EPSILON: f64 = 1e-12;

/// Incomplete elliptic integral of the first kind F(φ|m) where m = k².
/// Abramowitz and Stegun, 17.6.7.
pub fn elliptic_f(phi: f64, m: f64) -> f64 {
    if m == 0.0 {
        return phi;
    }
    if m == 1.0 {
        return (phi / 2.0 + FRAC_PI_4).tan().ln();
    }
    let mut a = 1.0;
    let mut b = (1.0 - m).sqrt();
    let mut c = m.sqrt();
    let mut phi = phi;
    let mut i = 0u32;

    while c.abs() > EPSILON {
        if phi % PI != 0.0 {
            let mut dp = (b * phi.tan() / a).atan();
            if dp < 0.0 {
                dp += PI;
            }
            phi += dp + (phi / PI).trunc() * PI;
        } else {
            phi += phi;
        }
        let new_c = (a + b) / 2.0;
        b = (a * b).sqrt();
        c = (new_c - b) / 2.0;
        a = new_c;
        i += 1;
    }

    phi / (2.0_f64.powi(i as i32) * a)
}

/// Complex incomplete elliptic integral F(φ + iψ | m).
/// Abramowitz and Stegun, 17.4.11.
pub fn elliptic_fi(phi: f64, psi: f64, m: f64) -> (f64, f64) {
    let r = phi.abs();
    let i = psi.abs();
    let sinh_psi = i.sinh();

    if r != 0.0 {
        let csc_phi = 1.0 / r.sin();
        let cot_phi2 = 1.0 / (r.tan() * r.tan());
        let b = -(cot_phi2 + m * (sinh_psi * sinh_psi * csc_phi * csc_phi) - 1.0 + m);
        let c = (m - 1.0) * cot_phi2;
        let cot_lambda2 = (-b + (b * b - 4.0 * c).sqrt()) / 2.0;
        (
            elliptic_f((1.0 / cot_lambda2.sqrt()).atan(), m) * phi.signum(),
            elliptic_f(((cot_lambda2 / cot_phi2 - 1.0) / m).sqrt().atan(), 1.0 - m) * psi.signum(),
        )
    } else {
        (
            0.0,
            elliptic_f(sinh_psi.atan(), 1.0 - m) * psi.signum(),
        )
    }
}
