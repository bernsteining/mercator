//! Data-driven fill: map a numeric feature property to a color (choropleth), plus
//! a legend. Configured via `fill_scale` / `legend` and applied in `resolve_style`.

use std::collections::HashMap;

use serde::Deserialize;
use serde_json::{Map, Value};

use crate::geometry::push_f64;
use crate::model::{Feature, GeoJson};

fn default_scale_type() -> String {
    "quantize".to_string()
}

/// `fill_scale` config: bind `property` to a color `range` via a scale `type`.
#[derive(Debug, Deserialize)]
pub struct FillScale {
    pub property: String,
    #[serde(rename = "type", default = "default_scale_type")]
    pub scale_type: String,
    /// `(min, max)`. Auto-computed from the data when omitted (`quantize`/`linear`).
    pub domain: Option<(f64, f64)>,
    /// Colors: discrete bins for `quantize`, gradient stops for `linear`.
    #[serde(default)]
    pub range: Vec<String>,
    /// Explicit value→color map for `type: "category"`.
    pub categories: Option<Map<String, Value>>,
    /// Palette auto-assigned to distinct values (in first-seen order) for
    /// `type: "category"` when `categories` is not given.
    pub palette: Option<Vec<String>>,
    /// Color for features whose value is missing / unmatched (default `#cccccc`).
    pub default: Option<String>,
}

fn default_radius_type() -> String {
    "sqrt".to_string()
}

/// `point_radius_scale` config: size Point/MultiPoint symbols by a numeric property.
#[derive(Debug, Deserialize)]
pub struct RadiusScale {
    pub property: String,
    /// `(min, max)` value domain. Auto-computed from the data when omitted.
    pub domain: Option<(f64, f64)>,
    /// Radius for the smallest value (default 0).
    pub min_radius: Option<f64>,
    /// Radius for the largest value (default 1.0).
    pub max_radius: Option<f64>,
    /// `"sqrt"` (default; area ∝ value) or `"linear"` (radius ∝ value).
    #[serde(rename = "type", default = "default_radius_type")]
    pub scale_type: String,
}

/// A resolved size scale: maps a feature's property to a point radius.
pub struct SizeScale {
    property: String,
    min_v: f64,
    max_v: f64,
    min_r: f64,
    max_r: f64,
    sqrt: bool,
}

impl SizeScale {
    pub fn build(cfg: &RadiusScale, gj: &GeoJson) -> Option<SizeScale> {
        let (min_v, max_v) = match cfg.domain {
            Some((a, b)) => (a, b),
            None => {
                let mut lo = f64::INFINITY;
                let mut hi = f64::NEG_INFINITY;
                for v in feature_values(gj, &cfg.property) {
                    lo = lo.min(v);
                    hi = hi.max(v);
                }
                if lo > hi {
                    return None;
                }
                (lo, hi)
            }
        };
        Some(SizeScale {
            property: cfg.property.clone(),
            min_v,
            max_v,
            min_r: cfg.min_radius.unwrap_or(0.0),
            max_r: cfg.max_radius.unwrap_or(1.0),
            sqrt: cfg.scale_type != "linear",
        })
    }

    /// Radius for a feature's properties. Missing/non-numeric → the minimum radius.
    /// `sqrt` scaling interpolates in √value space so symbol *area* tracks the value.
    pub fn radius_for(&self, props: &Map<String, Value>) -> f64 {
        let v = match props.get(&self.property).and_then(Value::as_f64) {
            Some(v) => v,
            None => return self.min_r,
        };
        let t = if self.sqrt {
            let s = |x: f64| x.max(0.0).sqrt();
            let (a, b) = (s(self.min_v), s(self.max_v));
            if b > a {
                ((s(v) - a) / (b - a)).clamp(0.0, 1.0)
            } else {
                0.0
            }
        } else if self.max_v > self.min_v {
            ((v - self.min_v) / (self.max_v - self.min_v)).clamp(0.0, 1.0)
        } else {
            0.0
        };
        self.min_r + (self.max_r - self.min_r) * t
    }
}

fn default_legend_pos() -> String {
    "bottom-right".to_string()
}

/// `legend` config: a swatch key for the active `fill_scale`.
#[derive(Debug, Deserialize)]
pub struct LegendConfig {
    pub title: Option<String>,
    /// One of `top-left`, `top-right`, `bottom-left`, `bottom-right`.
    #[serde(default = "default_legend_pos")]
    pub pos: String,
}

enum Kind {
    /// `n` equal-width bins over `[min, max]` → `colors[bin]`.
    Quantize { min: f64, max: f64, colors: Vec<String> },
    /// Interpolate RGB between evenly-spaced `stops` over `[min, max]`.
    Linear { min: f64, max: f64, stops: Vec<[f64; 3]> },
    /// Map a categorical value to a color. `lookup` resolves; `order` drives the legend.
    Category { lookup: HashMap<String, String>, order: Vec<(String, String)> },
}

/// Stringify a property value into a category key (string, number, or bool).
fn value_key(v: &Value) -> Option<String> {
    match v {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(b.to_string()),
        _ => None,
    }
}

/// Iterate the distinct category keys of `prop` across features, in first-seen order.
fn distinct_keys(gj: &GeoJson, prop: &str) -> Vec<String> {
    let feats: &[Feature] = match gj {
        GeoJson::FeatureCollection(f) => f,
        GeoJson::Feature(f) => std::slice::from_ref(f),
        GeoJson::Geometry(_) => &[],
    };
    let mut seen: HashMap<String, ()> = HashMap::new();
    let mut order = Vec::new();
    for f in feats {
        if let Some(k) = f.properties.as_ref().and_then(|p| p.get(prop)).and_then(value_key) {
            if seen.insert(k.clone(), ()).is_none() {
                order.push(k);
            }
        }
    }
    order
}

/// A resolved color scale ready to map a feature to a fill.
pub struct ColorScale {
    property: String,
    default: String,
    kind: Kind,
}

/// Iterate the numeric values of `prop` across a parsed GeoJSON's features.
fn feature_values<'a>(gj: &'a GeoJson, prop: &'a str) -> Box<dyn Iterator<Item = f64> + 'a> {
    let feats: &[Feature] = match gj {
        GeoJson::FeatureCollection(f) => f,
        GeoJson::Feature(f) => std::slice::from_ref(f),
        GeoJson::Geometry(_) => &[],
    };
    Box::new(
        feats
            .iter()
            .filter_map(move |f| f.properties.as_ref()?.get(prop)?.as_f64()),
    )
}

fn parse_hex(s: &str) -> Option<[f64; 3]> {
    let h = s.trim().strip_prefix('#')?;
    let (r, g, b) = match h.len() {
        6 => (
            u8::from_str_radix(&h[0..2], 16).ok()?,
            u8::from_str_radix(&h[2..4], 16).ok()?,
            u8::from_str_radix(&h[4..6], 16).ok()?,
        ),
        3 => {
            let d = |c: &str| u8::from_str_radix(&c.repeat(2), 16).ok();
            (d(&h[0..1])?, d(&h[1..2])?, d(&h[2..3])?)
        }
        _ => return None,
    };
    Some([r as f64, g as f64, b as f64])
}

impl ColorScale {
    /// Build a scale from config, computing the domain from `gj` when not given.
    /// Returns `None` if the range is empty or the domain can't be determined.
    pub fn build(cfg: &FillScale, gj: &GeoJson) -> Option<ColorScale> {
        let default = cfg.default.clone().unwrap_or_else(|| "#cccccc".to_string());

        if cfg.scale_type == "category" {
            let mut lookup = HashMap::new();
            let mut order = Vec::new();
            if let Some(cats) = &cfg.categories {
                // Explicit value→color map (insertion order preserved).
                for (k, v) in cats {
                    if let Some(color) = v.as_str() {
                        lookup.insert(k.clone(), color.to_string());
                        order.push((k.clone(), color.to_string()));
                    }
                }
            } else if let Some(palette) = &cfg.palette {
                // Auto-assign palette colors to distinct values, cycling if needed.
                if palette.is_empty() {
                    return None;
                }
                for key in distinct_keys(gj, &cfg.property) {
                    let color = palette[order.len() % palette.len()].clone();
                    lookup.insert(key.clone(), color.clone());
                    order.push((key, color));
                }
            } else {
                return None; // category needs `categories` or `palette`
            }
            if order.is_empty() {
                return None;
            }
            return Some(ColorScale {
                property: cfg.property.clone(),
                default,
                kind: Kind::Category { lookup, order },
            });
        }

        if cfg.range.is_empty() {
            return None;
        }
        let (min, max) = match cfg.domain {
            Some((a, b)) => (a, b),
            None => {
                let mut lo = f64::INFINITY;
                let mut hi = f64::NEG_INFINITY;
                for v in feature_values(gj, &cfg.property) {
                    lo = lo.min(v);
                    hi = hi.max(v);
                }
                if lo > hi {
                    return None; // no numeric values found
                }
                (lo, hi)
            }
        };
        let kind = if cfg.scale_type == "linear" {
            let stops: Vec<[f64; 3]> = cfg.range.iter().filter_map(|c| parse_hex(c)).collect();
            if stops.len() < 2 {
                // linear needs ≥2 parseable hex stops; fall back to discrete bins.
                Kind::Quantize { min, max, colors: cfg.range.clone() }
            } else {
                Kind::Linear { min, max, stops }
            }
        } else {
            Kind::Quantize { min, max, colors: cfg.range.clone() }
        };
        Some(ColorScale { property: cfg.property.clone(), default, kind })
    }

    /// Normalized position of `v` within `[min, max]`, clamped to `[0, 1]`.
    fn norm(min: f64, max: f64, v: f64) -> f64 {
        if max > min {
            ((v - min) / (max - min)).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    /// The color for a feature's properties (owned so it can override the fill).
    pub fn color_for(&self, props: &Map<String, Value>) -> String {
        // Category scales key on any value type; numeric scales need an f64.
        if let Kind::Category { lookup, .. } = &self.kind {
            return props
                .get(&self.property)
                .and_then(value_key)
                .and_then(|k| lookup.get(&k).cloned())
                .unwrap_or_else(|| self.default.clone());
        }
        let value = props.get(&self.property).and_then(Value::as_f64);
        let value = match value {
            Some(v) => v,
            None => return self.default.clone(),
        };
        match &self.kind {
            Kind::Quantize { min, max, colors } => {
                let n = colors.len();
                let t = Self::norm(*min, *max, value);
                let idx = ((t * n as f64) as usize).min(n - 1);
                colors[idx].clone()
            }
            Kind::Linear { min, max, stops } => {
                let t = Self::norm(*min, *max, value);
                let seg = (stops.len() - 1) as f64;
                let pos = t * seg;
                let i = (pos.floor() as usize).min(stops.len() - 2);
                let f = pos - i as f64;
                let a = stops[i];
                let b = stops[i + 1];
                let ch = |k: usize| (a[k] + (b[k] - a[k]) * f).round() as u8;
                format!("#{:02x}{:02x}{:02x}", ch(0), ch(1), ch(2))
            }
            // Handled by the early return above; kept for exhaustiveness.
            Kind::Category { .. } => self.default.clone(),
        }
    }

    /// Discrete (color, "lo–hi") swatches describing the scale, for the legend.
    fn swatches(&self) -> Vec<(String, String)> {
        match &self.kind {
            Kind::Quantize { min, max, colors } => {
                let n = colors.len();
                (0..n)
                    .map(|i| {
                        let lo = min + (max - min) * i as f64 / n as f64;
                        let hi = min + (max - min) * (i + 1) as f64 / n as f64;
                        (colors[i].clone(), format!("{} – {}", fmt_num(lo), fmt_num(hi)))
                    })
                    .collect()
            }
            Kind::Linear { min, max, .. } => {
                // Sample the gradient at 5 evenly-spaced stops.
                let n = 5;
                (0..n)
                    .map(|i| {
                        let v = min + (max - min) * i as f64 / (n - 1) as f64;
                        let mut m = Map::new();
                        m.insert(self.property.clone(), Value::from(v));
                        (self.color_for(&m), fmt_num(v))
                    })
                    .collect()
            }
            Kind::Category { order, .. } => {
                order.iter().map(|(label, color)| (color.clone(), label.clone())).collect()
            }
        }
    }
}

/// Format a legend number compactly (integers without a decimal point).
fn fmt_num(v: f64) -> String {
    if v.fract() == 0.0 && v.abs() < 1e15 {
        format!("{}", v as i64)
    } else {
        let s = format!("{:.2}", v);
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

/// Render the legend as SVG into `svg`, positioned inside `viewbox = (x, y, w, h)`.
pub fn render_legend(
    svg: &mut String,
    scale: &ColorScale,
    cfg: &LegendConfig,
    viewbox: (f64, f64, f64, f64),
) {
    let (vx, vy, vw, vh) = viewbox;
    let items = scale.swatches();
    if items.is_empty() {
        return;
    }

    // Size everything relative to the viewbox so the legend reads consistently.
    let unit = vw.min(vh);
    let sw = unit * 0.035; // swatch side
    let gap = sw * 0.35;
    let font = sw * 0.85;
    let pad = unit * 0.02;
    let row = sw + gap;
    let title_h = if cfg.title.is_some() { font * 1.6 } else { 0.0 };
    let block_h = title_h + items.len() as f64 * row - gap;
    // Rough legend width: swatch + text; assume ~0.6*font per char of the widest label.
    let max_label = items.iter().map(|(_, l)| l.len()).max().unwrap_or(0) as f64;
    let block_w = sw + gap + max_label * font * 0.6;

    let right = cfg.pos.contains("right");
    let bottom = cfg.pos.contains("bottom");
    let x0 = if right { vx + vw - pad - block_w } else { vx + pad };
    let y0 = if bottom { vy + vh - pad - block_h } else { vy + pad };

    let mut y = y0;
    if let Some(title) = &cfg.title {
        svg.push_str(r#"<text x=""#);
        push_f64(svg, x0);
        svg.push_str(r#"" y=""#);
        push_f64(svg, y + font);
        svg.push_str(r#"" font-size=""#);
        push_f64(svg, font * 1.1);
        svg.push_str(r#"" font-family="Arial" font-weight="bold" fill="black">"#);
        push_escaped(svg, title);
        svg.push_str("</text>");
        y += title_h;
    }

    for (color, label) in &items {
        svg.push_str(r#"<rect x=""#);
        push_f64(svg, x0);
        svg.push_str(r#"" y=""#);
        push_f64(svg, y);
        svg.push_str(r#"" width=""#);
        push_f64(svg, sw);
        svg.push_str(r#"" height=""#);
        push_f64(svg, sw);
        svg.push_str(r#"" fill=""#);
        push_escaped(svg, color);
        svg.push_str(r##"" stroke="#333" stroke-width=""##);
        push_f64(svg, sw * 0.04);
        svg.push_str(r#""/>"#);

        svg.push_str(r#"<text x=""#);
        push_f64(svg, x0 + sw + gap);
        svg.push_str(r#"" y=""#);
        push_f64(svg, y + sw * 0.78);
        svg.push_str(r#"" font-size=""#);
        push_f64(svg, font);
        svg.push_str(r#"" font-family="Arial" fill="black">"#);
        push_escaped(svg, label);
        svg.push_str("</text>");
        y += row;
    }
}

fn push_escaped(svg: &mut String, s: &str) {
    for c in s.chars() {
        match c {
            '<' => svg.push_str("&lt;"),
            '>' => svg.push_str("&gt;"),
            '&' => svg.push_str("&amp;"),
            _ => svg.push(c),
        }
    }
}
