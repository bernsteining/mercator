use std::borrow::Cow;

use crate::projection::ProjectionConfig;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct LabelLine {
    pub text: String,
    pub font_size: Option<f64>,
    pub color: Option<String>,
    pub font_family: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum LabelConfig {
    /// A bare template string, e.g. `label: "{name}"`.
    Simple(String),
    /// A single styled line, e.g. `label: (text: "{name}", font_size: 0.3)`.
    Single(LabelLine),
    /// Several stacked lines, e.g. `label: ((text: ".."), (text: ".."))`.
    Multi(Vec<LabelLine>),
}

pub struct LabelInstance {
    pub x: f64,
    pub y: f64,
    pub text: String,
    pub font_size: f64,
    pub color: String,
    pub font_family: String,
    pub anchor: &'static str,
    /// Optional halo (outline) drawn behind the text: `(color, width)`.
    pub halo: Option<(String, f64)>,
}

fn default_contour_bandwidth() -> usize { 4 }
fn default_contour_n() -> usize { 6 }
fn default_contour_scheme() -> String { "blues".to_string() }
fn default_contour_stroke_width() -> f64 { 0.01 }

/// Density contours of Point/MultiPoint features (a KDE traced with marching
/// squares). Points are rasterized into a grid, blurred, then iso-lines drawn.
#[derive(Debug, Deserialize)]
pub struct ContourConfig {
    /// Grid cell size in projected units (auto ≈ viewbox/120 when omitted).
    pub cell_size: Option<f64>,
    /// Blur passes controlling smoothness/spread (default 4).
    #[serde(default = "default_contour_bandwidth")]
    pub bandwidth: usize,
    /// Number of contour levels (default 6).
    #[serde(default = "default_contour_n")]
    pub n: usize,
    /// Color scheme for the levels (default `blues`).
    #[serde(default = "default_contour_scheme")]
    pub scheme: String,
    #[serde(default = "default_contour_stroke_width")]
    pub stroke_width: f64,
}

fn default_dorling_iterations() -> usize { 60 }
fn default_dorling_stroke_width() -> f64 { 0.01 }

/// Dorling cartogram: replace each feature with a circle sized by a numeric
/// property and positioned near its centroid, with collision repulsion so the
/// circles don't overlap. Circle fill comes from the feature's resolved `fill`
/// (so a `fill_scale` colors them).
#[derive(Debug, Deserialize)]
pub struct DorlingConfig {
    pub property: String,
    pub max_radius: f64,
    #[serde(default)]
    pub min_radius: f64,
    /// `(min, max)` value domain; auto-computed from the data when omitted.
    pub domain: Option<(f64, f64)>,
    #[serde(default = "default_dorling_iterations")]
    pub iterations: usize,
    pub stroke: Option<String>,
    #[serde(default = "default_dorling_stroke_width")]
    pub stroke_width: f64,
}

fn default_hexbin_scheme() -> String { "blues".to_string() }
fn default_hexbin_n() -> usize { 5 }
fn default_hexbin_stroke_width() -> f64 { 0.01 }

/// Hexagonal binning of Point/MultiPoint features: aggregate points into a hex
/// grid (in projected space) and draw each cell colored by its count.
#[derive(Debug, Deserialize)]
pub struct HexbinConfig {
    /// Hexagon radius in projected/map units.
    pub radius: f64,
    /// Named color scheme for the count ramp (default `blues`).
    #[serde(default = "default_hexbin_scheme")]
    pub scheme: String,
    /// Number of color classes (default 5).
    #[serde(default = "default_hexbin_n")]
    pub n: usize,
    pub stroke: Option<String>,
    #[serde(default = "default_hexbin_stroke_width")]
    pub stroke_width: f64,
}

/// Feature filter: render only features whose `property` satisfies every given
/// condition (`eq`/`ne` any value; `in` a list; `gt`/`lt`/`gte`/`lte` numeric).
#[derive(Debug, Deserialize)]
pub struct FilterConfig {
    pub property: String,
    pub eq: Option<serde_json::Value>,
    pub ne: Option<serde_json::Value>,
    #[serde(rename = "in")]
    pub in_: Option<Vec<serde_json::Value>>,
    pub gt: Option<f64>,
    pub lt: Option<f64>,
    pub gte: Option<f64>,
    pub lte: Option<f64>,
}

impl FilterConfig {
    pub fn matches(&self, props: &serde_json::Map<String, serde_json::Value>) -> bool {
        let v = props.get(&self.property);
        if let Some(eq) = &self.eq {
            if v != Some(eq) {
                return false;
            }
        }
        if let Some(ne) = &self.ne {
            if v == Some(ne) {
                return false;
            }
        }
        if let Some(list) = &self.in_ {
            if !v.map_or(false, |x| list.contains(x)) {
                return false;
            }
        }
        if self.gt.is_some() || self.lt.is_some() || self.gte.is_some() || self.lte.is_some() {
            let n = match v.and_then(serde_json::Value::as_f64) {
                Some(n) => n,
                None => return false,
            };
            if let Some(g) = self.gt {
                if !(n > g) {
                    return false;
                }
            }
            if let Some(l) = self.lt {
                if !(n < l) {
                    return false;
                }
            }
            if let Some(g) = self.gte {
                if !(n >= g) {
                    return false;
                }
            }
            if let Some(l) = self.lte {
                if !(n <= l) {
                    return false;
                }
            }
        }
        true
    }
}

fn default_stroke() -> String { "black".to_string() }
fn default_stroke_width() -> NumOrTemplate { NumOrTemplate::Num(0.05) }
fn default_fill() -> String { "white".to_string() }
fn default_fill_opacity() -> NumOrTemplate { NumOrTemplate::Num(1.0) }

/// A numeric style field that may instead be a `{property}` template resolved per
/// feature (e.g. data-driven `stroke_width` / `fill_opacity`).
#[derive(Debug, Clone)]
pub enum NumOrTemplate {
    Num(f64),
    Template(String),
}

impl NumOrTemplate {
    /// Resolve to a number: templates interpolate against `properties` then parse
    /// (falling back to 0.0 if missing/non-numeric).
    pub fn resolve(&self, properties: Option<&serde_json::Map<String, serde_json::Value>>) -> f64 {
        match self {
            NumOrTemplate::Num(n) => *n,
            NumOrTemplate::Template(t) => properties
                .and_then(|p| interpolate_template(t, p))
                .and_then(|s| s.trim().parse::<f64>().ok())
                .unwrap_or(0.0),
        }
    }
}

impl<'de> Deserialize<'de> for NumOrTemplate {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct V;
        impl<'de> serde::de::Visitor<'de> for V {
            type Value = NumOrTemplate;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("a number or a template string")
            }
            fn visit_f64<E>(self, v: f64) -> Result<NumOrTemplate, E> { Ok(NumOrTemplate::Num(v)) }
            fn visit_i64<E>(self, v: i64) -> Result<NumOrTemplate, E> { Ok(NumOrTemplate::Num(v as f64)) }
            fn visit_u64<E>(self, v: u64) -> Result<NumOrTemplate, E> { Ok(NumOrTemplate::Num(v as f64)) }
            fn visit_str<E>(self, v: &str) -> Result<NumOrTemplate, E> { Ok(NumOrTemplate::Template(v.to_string())) }
        }
        d.deserialize_any(V)
    }
}

#[derive(Debug, Deserialize)]
pub struct StyleConfig {
    #[serde(default = "default_stroke")]
    pub stroke: String,
    #[serde(default = "default_stroke_width")]
    pub stroke_width: NumOrTemplate,
    #[serde(default = "default_fill")]
    pub fill: String,
    #[serde(default = "default_fill_opacity")]
    pub fill_opacity: NumOrTemplate,
    pub viewbox: Option<(f64, f64, f64, f64)>,
    pub viewbox_padding: Option<f64>,
    pub label_color: Option<String>,
    pub label_font_size: Option<f64>,
    pub label_font_family: Option<String>,
    /// Halo/outline color drawn behind label text for legibility over busy maps.
    pub label_halo: Option<String>,
    /// Halo width in map units (default: 0.12 × the label font size).
    pub label_halo_width: Option<f64>,
    /// Drop labels whose bounding box overlaps an already-placed one (first wins).
    #[serde(default)]
    pub label_collide: bool,
    pub label: Option<LabelConfig>,
    /// Only render features whose properties match this filter.
    pub filter: Option<FilterConfig>,
    pub point_radius: Option<f64>,
    pub point_color: Option<String>,
    /// Marker shape for Point/MultiPoint: `circle` (default), `square`,
    /// `diamond`, `triangle`, `cross`, or `star`. Supports `{property}`.
    pub point_shape: Option<String>,
    /// SVG `stroke-dasharray` for lines/polygon borders, e.g. `"4 2"` (map units).
    pub stroke_dash: Option<String>,
    pub fill_pattern: Option<String>,
    pub projection: Option<ProjectionConfig>,
    /// Spherical pre-rotation `[lambda, phi, gamma]` in degrees (d3.geoRotation).
    /// Recenter/tilt/roll any non-azimuthal projection to an oblique aspect.
    pub rotate: Option<Vec<f64>>,
    /// Adaptive-resampling tolerance in projected units (see `RenderOutput`).
    /// When set, straight lon/lat segments are subdivided so they follow the
    /// projection's curve. Off when omitted.
    pub precision: Option<f64>,
    pub graticule: Option<GraticuleConfig>,
    pub tissot: Option<TissotConfig>,
    pub fill_scale: Option<crate::scale::FillScale>,
    pub point_radius_scale: Option<crate::scale::RadiusScale>,
    pub legend: Option<crate::scale::LegendConfig>,
    pub size_legend: Option<crate::scale::LegendConfig>,
    pub sphere: Option<SphereConfig>,
    /// Clip rendered geometry to a projected-space rectangle `[x0, y0, x1, y1]`.
    pub clip_extent: Option<[f64; 4]>,
    /// Clip an azimuthal map to a small circle of this angular radius (degrees).
    pub clip_angle: Option<f64>,
    pub hexbin: Option<HexbinConfig>,
    pub contour: Option<ContourConfig>,
    pub dorling: Option<DorlingConfig>,
    /// Clip polygons at the antimeridian (cylindrical projections) so seam-crossing
    /// shapes close cleanly instead of streaking across the map.
    #[serde(default)]
    pub antimeridian: bool,
}

/// Filled sphere/ocean disc drawn behind the land for azimuthal (globe)
/// projections. Purely cosmetic — hemisphere clipping happens automatically for
/// azimuthal projections whether or not a `sphere` is configured.
#[derive(Debug, Deserialize)]
pub struct SphereConfig {
    #[serde(default = "default_sphere_fill")]
    pub fill: String,
    pub stroke: Option<String>,
    #[serde(default = "default_sphere_stroke_width")]
    pub stroke_width: f64,
}

fn default_sphere_fill() -> String {
    "#cfe8ff".to_string()
}
fn default_sphere_stroke_width() -> f64 {
    0.005
}

#[derive(Debug, Deserialize)]
pub struct GraticuleConfig {
    #[serde(default = "default_graticule_step")]
    pub step: f64,
    #[serde(default = "default_graticule_color")]
    pub color: String,
    #[serde(default = "default_graticule_width")]
    pub width: f64,
    #[serde(default = "default_graticule_opacity")]
    pub opacity: f64,
}

fn default_graticule_step() -> f64 { 15.0 }
fn default_graticule_color() -> String { "#ccc".to_string() }
fn default_graticule_width() -> f64 { 0.5 }
fn default_graticule_opacity() -> f64 { 0.6 }

#[derive(Debug, Deserialize)]
pub struct TissotConfig {
    #[serde(default = "default_tissot_step")]
    pub step: f64,
    #[serde(default = "default_tissot_radius")]
    pub radius: f64,
    #[serde(default = "default_tissot_fill")]
    pub fill: String,
    #[serde(default = "default_tissot_fill_opacity")]
    pub fill_opacity: f64,
    #[serde(default = "default_tissot_stroke")]
    pub stroke: String,
    #[serde(default = "default_tissot_stroke_width")]
    pub stroke_width: f64,
    #[serde(default = "default_tissot_max_lat")]
    pub max_lat: f64,
}

fn default_tissot_step() -> f64 { 30.0 }
fn default_tissot_radius() -> f64 { 5.0 }
fn default_tissot_fill() -> String { "red".to_string() }
fn default_tissot_fill_opacity() -> f64 { 0.3 }
fn default_tissot_stroke() -> String { "red".to_string() }
fn default_tissot_stroke_width() -> f64 { 0.5 }
fn default_tissot_max_lat() -> f64 { 60.0 }

impl Default for StyleConfig {
    fn default() -> Self {
        Self {
            stroke: "black".to_string(),
            stroke_width: NumOrTemplate::Num(0.05),
            fill: "white".to_string(),
            fill_opacity: NumOrTemplate::Num(1.0),
            viewbox: None,
            viewbox_padding: None,
            label_color: Some("black".to_string()),
            label_font_size: Some(0.3),
            label_font_family: Some("Arial".to_string()),
            label_halo: None,
            label_halo_width: None,
            label_collide: false,
            label: None,
            filter: None,
            rotate: None,
            precision: None,
            point_radius: None,
            point_color: None,
            point_shape: None,
            stroke_dash: None,
            fill_pattern: None,
            projection: None,
            graticule: None,
            tissot: None,
            fill_scale: None,
            point_radius_scale: None,
            legend: None,
            size_legend: None,
            sphere: None,
            clip_extent: None,
            clip_angle: None,
            hexbin: None,
            contour: None,
            dorling: None,
            antimeridian: false,
        }
    }
}

pub struct ResolvedStyle<'a> {
    pub stroke: Cow<'a, str>,
    pub stroke_width: f64,
    pub fill: Cow<'a, str>,
    pub fill_opacity: f64,
    pub point_radius: f64,
    pub point_color: Option<Cow<'a, str>>,
    pub point_shape: Cow<'a, str>,
    pub stroke_dash: Option<Cow<'a, str>>,
    pub fill_pattern: Option<Cow<'a, str>>,
}

fn resolve_field<'a>(
    template: &'a str,
    fallback: &'a str,
    properties: Option<&serde_json::Map<String, serde_json::Value>>,
) -> Cow<'a, str> {
    if !template.contains('{') {
        return Cow::Borrowed(template);
    }
    if let Some(props) = properties {
        if let Some(val) = interpolate_template(template, props) {
            return Cow::Owned(val);
        }
    }
    Cow::Borrowed(fallback)
}

pub fn resolve_style<'a>(
    config: &'a StyleConfig,
    properties: Option<&serde_json::Map<String, serde_json::Value>>,
) -> ResolvedStyle<'a> {
    let fill_pattern = config.fill_pattern.as_deref().and_then(|template| {
        let resolved = resolve_field(template, "", properties);
        if resolved.is_empty() { None } else { Some(resolved) }
    });

    let point_color = config.point_color.as_deref().map(|template| {
        resolve_field(template, "none", properties)
    });

    let point_shape = config
        .point_shape
        .as_deref()
        .map(|template| resolve_field(template, "circle", properties))
        .unwrap_or(Cow::Borrowed("circle"));

    let stroke_width = config.stroke_width.resolve(properties);
    ResolvedStyle {
        stroke: resolve_field(&config.stroke, "black", properties),
        stroke_width,
        fill: resolve_field(&config.fill, "none", properties),
        fill_opacity: config.fill_opacity.resolve(properties),
        point_radius: config.point_radius.unwrap_or(stroke_width * 5.0),
        point_color,
        point_shape,
        stroke_dash: config.stroke_dash.as_deref().map(|t| resolve_field(t, "", properties)),
        fill_pattern,
    }
}

pub fn interpolate_template(
    template: &str,
    properties: &serde_json::Map<String, serde_json::Value>,
) -> Option<String> {
    let mut result = String::with_capacity(template.len());
    let bytes = template.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] == b'{' {
            i += 1;
            let key_start = i;
            while i < bytes.len() && bytes[i] != b'}' {
                i += 1;
            }
            let key = &template[key_start..i];
            let val = properties.get(key)?;
            match val {
                serde_json::Value::String(s) => result.push_str(s),
                serde_json::Value::Number(n) => {
                    use std::fmt::Write;
                    let _ = write!(result, "{}", n);
                }
                serde_json::Value::Bool(b) => {
                    result.push_str(if *b { "true" } else { "false" });
                }
                _ => return None,
            }
            if i < bytes.len() {
                i += 1; // skip '}'
            }
        } else {
            result.push(bytes[i] as char);
            i += 1;
        }
    }

    if result.is_empty() {
        None
    } else {
        Some(result)
    }
}
