mod albers;
mod equirectangular;
mod lambert;
mod mercator;
mod natural_earth;
mod orthographic;
mod robinson;

use crate::geometry::for_each_coord_mut;
use geojson::GeoJson;
use serde::Deserialize;

fn default_sp1() -> f64 {
    33.0
}
fn default_sp2() -> f64 {
    45.0
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ProjectionConfig {
    Equirectangular,
    Mercator {
        #[serde(default)]
        central_meridian: f64,
    },
    LambertConformalConic {
        #[serde(default = "default_sp1")]
        standard_parallel_1: f64,
        #[serde(default = "default_sp2")]
        standard_parallel_2: f64,
        #[serde(default)]
        central_meridian: f64,
        #[serde(default)]
        latitude_of_origin: f64,
    },
    AlbersEqualArea {
        #[serde(default = "default_sp1")]
        standard_parallel_1: f64,
        #[serde(default = "default_sp2")]
        standard_parallel_2: f64,
        #[serde(default)]
        central_meridian: f64,
        #[serde(default)]
        latitude_of_origin: f64,
    },
    Robinson {
        #[serde(default)]
        central_meridian: f64,
    },
    Orthographic {
        #[serde(default)]
        center_lat: f64,
        #[serde(default)]
        center_lon: f64,
    },
    NaturalEarth {
        #[serde(default)]
        central_meridian: f64,
    },
}

pub enum CompiledProjection {
    Equirectangular,
    Mercator { central_meridian: f64 },
    LambertConformalConic(lambert::Compiled),
    AlbersEqualArea(albers::Compiled),
    Robinson { central_meridian: f64 },
    Orthographic(orthographic::Compiled),
    NaturalEarth { central_meridian: f64 },
}

impl CompiledProjection {
    pub fn from_config(config: Option<ProjectionConfig>) -> Self {
        match config {
            None | Some(ProjectionConfig::Equirectangular) => Self::Equirectangular,
            Some(ProjectionConfig::Mercator { central_meridian }) => {
                Self::Mercator { central_meridian }
            }
            Some(ProjectionConfig::LambertConformalConic {
                standard_parallel_1,
                standard_parallel_2,
                central_meridian,
                latitude_of_origin,
            }) => Self::LambertConformalConic(lambert::compile(
                standard_parallel_1,
                standard_parallel_2,
                central_meridian,
                latitude_of_origin,
            )),
            Some(ProjectionConfig::AlbersEqualArea {
                standard_parallel_1,
                standard_parallel_2,
                central_meridian,
                latitude_of_origin,
            }) => Self::AlbersEqualArea(albers::compile(
                standard_parallel_1,
                standard_parallel_2,
                central_meridian,
                latitude_of_origin,
            )),
            Some(ProjectionConfig::Robinson { central_meridian }) => {
                Self::Robinson { central_meridian }
            }
            Some(ProjectionConfig::Orthographic {
                center_lat,
                center_lon,
            }) => Self::Orthographic(orthographic::compile(center_lat, center_lon)),
            Some(ProjectionConfig::NaturalEarth { central_meridian }) => {
                Self::NaturalEarth { central_meridian }
            }
        }
    }

    /// Maximum x-gap threshold for antimeridian crossing detection.
    pub fn antimeridian_gap(&self) -> f64 {
        match self {
            Self::Equirectangular => equirectangular::antimeridian_gap(),
            Self::Mercator { .. } => mercator::antimeridian_gap(),
            Self::Robinson { .. } => robinson::antimeridian_gap(),
            Self::NaturalEarth { .. } => natural_earth::antimeridian_gap(),
            _ => f64::INFINITY,
        }
    }

    /// Project (lon, lat) in degrees to (x, y) with y-flipped (north = negative y) for SVG.
    pub fn project(&self, lon: f64, lat: f64) -> (f64, f64) {
        match self {
            Self::Equirectangular => equirectangular::project(lon, lat),
            Self::Mercator { central_meridian } => mercator::project(lon, lat, *central_meridian),
            Self::LambertConformalConic(c) => lambert::project(lon, lat, c),
            Self::AlbersEqualArea(c) => albers::project(lon, lat, c),
            Self::Robinson { central_meridian } => robinson::project(lon, lat, *central_meridian),
            Self::Orthographic(c) => orthographic::project(lon, lat, c),
            Self::NaturalEarth { central_meridian } => {
                natural_earth::project(lon, lat, *central_meridian)
            }
        }
    }
}

/// Normalize a longitude difference to the range [-180, 180].
fn normalize_lon(mut d: f64) -> f64 {
    d = d % 360.0;
    if d > 180.0 {
        d -= 360.0;
    } else if d < -180.0 {
        d += 360.0;
    }
    d
}

/// Project all coordinates in a GeoJson structure in-place.
pub fn project_geojson(geojson: &mut GeoJson, proj: &CompiledProjection) {
    let mut project = |coord: &mut Vec<f64>| {
        if coord.len() >= 2 {
            let (x, y) = proj.project(coord[0], coord[1]);
            coord[0] = x;
            coord[1] = y;
        }
    };

    match geojson {
        GeoJson::FeatureCollection(fc) => {
            fc.bbox = None;
            for feat in &mut fc.features {
                feat.bbox = None;
                if let Some(ref mut geom) = feat.geometry {
                    geom.bbox = None;
                    for_each_coord_mut(&mut geom.value, &mut project);
                }
            }
        }
        GeoJson::Feature(feat) => {
            feat.bbox = None;
            if let Some(ref mut geom) = feat.geometry {
                geom.bbox = None;
                for_each_coord_mut(&mut geom.value, &mut project);
            }
        }
        GeoJson::Geometry(geom) => {
            geom.bbox = None;
            for_each_coord_mut(&mut geom.value, &mut project);
        }
    }
}
