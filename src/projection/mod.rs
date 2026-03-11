mod albers;
mod authagraph;
pub(crate) mod azimuthal;
mod azimuthal_equidistant;
mod bonne;
mod cassini;
mod elliptic;
mod equirectangular;
mod gnomonic;
mod hammer;
mod lambert;
mod lambert_azimuthal;
mod mercator;
mod natural_earth;
mod orthographic;
mod peirce;
mod polyconic;
mod robinson;
mod wiechel;
mod winkel_tripel;

use crate::geometry::for_each_coord_mut;
use geojson::GeoJson;
use serde::Deserialize;

pub trait Projection {
    fn project(&self, lon: f64, lat: f64) -> (f64, f64);
    fn antimeridian_gap(&self) -> f64 {
        f64::INFINITY
    }
}

#[inline]
fn prepare_lon_lat(lon: f64, lat: f64, central_meridian: f64) -> (f64, f64) {
    (
        normalize_lon(lon - central_meridian).to_radians(),
        lat.to_radians(),
    )
}

fn normalize_lon(mut d: f64) -> f64 {
    d = d % 360.0;
    if d > 180.0 {
        d -= 360.0;
    } else if d < -180.0 {
        d += 360.0;
    }
    d
}

fn default_sp1() -> f64 {
    33.0
}
fn default_sp2() -> f64 {
    45.0
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ProjectionConfig {
    Equirectangular {
        #[serde(default)]
        central_meridian: f64,
    },
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
    LambertAzimuthalEqualArea {
        #[serde(default)]
        center_lat: f64,
        #[serde(default)]
        center_lon: f64,
    },
    Gnomonic {
        #[serde(default)]
        center_lat: f64,
        #[serde(default)]
        center_lon: f64,
    },
    Wiechel {
        #[serde(default)]
        center_lat: f64,
        #[serde(default)]
        center_lon: f64,
    },
    PeirceQuincuncial {
        #[serde(default)]
        center_lon: f64,
    },
    Cassini {
        #[serde(default)]
        central_meridian: f64,
    },
    Bonne {
        #[serde(default)]
        central_meridian: f64,
        #[serde(default = "default_sp2")]
        standard_parallel: f64,
    },
    Polyconic {
        #[serde(default)]
        central_meridian: f64,
    },
    AzimuthalEquidistant {
        #[serde(default)]
        center_lat: f64,
        #[serde(default)]
        center_lon: f64,
    },
    Hammer {
        #[serde(default)]
        central_meridian: f64,
    },
    WinkelTripel {
        #[serde(default)]
        central_meridian: f64,
    },
    Authagraph,
}

pub fn from_config(config: Option<ProjectionConfig>) -> Box<dyn Projection> {
    match config {
        None => Box::new(equirectangular::Equirectangular {
            central_meridian: 0.0,
        }),
        Some(c) => match c {
            ProjectionConfig::Equirectangular { central_meridian } => {
                Box::new(equirectangular::Equirectangular { central_meridian })
            }
            ProjectionConfig::Mercator { central_meridian } => {
                Box::new(mercator::Mercator { central_meridian })
            }
            ProjectionConfig::LambertConformalConic {
                standard_parallel_1,
                standard_parallel_2,
                central_meridian,
                latitude_of_origin,
            } => Box::new(lambert::compile(
                standard_parallel_1,
                standard_parallel_2,
                central_meridian,
                latitude_of_origin,
            )),
            ProjectionConfig::AlbersEqualArea {
                standard_parallel_1,
                standard_parallel_2,
                central_meridian,
                latitude_of_origin,
            } => Box::new(albers::compile(
                standard_parallel_1,
                standard_parallel_2,
                central_meridian,
                latitude_of_origin,
            )),
            ProjectionConfig::Robinson { central_meridian } => {
                Box::new(robinson::Robinson { central_meridian })
            }
            ProjectionConfig::Orthographic {
                center_lat,
                center_lon,
            } => Box::new(orthographic::Orthographic(azimuthal::compile(
                center_lat, center_lon,
            ))),
            ProjectionConfig::NaturalEarth { central_meridian } => {
                Box::new(natural_earth::NaturalEarth { central_meridian })
            }
            ProjectionConfig::LambertAzimuthalEqualArea {
                center_lat,
                center_lon,
            } => Box::new(lambert_azimuthal::LambertAzimuthal(azimuthal::compile(
                center_lat, center_lon,
            ))),
            ProjectionConfig::Gnomonic {
                center_lat,
                center_lon,
            } => Box::new(gnomonic::Gnomonic(azimuthal::compile(
                center_lat, center_lon,
            ))),
            ProjectionConfig::Wiechel {
                center_lat,
                center_lon,
            } => Box::new(wiechel::Wiechel(azimuthal::compile(
                center_lat, center_lon,
            ))),
            ProjectionConfig::PeirceQuincuncial { center_lon } => {
                Box::new(peirce::compile(center_lon))
            }
            ProjectionConfig::Cassini { central_meridian } => {
                Box::new(cassini::Cassini { central_meridian })
            }
            ProjectionConfig::Bonne {
                central_meridian,
                standard_parallel,
            } => Box::new(bonne::Bonne {
                central_meridian,
                standard_parallel,
            }),
            ProjectionConfig::Polyconic { central_meridian } => {
                Box::new(polyconic::Polyconic { central_meridian })
            }
            ProjectionConfig::AzimuthalEquidistant {
                center_lat,
                center_lon,
            } => Box::new(azimuthal_equidistant::AzimuthalEquidistant(
                azimuthal::compile(center_lat, center_lon),
            )),
            ProjectionConfig::Hammer { central_meridian } => {
                Box::new(hammer::Hammer { central_meridian })
            }
            ProjectionConfig::WinkelTripel { central_meridian } => {
                Box::new(winkel_tripel::WinkelTripel { central_meridian })
            }
            ProjectionConfig::Authagraph => Box::new(authagraph::compile()),
        },
    }
}

/// Project all coordinates in a GeoJson structure in-place.
pub fn project_geojson(geojson: &mut GeoJson, proj: &dyn Projection) {
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
