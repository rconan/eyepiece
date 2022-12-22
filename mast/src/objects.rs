use std::fmt::Display;

use num_complex::Complex;
use serde::{Deserialize, Serialize};
use skyangle::{Conversion, SkyAngle};

use crate::GaiaPhotometry;

/// MAST query data object
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MastObject {
    #[serde(rename = "GAIAmag")]
    pub(crate) gaimag: Option<f64>,
    #[serde(rename = "ID")]
    pub(crate) id: u64,
    pub(crate) dec: f64,
    pub(crate) ra: f64,
    pub(crate) gaiabp: Option<f64>,
    pub(crate) gaiarp: Option<f64>,
}
impl MastObject {
    /// Checks if all the GAIA properties are valid
    pub fn is_valid(&self) -> bool {
        !(self.gaimag.is_none() || self.gaiabp.is_none() || self.gaiarp.is_none())
    }
}
/// MAST query data object container
#[derive(Debug, Serialize, Deserialize)]
pub struct MastObjects {
    pub(crate) target: String,
    pub(crate) origin: (f64, f64),
    pub(crate) radius: f64,
    pub(crate) objects: Vec<MastObject>,
    pub(crate) photometry: GaiaPhotometry,
}
impl Display for MastObjects {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.target.to_uppercase())?;
        writeln!(f, " . (ra,dec) degree: {:?}", self.origin)?;
        writeln!(f, " . radius: {:}arcmin", self.radius.to_arcmin())?;
        writeln!(f, " . {:} stars", self.objects.len())
    }
}

impl MastObjects {
    /// Returns the number of [MastObject]
    pub fn len(&self) -> usize {
        self.objects.len()
    }
}

impl From<MastObjects> for eyepiece::Objects {
    /// Converts a [MastObjects] into a [eyepiece::Objects]
    fn from(mast_objects: MastObjects) -> Self {
        let (ra, dec) = mast_objects.origin;
        let zc = Complex::from_polar(dec.to_radians(), ra.to_radians());
        let gaia_band = mast_objects.photometry;
        mast_objects
            .objects
            .iter()
            .map(|object| {
                let zo = Complex::from_polar(object.dec.to_radians(), object.ra.to_radians());
                let dz = zo - zc;
                eyepiece::Star {
                    coordinates: (
                        SkyAngle::Radian(dz.re).into_arcsec(),
                        SkyAngle::Radian(dz.im).into_arcsec(),
                    ),
                    magnitude: gaia_band.magnitude(&object).unwrap(),
                }
            })
            .collect::<Vec<eyepiece::Star>>()
            .into()
    }
}
