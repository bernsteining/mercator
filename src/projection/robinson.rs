use super::{normalize_lon, Projection};

const X_SCALE: f64 = 0.8487;
const Y_SCALE: f64 = 1.3523;
const TABLE_STEP: f64 = 5.0;

const TABLE: [(f64, f64); 19] = [
    (1.0000, 0.0000), // 0°
    (0.9986, 0.0620), // 5°
    (0.9954, 0.1240), // 10°
    (0.9900, 0.1860), // 15°
    (0.9822, 0.2480), // 20°
    (0.9730, 0.3100), // 25°
    (0.9600, 0.3720), // 30°
    (0.9427, 0.4340), // 35°
    (0.9216, 0.4958), // 40°
    (0.8962, 0.5571), // 45°
    (0.8679, 0.6176), // 50°
    (0.8350, 0.6769), // 55°
    (0.7986, 0.7346), // 60°
    (0.7597, 0.7903), // 65°
    (0.7186, 0.8435), // 70°
    (0.6732, 0.8936), // 75°
    (0.6213, 0.9394), // 80°
    (0.5722, 0.9761), // 85°
    (0.5322, 1.0000), // 90°
];

#[inline]
fn interpolate(abs_lat: f64) -> (f64, f64) {
    let idx = (abs_lat / TABLE_STEP).floor() as usize;
    if idx >= TABLE.len() - 1 {
        return TABLE[TABLE.len() - 1];
    }
    let t = (abs_lat - idx as f64 * TABLE_STEP) / TABLE_STEP;
    let (plen0, pdfe0) = TABLE[idx];
    let (plen1, pdfe1) = TABLE[idx + 1];
    (plen0 + t * (plen1 - plen0), pdfe0 + t * (pdfe1 - pdfe0))
}

pub struct Robinson {
    pub central_meridian: f64,
}

impl Projection for Robinson {
    fn project(&self, lon: f64, lat: f64) -> (f64, f64) {
        let abs_lat = lat.clamp(-90.0, 90.0).abs();
        let (plen, pdfe) = interpolate(abs_lat);
        let delta_lon = normalize_lon(lon - self.central_meridian);
        let x = X_SCALE * plen * delta_lon.to_radians();
        let y = Y_SCALE * pdfe * lat.signum();
        (x, -y)
    }

    fn antimeridian_gap(&self) -> f64 {
        3.0
    }
}
