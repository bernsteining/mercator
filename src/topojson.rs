//! Minimal TopoJSON decoder: parses a Topology and stitches its arcs into the
//! GeoJSON [`model`] this renderer consumes. Replaces the `topojson` + `geojson`
//! crates for the fallback path.
//!
//! Algorithm follows the TopoJSON spec / topojson-client: delta-decode each arc
//! once (applying the optional quantization transform), then a geometry's rings
//! concatenate their referenced arcs, reversing an arc when its index is negative
//! (`~i` = `-i - 1`). Point/MultiPoint coordinates are only scaled/translated.

use std::collections::HashMap;
use std::fmt;

use serde::de::{self, Deserializer, IgnoredAny, MapAccess, Visitor};
use serde::Deserialize;

use crate::model::{Feature, FeatureId, GeoJson, Geometry, Pos};

#[derive(Deserialize)]
struct Transform {
    scale: [f64; 2],
    translate: [f64; 2],
}

/// A TopoJSON geometry: arc-index references (or inline points) plus metadata.
enum TopoValue {
    Point(Pos),
    MultiPoint(Vec<Pos>),
    LineString(Vec<i32>),
    MultiLineString(Vec<Vec<i32>>),
    Polygon(Vec<Vec<i32>>),
    MultiPolygon(Vec<Vec<Vec<i32>>>),
    GeometryCollection(Vec<TopoGeometry>),
}

struct TopoGeometry {
    value: TopoValue,
    properties: Option<serde_json::Map<String, serde_json::Value>>,
    id: Option<FeatureId>,
}

fn topo_value_streaming<'de, A: MapAccess<'de>>(
    ty: &str,
    map: &mut A,
    coords: bool,
) -> Result<TopoValue, A::Error> {
    Ok(match (ty, coords) {
        ("Point", true) => TopoValue::Point(map.next_value()?),
        ("MultiPoint", true) => TopoValue::MultiPoint(map.next_value()?),
        ("LineString", false) => TopoValue::LineString(map.next_value()?),
        ("MultiLineString", false) => TopoValue::MultiLineString(map.next_value()?),
        ("Polygon", false) => TopoValue::Polygon(map.next_value()?),
        ("MultiPolygon", false) => TopoValue::MultiPolygon(map.next_value()?),
        (other, _) => return Err(de::Error::custom(format!("unexpected topology geometry: {other}"))),
    })
}

fn topo_value_buffered<E: de::Error>(
    ty: &str,
    coords: Option<serde_json::Value>,
    arcs: Option<serde_json::Value>,
) -> Result<TopoValue, E> {
    let err = |e: serde_json::Error| de::Error::custom(e);
    Ok(match ty {
        "Point" => TopoValue::Point(serde_json::from_value(need(coords)?).map_err(err)?),
        "MultiPoint" => TopoValue::MultiPoint(serde_json::from_value(need(coords)?).map_err(err)?),
        "LineString" => TopoValue::LineString(serde_json::from_value(need(arcs)?).map_err(err)?),
        "MultiLineString" => {
            TopoValue::MultiLineString(serde_json::from_value(need(arcs)?).map_err(err)?)
        }
        "Polygon" => TopoValue::Polygon(serde_json::from_value(need(arcs)?).map_err(err)?),
        "MultiPolygon" => TopoValue::MultiPolygon(serde_json::from_value(need(arcs)?).map_err(err)?),
        other => return Err(de::Error::custom(format!("unexpected topology geometry: {other}"))),
    })
}

fn need<T, E: de::Error>(v: Option<T>) -> Result<T, E> {
    v.ok_or_else(|| de::Error::custom("topology geometry missing coordinates/arcs"))
}

impl<'de> Deserialize<'de> for TopoGeometry {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct V;
        impl<'de> Visitor<'de> for V {
            type Value = TopoGeometry;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a TopoJSON geometry object")
            }
            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<TopoGeometry, A::Error> {
                let mut ty: Option<String> = None;
                let mut value: Option<TopoValue> = None;
                let mut properties = None;
                let mut id = None;
                let mut coords_buf: Option<serde_json::Value> = None;
                let mut arcs_buf: Option<serde_json::Value> = None;

                while let Some(key) = map.next_key::<&str>()? {
                    match key {
                        "type" => ty = Some(map.next_value()?),
                        "coordinates" => match ty.as_deref() {
                            Some(t) => value = Some(topo_value_streaming(t, &mut map, true)?),
                            None => coords_buf = Some(map.next_value()?),
                        },
                        "arcs" => match ty.as_deref() {
                            Some(t) => value = Some(topo_value_streaming(t, &mut map, false)?),
                            None => arcs_buf = Some(map.next_value()?),
                        },
                        "geometries" => {
                            value = Some(TopoValue::GeometryCollection(map.next_value()?));
                        }
                        "properties" => properties = map.next_value()?,
                        "id" => id = map.next_value()?,
                        _ => {
                            map.next_value::<IgnoredAny>()?;
                        }
                    }
                }

                let value = match value {
                    Some(v) => v,
                    None => match ty.as_deref() {
                        Some(t) => topo_value_buffered(t, coords_buf, arcs_buf)?,
                        None => return Err(de::Error::missing_field("type")),
                    },
                };
                Ok(TopoGeometry { value, properties, id })
            }
        }
        d.deserialize_map(V)
    }
}

#[derive(Deserialize)]
struct Topology {
    #[serde(default)]
    transform: Option<Transform>,
    arcs: Vec<Vec<Pos>>,
    objects: HashMap<String, TopoGeometry>,
}

/// Delta-decode + dequantize a single arc.
fn decode_arc(arc: &[Pos], tr: &Option<Transform>) -> Vec<Pos> {
    match tr {
        None => arc.to_vec(),
        Some(tr) => {
            let (mut x, mut y) = (0.0, 0.0);
            arc.iter()
                .map(|p| {
                    x += p.x;
                    y += p.y;
                    Pos {
                        x: x * tr.scale[0] + tr.translate[0],
                        y: y * tr.scale[1] + tr.translate[1],
                    }
                })
                .collect()
        }
    }
}

fn make_pt(p: Pos, tr: &Option<Transform>) -> Pos {
    match tr {
        None => p,
        Some(tr) => Pos {
            x: p.x * tr.scale[0] + tr.translate[0],
            y: p.y * tr.scale[1] + tr.translate[1],
        },
    }
}

/// Stitch a ring from arc indices; negative index means the arc, reversed.
fn make_ring(decoded: &[Vec<Pos>], ixs: &[i32]) -> Vec<Pos> {
    let mut line = Vec::new();
    for &ix in ixs {
        if ix < 0 {
            if let Some(arc) = decoded.get((!ix) as usize) {
                line.extend(arc.iter().rev().copied());
            }
        } else if let Some(arc) = decoded.get(ix as usize) {
            line.extend_from_slice(arc);
        }
    }
    line
}

fn to_geometry(value: &TopoValue, decoded: &[Vec<Pos>], tr: &Option<Transform>) -> Geometry {
    match value {
        TopoValue::Point(p) => Geometry::Point(make_pt(*p, tr)),
        TopoValue::MultiPoint(ps) => {
            Geometry::MultiPoint(ps.iter().map(|p| make_pt(*p, tr)).collect())
        }
        TopoValue::LineString(ixs) => Geometry::LineString(make_ring(decoded, ixs)),
        TopoValue::MultiLineString(v) => {
            Geometry::MultiLineString(v.iter().map(|ixs| make_ring(decoded, ixs)).collect())
        }
        TopoValue::Polygon(v) => {
            Geometry::Polygon(v.iter().map(|ixs| make_ring(decoded, ixs)).collect())
        }
        TopoValue::MultiPolygon(vv) => Geometry::MultiPolygon(
            vv.iter()
                .map(|rings| rings.iter().map(|ixs| make_ring(decoded, ixs)).collect())
                .collect(),
        ),
        TopoValue::GeometryCollection(geoms) => {
            Geometry::GeometryCollection(geoms.iter().map(|g| to_geometry(&g.value, decoded, tr)).collect())
        }
    }
}

fn to_feature(g: TopoGeometry, decoded: &[Vec<Pos>], tr: &Option<Transform>) -> Feature {
    Feature {
        geometry: Some(to_geometry(&g.value, decoded, tr)),
        properties: g.properties,
        id: g.id,
    }
}

fn geometry_count(v: &TopoValue) -> usize {
    match v {
        TopoValue::GeometryCollection(g) => g.len(),
        _ => 1,
    }
}

/// Parse TopoJSON and convert its richest object into a GeoJSON FeatureCollection,
/// matching the previous crate-based behavior (pick the object with the most
/// geometries; each geometry in a collection becomes a feature).
pub fn try_topojson(content: &str) -> Result<GeoJson, String> {
    let mut topo: Topology = serde_json::from_str(content)
        .map_err(|e| format!("Not valid GeoJSON or TopoJSON: {e}"))?;

    let best_key = topo
        .objects
        .iter()
        .max_by_key(|(_, g)| geometry_count(&g.value))
        .map(|(k, _)| k.clone())
        .ok_or_else(|| "TopoJSON has no named objects".to_string())?;
    let best = topo.objects.remove(&best_key).unwrap();

    let decoded: Vec<Vec<Pos>> = topo.arcs.iter().map(|a| decode_arc(a, &topo.transform)).collect();

    let features = match best.value {
        TopoValue::GeometryCollection(geoms) => geoms
            .into_iter()
            .map(|g| to_feature(g, &decoded, &topo.transform))
            .collect(),
        _ => vec![to_feature(best, &decoded, &topo.transform)],
    };

    Ok(GeoJson::FeatureCollection(features))
}
