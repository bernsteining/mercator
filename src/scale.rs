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

/// `fill_scale` config: bind `property` to a color `range`/`scheme` via a scale `type`.
#[derive(Debug, Deserialize)]
pub struct FillScale {
    pub property: String,
    /// `quantize` | `quantile` | `threshold` | `linear` | `diverging` | `category`.
    #[serde(rename = "type", default = "default_scale_type")]
    pub scale_type: String,
    /// `(min, max)`. Auto-computed from the data when omitted (`quantize`/`linear`/…).
    pub domain: Option<(f64, f64)>,
    /// Colors: discrete bins for `quantize`/`quantile`/`threshold`, gradient stops
    /// for `linear`/`diverging`. Omit and set `scheme` to use a named palette.
    #[serde(default)]
    pub range: Vec<String>,
    /// Named color scheme (e.g. `blues`, `viridis`, `rdbu`, `spectral`, `tableau10`)
    /// used when `range` is empty. See `schemes` below.
    pub scheme: Option<String>,
    /// Number of classes to sample from a `scheme` for discrete scales
    /// (`quantize`/`quantile`). Defaults to 5.
    pub n: Option<usize>,
    /// Explicit break points for `type: "threshold"` (N breaks → N+1 colors).
    pub breaks: Option<Vec<f64>>,
    /// Midpoint anchored to the middle color for `type: "diverging"`
    /// (default: the domain's midpoint).
    pub midpoint: Option<f64>,
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
    pub fn radius_for(&self, props: &Map<String, Value>) -> f64 {
        match props.get(&self.property).and_then(Value::as_f64) {
            Some(v) => self.radius_for_value(v),
            None => self.min_r,
        }
    }

    /// Radius for a raw numeric value. `sqrt` scaling interpolates in √value
    /// space so symbol *area* tracks the value (the perceptual default).
    fn radius_for_value(&self, v: f64) -> f64 {
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

    /// The outer radius, and representative (value, radius) samples for a size key.
    pub fn max_radius(&self) -> f64 {
        self.max_r
    }
    pub fn legend_samples(&self) -> Vec<(f64, f64)> {
        [self.max_v, (self.min_v + self.max_v) / 2.0, self.min_v]
            .iter()
            .map(|&v| (v, self.radius_for_value(v)))
            .collect()
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
    /// Equal-count bins: `thresholds` (len = colors-1) split the sorted data so
    /// each bin holds ~the same number of features. Robust to outliers.
    Quantile { thresholds: Vec<f64>, colors: Vec<String> },
    /// Explicit break points: `breaks` (len = colors-1) → `colors[bin]`.
    Threshold { breaks: Vec<f64>, colors: Vec<String> },
    /// Interpolate RGB between evenly-spaced `stops` over `[min, max]`.
    Linear { min: f64, max: f64, stops: Vec<[f64; 3]> },
    /// Like `Linear` but `mid` is pinned to the middle stop (for +/- data).
    Diverging { min: f64, mid: f64, max: f64, stops: Vec<[f64; 3]> },
    /// Map a categorical value to a color. `lookup` resolves; `order` drives the legend.
    Category { lookup: HashMap<String, String>, order: Vec<(String, String)> },
}

/// Named color schemes as anchor stops: sampled for discrete N-class ramps, used
/// directly as gradient stops for `linear`/`diverging`, cycled for `category`.
fn scheme_stops(name: &str) -> Option<&'static [&'static str]> {
    Some(match name.to_ascii_lowercase().as_str() {
        // sequential
        "blues" => &["#f7fbff", "#c6dbef", "#6baed6", "#2171b5", "#08306b"],
        "greens" => &["#f7fcf5", "#c7e9c0", "#74c476", "#238b45", "#00441b"],
        "oranges" => &["#fff5eb", "#fdd0a2", "#fd8d3c", "#d94801", "#7f2704"],
        "reds" => &["#fff5f0", "#fcbba1", "#fb6a4a", "#cb181d", "#67000d"],
        "purples" => &["#fcfbfd", "#dadaeb", "#9e9ac8", "#6a51a3", "#3f007d"],
        "greys" | "grays" => &["#ffffff", "#d9d9d9", "#969696", "#525252", "#000000"],
        "viridis" => &["#440154", "#414487", "#2a788e", "#22a884", "#7ad151", "#fde725"],
        "magma" => &["#000004", "#3b0f70", "#8c2981", "#de4968", "#fe9f6d", "#fcfdbf"],
        "ylgnbu" => &["#ffffd9", "#c7e9b4", "#41b6c4", "#225ea8", "#081d58"],
        "ylorrd" => &["#ffffcc", "#fed976", "#fd8d3c", "#e31a1c", "#800026"],
        // diverging
        "rdbu" => &["#b2182b", "#ef8a62", "#fddbc7", "#f7f7f7", "#d1e5f0", "#67a9cf", "#2166ac"],
        "rdylbu" => &["#d73027", "#fc8d59", "#fee090", "#ffffbf", "#e0f3f8", "#91bfdb", "#4575b4"],
        "brbg" => &["#8c510a", "#d8b365", "#f6e8c3", "#f5f5f5", "#c7eae5", "#5ab4ac", "#01665e"],
        "piyg" => &["#c51b7d", "#e9a3c9", "#fde0ef", "#f7f7f7", "#e6f5d0", "#a1d76a", "#4d9221"],
        "spectral" => &["#d53e4f", "#fc8d59", "#fee08b", "#ffffbf", "#e6f598", "#99d594", "#3288bd"],
        // categorical
        "category10" | "tableau10" => &["#4e79a7", "#f28e2c", "#e15759", "#76b7b2", "#59a14f", "#edc949", "#af7aa1", "#ff9da7", "#9c755f", "#bab0ab"],
        "set1" => &["#e41a1c", "#377eb8", "#4daf4a", "#984ea3", "#ff7f00", "#ffff33", "#a65628", "#f781bf"],
        "set2" => &["#66c2a5", "#fc8d62", "#8da0cb", "#e78ac3", "#a6d854", "#ffd92f", "#e5c494"],
        "dark2" => &["#1b9e77", "#d95f02", "#7570b3", "#e7298a", "#66a61e", "#e6ab02", "#a6761d"],
        _ => return None,
    })
}

/// Interpolate a piecewise-linear RGB gradient of `stops` at `t` in `[0,1]`.
fn interp_stops(stops: &[[f64; 3]], t: f64) -> String {
    if stops.is_empty() {
        return "#cccccc".to_string();
    }
    if stops.len() == 1 {
        let a = stops[0];
        return format!("#{:02x}{:02x}{:02x}", a[0] as u8, a[1] as u8, a[2] as u8);
    }
    let t = t.clamp(0.0, 1.0);
    let seg = (stops.len() - 1) as f64;
    let pos = t * seg;
    let i = (pos.floor() as usize).min(stops.len() - 2);
    let f = pos - i as f64;
    let (a, b) = (stops[i], stops[i + 1]);
    let ch = |k: usize| (a[k] + (b[k] - a[k]) * f).round() as u8;
    format!("#{:02x}{:02x}{:02x}", ch(0), ch(1), ch(2))
}

/// N discrete colors sampled evenly across a stop gradient (for class ramps).
fn sample_gradient(stops: &[[f64; 3]], n: usize) -> Vec<String> {
    (0..n)
        .map(|i| interp_stops(stops, if n <= 1 { 0.5 } else { i as f64 / (n - 1) as f64 }))
        .collect()
}

/// Parse a scheme name into its anchor stops as RGB triples.
fn scheme_as_stops(name: &str) -> Option<Vec<[f64; 3]>> {
    scheme_stops(name).map(|s| s.iter().filter_map(|c| parse_hex(c)).collect())
}

/// `n` discrete colors sampled from a named scheme (for e.g. hexbin counts).
pub fn scheme_colors(name: &str, n: usize) -> Option<Vec<String>> {
    scheme_as_stops(name).map(|stops| sample_gradient(&stops, n.max(1)))
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
            } else {
                // Palette from explicit `palette` or a named `scheme`; cycled over
                // distinct values in first-seen order.
                let palette: Vec<String> = if let Some(p) = &cfg.palette {
                    p.clone()
                } else if let Some(sch) = &cfg.scheme {
                    scheme_stops(sch).map(|s| s.iter().map(|c| c.to_string()).collect()).unwrap_or_default()
                } else {
                    return None; // category needs `categories`, `palette`, or `scheme`
                };
                if palette.is_empty() {
                    return None;
                }
                for key in distinct_keys(gj, &cfg.property) {
                    let color = palette[order.len() % palette.len()].clone();
                    lookup.insert(key.clone(), color.clone());
                    order.push((key, color));
                }
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

        // Discrete colors: explicit `range`, else `n` sampled from a named `scheme`.
        let discrete_colors = |n: usize| -> Vec<String> {
            if !cfg.range.is_empty() {
                cfg.range.clone()
            } else if let Some(stops) = cfg.scheme.as_deref().and_then(scheme_as_stops) {
                sample_gradient(&stops, n)
            } else {
                Vec::new()
            }
        };
        // Gradient stops: explicit `range`, else a named `scheme`'s anchors.
        let gradient_stops = || -> Vec<[f64; 3]> {
            if !cfg.range.is_empty() {
                cfg.range.iter().filter_map(|c| parse_hex(c)).collect()
            } else if let Some(stops) = cfg.scheme.as_deref().and_then(scheme_as_stops) {
                stops
            } else {
                Vec::new()
            }
        };
        let n_classes = cfg.n.unwrap_or(5).max(1);

        // Types that don't need a [min,max] domain:
        if cfg.scale_type == "threshold" {
            let breaks = cfg.breaks.clone().unwrap_or_default();
            if breaks.is_empty() {
                return None; // threshold needs explicit `breaks`
            }
            let colors = discrete_colors(breaks.len() + 1);
            if colors.is_empty() {
                return None;
            }
            return Some(ColorScale { property: cfg.property.clone(), default, kind: Kind::Threshold { breaks, colors } });
        }
        if cfg.scale_type == "quantile" {
            let colors = discrete_colors(n_classes);
            if colors.is_empty() {
                return None;
            }
            let mut vals: Vec<f64> = feature_values(gj, &cfg.property).collect();
            if vals.is_empty() {
                return None;
            }
            vals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            let nb = colors.len();
            // Interpolated quantile break at each bin boundary of the sorted data.
            let thresholds: Vec<f64> = (1..nb)
                .map(|i| {
                    let p = i as f64 / nb as f64 * (vals.len() - 1) as f64;
                    let lo = p.floor() as usize;
                    let hi = (lo + 1).min(vals.len() - 1);
                    vals[lo] + (vals[hi] - vals[lo]) * (p - lo as f64)
                })
                .collect();
            return Some(ColorScale { property: cfg.property.clone(), default, kind: Kind::Quantile { thresholds, colors } });
        }

        // quantize / linear / diverging need a numeric [min, max] domain.
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
            let stops = gradient_stops();
            if stops.len() < 2 {
                Kind::Quantize { min, max, colors: discrete_colors(n_classes) }
            } else {
                Kind::Linear { min, max, stops }
            }
        } else if cfg.scale_type == "diverging" {
            let stops = gradient_stops();
            if stops.len() < 2 {
                return None;
            }
            let mid = cfg.midpoint.unwrap_or((min + max) / 2.0);
            Kind::Diverging { min, mid, max, stops }
        } else {
            let colors = discrete_colors(n_classes);
            if colors.is_empty() {
                return None;
            }
            Kind::Quantize { min, max, colors }
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
            Kind::Quantile { thresholds, colors } | Kind::Threshold { breaks: thresholds, colors } => {
                // First threshold the value falls below picks the bin.
                let idx = thresholds.iter().position(|&t| value < t).unwrap_or(colors.len() - 1);
                colors[idx.min(colors.len() - 1)].clone()
            }
            Kind::Linear { min, max, stops } => interp_stops(stops, Self::norm(*min, *max, value)),
            Kind::Diverging { min, mid, max, stops } => {
                // Pin `mid` to t = 0.5 so the middle stop sits at the midpoint.
                let t = if value <= *mid {
                    if mid > min { 0.5 * (value - min) / (mid - min) } else { 0.0 }
                } else if max > mid {
                    0.5 + 0.5 * (value - mid) / (max - mid)
                } else {
                    1.0
                };
                interp_stops(stops, t.clamp(0.0, 1.0))
            }
            // Handled by the early return above; kept for exhaustiveness.
            Kind::Category { .. } => self.default.clone(),
        }
    }

    /// `(min, max)` for continuous scales (linear/diverging), else `None`.
    fn continuous_domain(&self) -> Option<(f64, f64)> {
        match &self.kind {
            Kind::Linear { min, max, .. } | Kind::Diverging { min, max, .. } => Some((*min, *max)),
            _ => None,
        }
    }

    /// The color at a numeric value (used to sample the gradient bar).
    fn color_at(&self, value: f64) -> String {
        let mut m = Map::new();
        m.insert(self.property.clone(), Value::from(value));
        self.color_for(&m)
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
            Kind::Quantile { thresholds, colors } | Kind::Threshold { breaks: thresholds, colors } => {
                // Bins delimited by the thresholds: "< t0", "t0 – t1", …, "≥ tn".
                (0..colors.len())
                    .map(|i| {
                        let label = if colors.len() == 1 {
                            "all".to_string()
                        } else if i == 0 {
                            format!("< {}", fmt_num(thresholds[0]))
                        } else if i == colors.len() - 1 {
                            format!("≥ {}", fmt_num(thresholds[i - 1]))
                        } else {
                            format!("{} – {}", fmt_num(thresholds[i - 1]), fmt_num(thresholds[i]))
                        };
                        (colors[i].clone(), label)
                    })
                    .collect()
            }
            Kind::Linear { min, max, .. } | Kind::Diverging { min, max, .. } => {
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
    // Continuous scales get a smooth gradient bar instead of discrete swatches.
    if let Some((min, max)) = scale.continuous_domain() {
        render_gradient_legend(svg, scale, cfg, viewbox, min, max);
        return;
    }

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

/// Continuous gradient bar legend for linear/diverging scales, with min/mid/max
/// tick labels. Drawn with an SVG `linearGradient` sampled from the scale.
fn render_gradient_legend(
    svg: &mut String,
    scale: &ColorScale,
    cfg: &LegendConfig,
    viewbox: (f64, f64, f64, f64),
    min: f64,
    max: f64,
) {
    let (vx, vy, vw, vh) = viewbox;
    let unit = vw.min(vh);
    let bar_w = unit * 0.03;
    let bar_h = unit * 0.30;
    let font = unit * 0.028;
    let pad = unit * 0.02;
    let gap = font * 0.5;
    let title_h = if cfg.title.is_some() { font * 1.5 } else { 0.0 };
    let block_w = bar_w + gap + font * 4.0;
    let block_h = title_h + bar_h;

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

    // Gradient definition: offset 0 (bottom) = min, offset 1 (top) = max.
    svg.push_str(r#"<defs><linearGradient id="mgrad" x1="0" y1="1" x2="0" y2="0">"#);
    let n = 12;
    for i in 0..=n {
        let t = i as f64 / n as f64;
        svg.push_str(r#"<stop offset=""#);
        push_f64(svg, t);
        svg.push_str(r#"" stop-color=""#);
        push_escaped(svg, &scale.color_at(min + (max - min) * t));
        svg.push_str(r#""/>"#);
    }
    svg.push_str("</linearGradient></defs>");

    svg.push_str(r#"<rect x=""#);
    push_f64(svg, x0);
    svg.push_str(r#"" y=""#);
    push_f64(svg, y);
    svg.push_str(r#"" width=""#);
    push_f64(svg, bar_w);
    svg.push_str(r#"" height=""#);
    push_f64(svg, bar_h);
    svg.push_str(r##"" fill="url(#mgrad)" stroke="#333" stroke-width=""##);
    push_f64(svg, bar_w * 0.04);
    svg.push_str(r#""/>"#);

    // Tick labels: max (top), midpoint, min (bottom).
    for (frac, val) in [(0.0, max), (0.5, (min + max) / 2.0), (1.0, min)] {
        svg.push_str(r#"<text x=""#);
        push_f64(svg, x0 + bar_w + gap);
        svg.push_str(r#"" y=""#);
        push_f64(svg, y + bar_h * frac + font * 0.35);
        svg.push_str(r#"" font-size=""#);
        push_f64(svg, font);
        svg.push_str(r#"" font-family="Arial" fill="black">"#);
        push_escaped(svg, &fmt_num(val));
        svg.push_str("</text>");
    }
}

/// Nested-circle size key for a proportional-symbol map (`point_radius_scale`).
/// Circles share a bottom tangent (largest behind), each with a value label.
pub fn render_size_legend(
    svg: &mut String,
    size: &SizeScale,
    cfg: &LegendConfig,
    viewbox: (f64, f64, f64, f64),
) {
    let rmax = size.max_radius();
    let samples = size.legend_samples();
    if rmax <= 0.0 || samples.is_empty() {
        return;
    }
    let (vx, vy, vw, vh) = viewbox;
    let unit = vw.min(vh);
    let font = unit * 0.028;
    let pad = unit * 0.02;
    let gap = font * 0.5;
    let title_h = if cfg.title.is_some() { font * 1.5 } else { 0.0 };
    let block_w = 2.0 * rmax + gap + font * 4.0;
    let block_h = title_h + 2.0 * rmax;

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

    let baseline = y + 2.0 * rmax;
    let cx = x0 + rmax;
    let sw = (rmax * 0.03).max(unit * 0.001);
    for (val, r) in &samples {
        if *r <= 0.0 {
            continue;
        }
        svg.push_str(r#"<circle cx=""#);
        push_f64(svg, cx);
        svg.push_str(r#"" cy=""#);
        push_f64(svg, baseline - r);
        svg.push_str(r#"" r=""#);
        push_f64(svg, *r);
        svg.push_str(r##"" fill="none" stroke="#555" stroke-width=""##);
        push_f64(svg, sw);
        svg.push_str(r#""/>"#);

        let ty = baseline - 2.0 * r;
        svg.push_str(r##"<line stroke="#aaa" stroke-width=""##);
        push_f64(svg, sw);
        svg.push_str(r#"" x1=""#);
        push_f64(svg, cx);
        svg.push_str(r#"" y1=""#);
        push_f64(svg, ty);
        svg.push_str(r#"" x2=""#);
        push_f64(svg, x0 + 2.0 * rmax + gap * 0.6);
        svg.push_str(r#"" y2=""#);
        push_f64(svg, ty);
        svg.push_str(r#""/>"#);

        svg.push_str(r#"<text x=""#);
        push_f64(svg, x0 + 2.0 * rmax + gap);
        svg.push_str(r#"" y=""#);
        push_f64(svg, ty + font * 0.35);
        svg.push_str(r#"" font-size=""#);
        push_f64(svg, font);
        svg.push_str(r#"" font-family="Arial" fill="black">"#);
        push_escaped(svg, &fmt_num(*val));
        svg.push_str("</text>");
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
