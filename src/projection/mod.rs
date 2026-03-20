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

#[inline]
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

pub(crate) enum Proj {
    Equirectangular(equirectangular::Equirectangular),
    Mercator(mercator::Mercator),
    LambertConformalConic(lambert::Compiled),
    AlbersEqualArea(albers::Compiled),
    Robinson(robinson::Robinson),
    Orthographic(orthographic::Orthographic),
    NaturalEarth(natural_earth::NaturalEarth),
    LambertAzimuthal(lambert_azimuthal::LambertAzimuthal),
    Gnomonic(gnomonic::Gnomonic),
    Wiechel(wiechel::Wiechel),
    PeirceQuincuncial(peirce::Compiled),
    Cassini(cassini::Cassini),
    Bonne(bonne::Bonne),
    Polyconic(polyconic::Polyconic),
    AzimuthalEquidistant(azimuthal_equidistant::AzimuthalEquidistant),
    Hammer(hammer::Hammer),
    WinkelTripel(winkel_tripel::WinkelTripel),
    Authagraph(authagraph::Compiled),
}

macro_rules! dispatch {
    ($self:expr, $method:ident $(, $arg:expr)*) => {
        match $self {
            Proj::Equirectangular(p) => p.$method($($arg),*),
            Proj::Mercator(p) => p.$method($($arg),*),
            Proj::LambertConformalConic(p) => p.$method($($arg),*),
            Proj::AlbersEqualArea(p) => p.$method($($arg),*),
            Proj::Robinson(p) => p.$method($($arg),*),
            Proj::Orthographic(p) => p.$method($($arg),*),
            Proj::NaturalEarth(p) => p.$method($($arg),*),
            Proj::LambertAzimuthal(p) => p.$method($($arg),*),
            Proj::Gnomonic(p) => p.$method($($arg),*),
            Proj::Wiechel(p) => p.$method($($arg),*),
            Proj::PeirceQuincuncial(p) => p.$method($($arg),*),
            Proj::Cassini(p) => p.$method($($arg),*),
            Proj::Bonne(p) => p.$method($($arg),*),
            Proj::Polyconic(p) => p.$method($($arg),*),
            Proj::AzimuthalEquidistant(p) => p.$method($($arg),*),
            Proj::Hammer(p) => p.$method($($arg),*),
            Proj::WinkelTripel(p) => p.$method($($arg),*),
            Proj::Authagraph(p) => p.$method($($arg),*),
        }
    };
}

impl Proj {
    #[inline]
    pub fn project(&self, lon: f64, lat: f64) -> (f64, f64) {
        dispatch!(self, project, lon, lat)
    }

    #[inline]
    pub fn antimeridian_gap(&self) -> f64 {
        dispatch!(self, antimeridian_gap)
    }
}

pub fn from_config(config: Option<ProjectionConfig>) -> Proj {
    match config {
        None => Proj::Equirectangular(equirectangular::Equirectangular {
            central_meridian: 0.0,
        }),
        Some(c) => match c {
            ProjectionConfig::Equirectangular { central_meridian } => {
                Proj::Equirectangular(equirectangular::Equirectangular { central_meridian })
            }
            ProjectionConfig::Mercator { central_meridian } => {
                Proj::Mercator(mercator::Mercator { central_meridian })
            }
            ProjectionConfig::LambertConformalConic {
                standard_parallel_1,
                standard_parallel_2,
                central_meridian,
                latitude_of_origin,
            } => Proj::LambertConformalConic(lambert::compile(
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
            } => Proj::AlbersEqualArea(albers::compile(
                standard_parallel_1,
                standard_parallel_2,
                central_meridian,
                latitude_of_origin,
            )),
            ProjectionConfig::Robinson { central_meridian } => {
                Proj::Robinson(robinson::Robinson { central_meridian })
            }
            ProjectionConfig::Orthographic {
                center_lat,
                center_lon,
            } => Proj::Orthographic(orthographic::Orthographic(azimuthal::compile(
                center_lat, center_lon,
            ))),
            ProjectionConfig::NaturalEarth { central_meridian } => {
                Proj::NaturalEarth(natural_earth::NaturalEarth { central_meridian })
            }
            ProjectionConfig::LambertAzimuthalEqualArea {
                center_lat,
                center_lon,
            } => Proj::LambertAzimuthal(lambert_azimuthal::LambertAzimuthal(azimuthal::compile(
                center_lat, center_lon,
            ))),
            ProjectionConfig::Gnomonic {
                center_lat,
                center_lon,
            } => Proj::Gnomonic(gnomonic::Gnomonic(azimuthal::compile(
                center_lat, center_lon,
            ))),
            ProjectionConfig::Wiechel {
                center_lat,
                center_lon,
            } => Proj::Wiechel(wiechel::Wiechel(azimuthal::compile(
                center_lat, center_lon,
            ))),
            ProjectionConfig::PeirceQuincuncial { center_lon } => {
                Proj::PeirceQuincuncial(peirce::compile(center_lon))
            }
            ProjectionConfig::Cassini { central_meridian } => {
                Proj::Cassini(cassini::Cassini { central_meridian })
            }
            ProjectionConfig::Bonne {
                central_meridian,
                standard_parallel,
            } => Proj::Bonne(bonne::Bonne::new(central_meridian, standard_parallel)),
            ProjectionConfig::Polyconic { central_meridian } => {
                Proj::Polyconic(polyconic::Polyconic { central_meridian })
            }
            ProjectionConfig::AzimuthalEquidistant {
                center_lat,
                center_lon,
            } => Proj::AzimuthalEquidistant(azimuthal_equidistant::AzimuthalEquidistant(
                azimuthal::compile(center_lat, center_lon),
            )),
            ProjectionConfig::Hammer { central_meridian } => {
                Proj::Hammer(hammer::Hammer { central_meridian })
            }
            ProjectionConfig::WinkelTripel { central_meridian } => {
                Proj::WinkelTripel(winkel_tripel::WinkelTripel::new(central_meridian))
            }
            ProjectionConfig::Authagraph => Proj::Authagraph(authagraph::compile()),
        },
    }
}

