use geojson::GeoJson;

pub fn try_topojson(content: &str) -> Result<GeoJson, String> {
    let topo: topojson::Topology =
        serde_json::from_str(content).map_err(|e| format!("Not valid GeoJSON or TopoJSON: {e}"))?;

    let best = topo
        .objects
        .iter()
        .max_by_key(|ng| match &ng.geometry.value {
            topojson::Value::GeometryCollection(geoms) => geoms.len(),
            _ => 1,
        })
        .ok_or_else(|| "TopoJSON has no named objects".to_string())?;

    let fc = topojson::to_geojson(&topo, &best.name).map_err(|e| e.to_string())?;
    Ok(GeoJson::FeatureCollection(geojson::FeatureCollection {
        bbox: None,
        features: fc.features,
        foreign_members: None,
    }))
}
