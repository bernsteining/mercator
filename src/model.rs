//! Lightweight GeoJSON model tailored to this renderer, with a fast custom parser.
//!
//! Unlike the `geojson` crate — which stores every coordinate as its own heap
//! `Vec<f64>` — positions here are inline `Pos` values packed contiguously in
//! their parent `Vec`. For a 44 MB / 2.2 M-coordinate file this cut parse time
//! ~4x (measured) and improves render cache locality.
//!
//! Parsing assumes the conventional key order (`"type"` before `"coordinates"`),
//! which every real-world GeoJSON producer emits, but falls back to a buffered
//! decode if a geometry lists `coordinates` before `type`, so all valid inputs
//! are still accepted. A third `z`/altitude ordinate, if present, is ignored.

use std::fmt;

use serde::de::{self, Deserializer, IgnoredAny, MapAccess, SeqAccess, Visitor};
use serde::Deserialize;

/// A 2D position. Any extra ordinates (altitude) in the source are dropped.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pos {
    pub x: f64,
    pub y: f64,
}

impl<'de> Deserialize<'de> for Pos {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct PosVisitor;
        impl<'de> Visitor<'de> for PosVisitor {
            type Value = Pos;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a coordinate array [x, y, ...]")
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Pos, A::Error> {
                let x = seq
                    .next_element::<f64>()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let y = seq
                    .next_element::<f64>()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                // Ignore any further ordinates (z / m).
                while seq.next_element::<IgnoredAny>()?.is_some() {}
                Ok(Pos { x, y })
            }
        }
        d.deserialize_seq(PosVisitor)
    }
}

/// GeoJSON geometry. Coordinates are inline; `GeometryCollection` nests.
#[derive(Debug)]
pub enum Geometry {
    Point(Pos),
    MultiPoint(Vec<Pos>),
    LineString(Vec<Pos>),
    MultiLineString(Vec<Vec<Pos>>),
    Polygon(Vec<Vec<Pos>>),
    MultiPolygon(Vec<Vec<Vec<Pos>>>),
    GeometryCollection(Vec<Geometry>),
}

/// Build a coordinate geometry from an already-known `type` tag, reading the
/// `coordinates` value directly off the map (streaming — no intermediate DOM).
fn coords_streaming<'de, A: MapAccess<'de>>(ty: &str, map: &mut A) -> Result<Geometry, A::Error> {
    Ok(match ty {
        "Point" => Geometry::Point(map.next_value()?),
        "MultiPoint" => Geometry::MultiPoint(map.next_value()?),
        "LineString" => Geometry::LineString(map.next_value()?),
        "MultiLineString" => Geometry::MultiLineString(map.next_value()?),
        "Polygon" => Geometry::Polygon(map.next_value()?),
        "MultiPolygon" => Geometry::MultiPolygon(map.next_value()?),
        other => return Err(de::Error::custom(format!("unknown geometry type: {other}"))),
    })
}

/// Same, but from a buffered JSON value (used only when `coordinates` preceded
/// `type` in the object — rare, so the extra allocation is acceptable).
fn coords_buffered<E: de::Error>(ty: &str, v: serde_json::Value) -> Result<Geometry, E> {
    let map_err = |e: serde_json::Error| de::Error::custom(e);
    Ok(match ty {
        "Point" => Geometry::Point(serde_json::from_value(v).map_err(map_err)?),
        "MultiPoint" => Geometry::MultiPoint(serde_json::from_value(v).map_err(map_err)?),
        "LineString" => Geometry::LineString(serde_json::from_value(v).map_err(map_err)?),
        "MultiLineString" => Geometry::MultiLineString(serde_json::from_value(v).map_err(map_err)?),
        "Polygon" => Geometry::Polygon(serde_json::from_value(v).map_err(map_err)?),
        "MultiPolygon" => Geometry::MultiPolygon(serde_json::from_value(v).map_err(map_err)?),
        other => return Err(de::Error::custom(format!("unknown geometry type: {other}"))),
    })
}

impl<'de> Deserialize<'de> for Geometry {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct GeometryVisitor;
        impl<'de> Visitor<'de> for GeometryVisitor {
            type Value = Geometry;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a GeoJSON geometry object")
            }
            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Geometry, A::Error> {
                let mut ty: Option<String> = None;
                let mut geom: Option<Geometry> = None;
                let mut coords_buf: Option<serde_json::Value> = None;

                while let Some(key) = map.next_key::<&str>()? {
                    match key {
                        "type" => ty = Some(map.next_value()?),
                        "coordinates" => match ty.as_deref() {
                            Some(t) => geom = Some(coords_streaming(t, &mut map)?),
                            None => coords_buf = Some(map.next_value()?),
                        },
                        "geometries" => {
                            geom = Some(Geometry::GeometryCollection(map.next_value()?));
                        }
                        _ => {
                            map.next_value::<IgnoredAny>()?;
                        }
                    }
                }

                if let Some(g) = geom {
                    return Ok(g);
                }
                match (ty.as_deref(), coords_buf) {
                    (Some(t), Some(v)) => coords_buffered(t, v),
                    _ => Err(de::Error::custom("geometry missing coordinates/geometries")),
                }
            }
        }
        d.deserialize_map(GeometryVisitor)
    }
}

/// A GeoJSON feature id (string or number).
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum FeatureId {
    Number(serde_json::Number),
    String(String),
}

/// A GeoJSON feature: an optional geometry plus untyped properties and id.
#[derive(Debug, Default)]
pub struct Feature {
    pub geometry: Option<Geometry>,
    pub properties: Option<serde_json::Map<String, serde_json::Value>>,
    pub id: Option<FeatureId>,
}

impl<'de> Deserialize<'de> for Feature {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct FeatureVisitor;
        impl<'de> Visitor<'de> for FeatureVisitor {
            type Value = Feature;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a GeoJSON feature object")
            }
            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Feature, A::Error> {
                let mut feat = Feature::default();
                while let Some(key) = map.next_key::<&str>()? {
                    match key {
                        "geometry" => feat.geometry = map.next_value()?,
                        "properties" => feat.properties = map.next_value()?,
                        "id" => feat.id = map.next_value()?,
                        _ => {
                            map.next_value::<IgnoredAny>()?;
                        }
                    }
                }
                Ok(feat)
            }
        }
        d.deserialize_map(FeatureVisitor)
    }
}

/// A top-level GeoJSON object.
#[derive(Debug)]
pub enum GeoJson {
    FeatureCollection(Vec<Feature>),
    Feature(Feature),
    Geometry(Geometry),
}

impl<'de> Deserialize<'de> for GeoJson {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct GeoJsonVisitor;
        impl<'de> Visitor<'de> for GeoJsonVisitor {
            type Value = GeoJson;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a GeoJSON object")
            }
            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<GeoJson, A::Error> {
                let mut ty: Option<String> = None;
                let mut features: Option<Vec<Feature>> = None;
                let mut feat_geometry: Option<Geometry> = None;
                let mut properties = None;
                let mut id = None;
                let mut bare_geometry: Option<Geometry> = None;
                let mut coords_buf: Option<serde_json::Value> = None;

                while let Some(key) = map.next_key::<&str>()? {
                    match key {
                        "type" => ty = Some(map.next_value()?),
                        "features" => features = Some(map.next_value()?),
                        "geometry" => feat_geometry = map.next_value()?,
                        "properties" => properties = map.next_value()?,
                        "id" => id = map.next_value()?,
                        "coordinates" => match ty.as_deref() {
                            Some(t) => bare_geometry = Some(coords_streaming(t, &mut map)?),
                            None => coords_buf = Some(map.next_value()?),
                        },
                        "geometries" => {
                            bare_geometry = Some(Geometry::GeometryCollection(map.next_value()?));
                        }
                        _ => {
                            map.next_value::<IgnoredAny>()?;
                        }
                    }
                }

                match ty.as_deref() {
                    Some("FeatureCollection") => {
                        Ok(GeoJson::FeatureCollection(features.unwrap_or_default()))
                    }
                    Some("Feature") => Ok(GeoJson::Feature(Feature {
                        geometry: feat_geometry,
                        properties,
                        id,
                    })),
                    Some(t) => {
                        let g = match bare_geometry {
                            Some(g) => g,
                            None => match coords_buf {
                                Some(v) => coords_buffered(t, v)?,
                                None => {
                                    return Err(de::Error::custom(
                                        "geometry missing coordinates/geometries",
                                    ))
                                }
                            },
                        };
                        Ok(GeoJson::Geometry(g))
                    }
                    None => Err(de::Error::missing_field("type")),
                }
            }
        }
        d.deserialize_map(GeoJsonVisitor)
    }
}

/// Parse GeoJSON bytes into the model.
pub fn parse(data: &[u8]) -> Result<GeoJson, serde_json::Error> {
    serde_json::from_slice(data)
}
