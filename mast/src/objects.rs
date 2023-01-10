use std::fmt::Display;

use serde::{Deserialize, Serialize};
use skyangle::{Conversion, SkyAngle};

use crate::GaiaPhotometry;

/// MAST query data object
#[derive(Default, Debug, Serialize, Deserialize, Clone, PartialEq, PartialOrd)]
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
impl From<(f64, f64)> for MastObject {
    fn from((ra, dec): (f64, f64)) -> Self {
        Self {
            id: 0,
            dec,
            ra,
            ..Default::default()
        }
    }
}
impl MastObject {
    /// Checks if all the GAIA properties are valid
    pub fn is_valid(&self) -> bool {
        !(self.gaimag.is_none() || self.gaiabp.is_none() || self.gaiarp.is_none())
    }
    /// Returns on-sky angular separation between 2 objects
    pub fn separation(&self, other: &MastObject) -> SkyAngle<f64> {
        let lon1 = self.ra.to_radians();
        let lat1 = self.dec.to_radians();
        let lon2 = other.ra.to_radians();
        let lat2 = other.dec.to_radians();

        let sdlon = (lon2 - lon1).sin();
        let cdlon = (lon2 - lon1).cos();
        let slat1 = (lat1).sin();
        let slat2 = (lat2).sin();
        let clat1 = (lat1).cos();
        let clat2 = (lat2).cos();

        let num1 = clat2 * sdlon;
        let num2 = clat1 * slat2 - slat1 * clat2 * cdlon;
        let denominator = slat1 * slat2 + clat1 * clat2 * cdlon;

        SkyAngle::Radian(num1.hypot(num2).atan2(denominator))
    }
    /// Returns the on-sky position angle between 2 objects
    pub fn position_angle(&self, other: &MastObject) -> SkyAngle<f64> {
        let lon1 = self.ra.to_radians();
        let lat1 = self.dec.to_radians();
        let lon2 = other.ra.to_radians();
        let lat2 = other.dec.to_radians();

        let deltalon = lon2 - lon1;
        let colat = lat2.cos();

        let x = lat2.sin() * lat1.cos() - colat * lat1.sin() * deltalon.cos();
        let y = deltalon.sin() * colat;

        SkyAngle::Radian(y.atan2(x))
    }
    /// Returns the cartesian offset angles between 2 objects
    pub fn offsets(&self, other: &MastObject) -> (SkyAngle<f64>, SkyAngle<f64>) {
        let sep = self.separation(other).to_radians();
        let pos_angle = self.position_angle(other).to_radians();
        let (s, c) = pos_angle.sin_cos();
        (SkyAngle::Radian(sep * c), SkyAngle::Radian(sep * s))
    }
}
/// MAST query data object container
#[derive(Debug, Serialize, Deserialize)]
pub struct MastObjects {
    pub(crate) target: String,
    // field origin ra & dec [rad]
    pub(crate) origin: (f64, f64),
    pub(crate) radius: f64,
    pub(crate) objects: Vec<MastObject>,
    pub(crate) photometry: GaiaPhotometry,
}
impl Display for MastObjects {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.target.to_uppercase())?;
        writeln!(f, " . (ra,dec) degree: {:.3?}", self.origin)?;
        writeln!(f, " . radius: {:.3}arcmin", self.radius.to_arcmin())?;
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
        let origin = MastObject::from(mast_objects.origin);
        let gaia_band = mast_objects.photometry;
        mast_objects
            .objects
            .iter()
            .map(|object| eyepiece::Star {
                coordinates: object.offsets(&origin),
                magnitude: gaia_band.magnitude(&object).unwrap(),
            })
            .collect::<Vec<eyepiece::Star>>()
            .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn separation() {
        let c = MastObject::from((12.11, 85.26));
        let o = MastObject::from((12., 85.));
        let sep = o.separation(&c);
        assert!(sep.to_radians() - 0.00454078 < 1e-9);
    }

    #[test]
    fn position_angle() {
        let c = MastObject::from((12.11, 85.26));
        let o = MastObject::from((12., 85.));
        let pos_angle = o.position_angle(&c);
        assert!(pos_angle.to_radians() - 0.0349453518 < 1e-9);
    }

    #[test]
    fn offsets() {
        let c = MastObject::from((12.11, 85.26));
        let o = MastObject::from((12., 85.));
        let offsets = o.offsets(&c);
        assert!(
            offsets.0.to_radians() - 0.0045380077 < 1e-9
                && offsets.1.to_radians() - 0.0001586468 < 1e-9
        );
    }
}
