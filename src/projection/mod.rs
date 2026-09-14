mod aitoff;
mod albers;
mod authagraph;
pub(crate) mod azimuthal;
mod azimuthal_equidistant;
mod bonne;
mod cassini;
mod eckert4;
mod eckert6;
mod elliptic;
mod equirectangular;
mod gall_peters;
mod gall_stereographic;
mod gnomonic;
mod kavrayskiy7;
mod hammer;
mod lambert;
mod lambert_azimuthal;
mod mercator;
mod miller;
mod mollweide;
mod natural_earth;
mod orthographic;
mod peirce;
mod polyconic;
mod robinson;
mod sinusoidal;
mod van_der_grinten;
mod wagner6;
mod wiechel;
mod winkel_tripel;

use serde::Deserialize;

pub trait Projection {
    fn project(&self, lon: f64, lat: f64) -> (f64, f64);
    fn antimeridian_gap(&self) -> f64 {
        f64::INFINITY
    }
    /// Unit center vector for azimuthal projections that show only the near
    /// hemisphere (the limb clips at 90°). `None` means no hemisphere clipping.
    fn clip_center(&self) -> Option<[f64; 3]> {
        None
    }
    /// Central meridian for projections that support antimeridian clipping
    /// (cylindrical-family, where the antimeridian is a clean seam). `None`
    /// means antimeridian clipping is not offered for this projection.
    fn antimeridian_center(&self) -> Option<f64> {
        None
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
    Sinusoidal {
        #[serde(default)]
        central_meridian: f64,
    },
    Miller {
        #[serde(default)]
        central_meridian: f64,
    },
    Mollweide {
        #[serde(default)]
        central_meridian: f64,
    },
    Aitoff {
        #[serde(default)]
        central_meridian: f64,
    },
    Eckert4 {
        #[serde(default)]
        central_meridian: f64,
    },
    GallStereographic {
        #[serde(default)]
        central_meridian: f64,
    },
    GallPeters {
        #[serde(default)]
        central_meridian: f64,
    },
    Kavrayskiy7 {
        #[serde(default)]
        central_meridian: f64,
    },
    Wagner6 {
        #[serde(default)]
        central_meridian: f64,
    },
    Eckert6 {
        #[serde(default)]
        central_meridian: f64,
    },
    VanDerGrinten {
        #[serde(default)]
        central_meridian: f64,
    },
    Authagraph,
}

enum ProjKind {
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
    Sinusoidal(sinusoidal::Sinusoidal),
    Miller(miller::Miller),
    Mollweide(mollweide::Mollweide),
    Aitoff(aitoff::Aitoff),
    Eckert4(eckert4::Eckert4),
    GallStereographic(gall_stereographic::GallStereographic),
    GallPeters(gall_peters::GallPeters),
    Kavrayskiy7(kavrayskiy7::Kavrayskiy7),
    Wagner6(wagner6::Wagner6),
    Eckert6(eckert6::Eckert6),
    VanDerGrinten(van_der_grinten::VanDerGrinten),
    Authagraph(authagraph::Compiled),
}

macro_rules! dispatch {
    ($self:expr, $method:ident $(, $arg:expr)*) => {
        match $self {
            ProjKind::Equirectangular(p) => p.$method($($arg),*),
            ProjKind::Mercator(p) => p.$method($($arg),*),
            ProjKind::LambertConformalConic(p) => p.$method($($arg),*),
            ProjKind::AlbersEqualArea(p) => p.$method($($arg),*),
            ProjKind::Robinson(p) => p.$method($($arg),*),
            ProjKind::Orthographic(p) => p.$method($($arg),*),
            ProjKind::NaturalEarth(p) => p.$method($($arg),*),
            ProjKind::LambertAzimuthal(p) => p.$method($($arg),*),
            ProjKind::Gnomonic(p) => p.$method($($arg),*),
            ProjKind::Wiechel(p) => p.$method($($arg),*),
            ProjKind::PeirceQuincuncial(p) => p.$method($($arg),*),
            ProjKind::Cassini(p) => p.$method($($arg),*),
            ProjKind::Bonne(p) => p.$method($($arg),*),
            ProjKind::Polyconic(p) => p.$method($($arg),*),
            ProjKind::AzimuthalEquidistant(p) => p.$method($($arg),*),
            ProjKind::Hammer(p) => p.$method($($arg),*),
            ProjKind::WinkelTripel(p) => p.$method($($arg),*),
            ProjKind::Sinusoidal(p) => p.$method($($arg),*),
            ProjKind::Miller(p) => p.$method($($arg),*),
            ProjKind::Mollweide(p) => p.$method($($arg),*),
            ProjKind::Aitoff(p) => p.$method($($arg),*),
            ProjKind::Eckert4(p) => p.$method($($arg),*),
            ProjKind::GallStereographic(p) => p.$method($($arg),*),
            ProjKind::GallPeters(p) => p.$method($($arg),*),
            ProjKind::Kavrayskiy7(p) => p.$method($($arg),*),
            ProjKind::Wagner6(p) => p.$method($($arg),*),
            ProjKind::Eckert6(p) => p.$method($($arg),*),
            ProjKind::VanDerGrinten(p) => p.$method($($arg),*),
            ProjKind::Authagraph(p) => p.$method($($arg),*),
        }
    };
}

impl ProjKind {
    #[inline]
    fn project(&self, lon: f64, lat: f64) -> (f64, f64) {
        dispatch!(self, project, lon, lat)
    }

    #[inline]
    fn antimeridian_gap(&self) -> f64 {
        dispatch!(self, antimeridian_gap)
    }

    #[inline]
    fn clip_center(&self) -> Option<[f64; 3]> {
        dispatch!(self, clip_center)
    }

    #[inline]
    fn antimeridian_center(&self) -> Option<f64> {
        dispatch!(self, antimeridian_center)
    }
}

/// A spherical pre-rotation `[lambda, phi, gamma]` (degrees), applied to input
/// coordinates before projecting — a faithful port of d3.geoRotation. Lets any
/// projection be recentered/tilted/rolled to an oblique aspect.
struct Rotate {
    delta_lambda: f64,
    cos_dp: f64,
    sin_dp: f64,
    cos_dg: f64,
    sin_dg: f64,
    phi_gamma: bool,
}

impl Rotate {
    fn new(r: [f64; 3]) -> Rotate {
        let dp = r[1].to_radians();
        let dg = r[2].to_radians();
        Rotate {
            delta_lambda: r[0].to_radians(),
            cos_dp: dp.cos(),
            sin_dp: dp.sin(),
            cos_dg: dg.cos(),
            sin_dg: dg.sin(),
            phi_gamma: r[1] != 0.0 || r[2] != 0.0,
        }
    }

    /// Rotate a lon/lat (degrees) → lon/lat (degrees).
    #[inline]
    fn apply(&self, lon: f64, lat: f64) -> (f64, f64) {
        use std::f64::consts::{PI, TAU};
        let mut lambda = lon.to_radians() + self.delta_lambda;
        if lambda > PI {
            lambda -= TAU;
        } else if lambda < -PI {
            lambda += TAU;
        }
        let phi = lat.to_radians();
        if !self.phi_gamma {
            return (lambda.to_degrees(), phi.to_degrees());
        }
        let cos_phi = phi.cos();
        let x = lambda.cos() * cos_phi;
        let y = lambda.sin() * cos_phi;
        let z = phi.sin();
        let k = z * self.cos_dp + x * self.sin_dp;
        let out_lambda = (y * self.cos_dg - k * self.sin_dg).atan2(x * self.cos_dp - z * self.sin_dp);
        let out_phi = (k * self.cos_dg + y * self.sin_dg).clamp(-1.0, 1.0).asin();
        (out_lambda.to_degrees(), out_phi.to_degrees())
    }
}

/// A projection plus an optional spherical pre-rotation.
pub(crate) struct Proj {
    kind: ProjKind,
    rot: Option<Rotate>,
}

impl Proj {
    #[inline]
    pub fn project(&self, lon: f64, lat: f64) -> (f64, f64) {
        match &self.rot {
            Some(r) => {
                let (l, p) = r.apply(lon, lat);
                self.kind.project(l, p)
            }
            None => self.kind.project(lon, lat),
        }
    }

    #[inline]
    pub fn antimeridian_gap(&self) -> f64 {
        self.kind.antimeridian_gap()
    }

    #[inline]
    pub fn clip_center(&self) -> Option<[f64; 3]> {
        self.kind.clip_center()
    }

    #[inline]
    pub fn antimeridian_center(&self) -> Option<f64> {
        // A pre-rotation moves the ±180° seam, so the (unrotated) antimeridian
        // clip no longer applies; disable it when rotated.
        if self.rot.is_some() {
            None
        } else {
            self.kind.antimeridian_center()
        }
    }
}

/// Build a projection from config, with an optional `rotate: [lambda, phi, gamma]`
/// (degrees). Rotation is applied to azimuthal projections' input too, but those
/// already self-center via `center_lat`/`center_lon` and use a limb clip that
/// assumes unrotated input — so rotation is only attached when the projection has
/// no limb clip (i.e. non-azimuthal families).
pub fn from_config(config: Option<ProjectionConfig>, rotate: Option<[f64; 3]>) -> Proj {
    let kind = build_kind(config);
    let rot = rotate
        .filter(|r| *r != [0.0, 0.0, 0.0] && kind.clip_center().is_none())
        .map(Rotate::new);
    Proj { kind, rot }
}

fn build_kind(config: Option<ProjectionConfig>) -> ProjKind {
    match config {
        None => ProjKind::Equirectangular(equirectangular::Equirectangular {
            central_meridian: 0.0,
        }),
        Some(c) => match c {
            ProjectionConfig::Equirectangular { central_meridian } => {
                ProjKind::Equirectangular(equirectangular::Equirectangular { central_meridian })
            }
            ProjectionConfig::Mercator { central_meridian } => {
                ProjKind::Mercator(mercator::Mercator { central_meridian })
            }
            ProjectionConfig::LambertConformalConic {
                standard_parallel_1,
                standard_parallel_2,
                central_meridian,
                latitude_of_origin,
            } => ProjKind::LambertConformalConic(lambert::compile(
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
            } => ProjKind::AlbersEqualArea(albers::compile(
                standard_parallel_1,
                standard_parallel_2,
                central_meridian,
                latitude_of_origin,
            )),
            ProjectionConfig::Robinson { central_meridian } => {
                ProjKind::Robinson(robinson::Robinson { central_meridian })
            }
            ProjectionConfig::Orthographic {
                center_lat,
                center_lon,
            } => ProjKind::Orthographic(orthographic::Orthographic(azimuthal::compile(
                center_lat, center_lon,
            ))),
            ProjectionConfig::NaturalEarth { central_meridian } => {
                ProjKind::NaturalEarth(natural_earth::NaturalEarth { central_meridian })
            }
            ProjectionConfig::LambertAzimuthalEqualArea {
                center_lat,
                center_lon,
            } => ProjKind::LambertAzimuthal(lambert_azimuthal::LambertAzimuthal(azimuthal::compile(
                center_lat, center_lon,
            ))),
            ProjectionConfig::Gnomonic {
                center_lat,
                center_lon,
            } => ProjKind::Gnomonic(gnomonic::Gnomonic(azimuthal::compile(
                center_lat, center_lon,
            ))),
            ProjectionConfig::Wiechel {
                center_lat,
                center_lon,
            } => ProjKind::Wiechel(wiechel::Wiechel(azimuthal::compile(
                center_lat, center_lon,
            ))),
            ProjectionConfig::PeirceQuincuncial { center_lon } => {
                ProjKind::PeirceQuincuncial(peirce::compile(center_lon))
            }
            ProjectionConfig::Cassini { central_meridian } => {
                ProjKind::Cassini(cassini::Cassini { central_meridian })
            }
            ProjectionConfig::Bonne {
                central_meridian,
                standard_parallel,
            } => ProjKind::Bonne(bonne::Bonne::new(central_meridian, standard_parallel)),
            ProjectionConfig::Polyconic { central_meridian } => {
                ProjKind::Polyconic(polyconic::Polyconic { central_meridian })
            }
            ProjectionConfig::AzimuthalEquidistant {
                center_lat,
                center_lon,
            } => ProjKind::AzimuthalEquidistant(azimuthal_equidistant::AzimuthalEquidistant(
                azimuthal::compile(center_lat, center_lon),
            )),
            ProjectionConfig::Hammer { central_meridian } => {
                ProjKind::Hammer(hammer::Hammer { central_meridian })
            }
            ProjectionConfig::WinkelTripel { central_meridian } => {
                ProjKind::WinkelTripel(winkel_tripel::WinkelTripel::new(central_meridian))
            }
            ProjectionConfig::Sinusoidal { central_meridian } => {
                ProjKind::Sinusoidal(sinusoidal::Sinusoidal { central_meridian })
            }
            ProjectionConfig::Miller { central_meridian } => {
                ProjKind::Miller(miller::Miller { central_meridian })
            }
            ProjectionConfig::Mollweide { central_meridian } => {
                ProjKind::Mollweide(mollweide::Mollweide { central_meridian })
            }
            ProjectionConfig::Aitoff { central_meridian } => {
                ProjKind::Aitoff(aitoff::Aitoff { central_meridian })
            }
            ProjectionConfig::Eckert4 { central_meridian } => {
                ProjKind::Eckert4(eckert4::Eckert4 { central_meridian })
            }
            ProjectionConfig::GallStereographic { central_meridian } => {
                ProjKind::GallStereographic(gall_stereographic::GallStereographic { central_meridian })
            }
            ProjectionConfig::GallPeters { central_meridian } => {
                ProjKind::GallPeters(gall_peters::GallPeters { central_meridian })
            }
            ProjectionConfig::Kavrayskiy7 { central_meridian } => {
                ProjKind::Kavrayskiy7(kavrayskiy7::Kavrayskiy7 { central_meridian })
            }
            ProjectionConfig::Wagner6 { central_meridian } => {
                ProjKind::Wagner6(wagner6::Wagner6 { central_meridian })
            }
            ProjectionConfig::Eckert6 { central_meridian } => {
                ProjKind::Eckert6(eckert6::Eckert6 { central_meridian })
            }
            ProjectionConfig::VanDerGrinten { central_meridian } => {
                ProjKind::VanDerGrinten(van_der_grinten::VanDerGrinten { central_meridian })
            }
            ProjectionConfig::Authagraph => ProjKind::Authagraph(authagraph::compile()),
        },
    }
}

