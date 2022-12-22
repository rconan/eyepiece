//! # MAST-EYEPIECE
//!
//! Use [MAST](https://archive.stsci.edu/) to query a circular region on the sky
//! around a given object taken from the [TIC](https://tess.mit.edu/science/tess-input-catalogue/) catalog
//! keeping object only within the [GAIA](https://www.gaia-eso.eu/) survey
//!
//! ## Example
//! ```
//! # tokio_test::block_on(async{
//! use mast_eyepiece::Mast;
//! use skyangle::SkyAngle;
//!
//! let mast = Mast::new();
//! let mut objects = mast
//!     .query_region("NGC 6405", SkyAngle::Arcminute(1.))
//!     .await
//!     .unwrap();
//! # })
//! ```

use num_complex::Complex;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use skyangle::{Conversion, SkyAngle};
use std::{collections::HashMap, fmt::Display};

const MAST_URL: &str = "https://mast.stsci.edu/api/v0/invoke";

pub struct FitCoefficients(Vec<f64>);
pub enum GaiaMagnitude {
    V,
    R,
    I,
    J,
    H,
    K,
}
impl GaiaMagnitude {
    /// GAIA magnitude polynomial fit coefficients
    ///
    /// From [Table 5.7](https://gea.esac.esa.int/archive/documentation/GEDR3/Data_processing/chap_cu5pho/cu5pho_sec_photSystem/cu5pho_ssec_photRelations.html) of the EDR3 data guide
    pub fn polynomial(&self) -> Vec<f64> {
        match self {
            GaiaMagnitude::V => vec![-0.02704, 0.01424, -0.2156, 0.01426],
            GaiaMagnitude::R => vec![-0.02275, 0.3961, -0.1243, -0.01396, 0.003775],
            GaiaMagnitude::I => vec![0.01753, 0.76, -0.0991],
            GaiaMagnitude::J => vec![0.01798, 1.389, -0.09338],
            GaiaMagnitude::H => vec![-0.1048, 2.011, -0.1758],
            GaiaMagnitude::K => vec![-0.0981, 2.089, -0.1579],
        }
    }
    pub fn magnitude(&self, object: &MastObject) -> Option<f64> {
        let MastObject {
            gaimag,
            gaiabp,
            gaiarp,
            ..
        } = object;
        gaimag
            .as_ref()
            .zip(gaiabp.as_ref().zip(gaiarp.as_ref()))
            .map(|(g, (g_bp, g_rb))| {
                let bp_rb = g_bp - g_rb;
                let p = self.polynomial();
                let n = p.len();
                let dg = p[0]
                    + p.into_iter().rev().take(n - 1).fold(0f64, |mut a, p| {
                        a = (a + p) * bp_rb;
                        a
                    });
                g - dg
            })
    }
}
impl From<&str> for GaiaMagnitude {
    /// Astronomical photometric bands
    ///
    /// Converts the bands V, R, I, J, H and K into star [Photometry]
    fn from(band: &str) -> Self {
        match band {
            "V" => GaiaMagnitude::V,
            "R" => GaiaMagnitude::R,
            "I" => GaiaMagnitude::I,
            "J" => GaiaMagnitude::J,
            "H" => GaiaMagnitude::H,
            "K" => GaiaMagnitude::K,
            _ => panic!("expected the photometric band: V, R, I, J, H or K, found {band}"),
        }
    }
}
#[derive(Debug, thiserror::Error)]
pub enum MastEyepieceError {
    #[error("failed to serialize data into JSON")]
    JSONSerialize(#[from] serde_json::Error),
    #[error("MAST query hast failed")]
    QueryRegion(#[from] reqwest::Error),
}

pub type Result<T> = std::result::Result<T, MastEyepieceError>;

/// Mast request
#[derive(Debug, Serialize, Deserialize)]
pub struct Request {
    service: String,
    format: String,
    params: Value,
}
impl Request {
    /// Creates a new request from a [service](https://mast.stsci.edu/api/v0/_services.html) and parameters
    pub fn new<S: Into<String>, T: Into<Value>>(service: S, params: T) -> Self {
        Self {
            service: service.into(),
            format: "json".to_string(),
            params: params.into(),
        }
    }
}
impl TryFrom<Request> for HashMap<String, String> {
    type Error = serde_json::Error;
    /// Converts a request into a [HashMap]
    ///
    /// The [HashMap] can be sent to MAST
    fn try_from(value: Request) -> std::result::Result<Self, Self::Error> {
        let mut map = HashMap::new();
        map.insert("request".to_string(), serde_json::to_string(&value)?);
        Ok(map)
    }
}

/// MAST query data object
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MastObject {
    #[serde(rename = "GAIAmag")]
    gaimag: Option<f64>,
    #[serde(rename = "ID")]
    id: u64,
    dec: f64,
    ra: f64,
    gaiabp: Option<f64>,
    gaiarp: Option<f64>,
}
impl MastObject {
    /// Checks if all the GAIA properties are valid
    pub fn is_valid(&self) -> bool {
        !(self.gaimag.is_none() || self.gaiabp.is_none() || self.gaiarp.is_none())
    }
}
/// MAST query data
#[derive(Debug, Serialize, Deserialize)]
pub struct MastObjects {
    target: String,
    origin: (f64, f64),
    radius: f64,
    objects: Vec<MastObject>,
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
    pub fn len(&self) -> usize {
        self.objects.len()
    }
    /// Returns the number of valid objects
    ///
    /// Valid objects are objects with valid GAIA properties
    pub fn n_valid(&self) -> usize {
        self.objects
            .iter()
            .filter(|object| object.is_valid())
            .map(|_| 1usize)
            .sum()
    }
    /// Keeps only valid objects
    pub fn only_valid(&mut self) -> &mut Self {
        self.objects = self
            .objects
            .iter()
            .filter(|object| object.is_valid())
            .cloned()
            .collect();
        self
    }
    pub fn into_eyepiece_objects<B: Into<GaiaMagnitude>>(self, band: B) -> eyepiece::Objects {
        let (ra, dec) = self.origin;
        let zc = Complex::from_polar(dec.to_radians(), ra.to_radians());
        let gaia_band: GaiaMagnitude = band.into();
        self.objects
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

/// MAST interface
pub struct Mast {
    client: reqwest::Client,
    #[allow(dead_code)]
    format: String,
    url: String,
}
impl Default for Mast {
    fn default() -> Self {
        Self {
            client: reqwest::Client::new(),
            format: "json".to_string(),
            url: MAST_URL.to_string(),
        }
    }
}
impl Mast {
    /// Creates a new MAST interface
    pub fn new() -> Self {
        Default::default()
    }
    /// Queries a circular region aroud a given object
    pub async fn query_region<S: Into<String> + Serialize>(
        &self,
        object_id: S,
        radius: SkyAngle<f64>,
    ) -> Result<MastObjects> {
        let form: HashMap<_, _> = Request::new(
            "Mast.Name.Lookup",
            json!(
                {
                        "input": object_id,
                        "format": "json",
                    }
            ),
        )
        .try_into()?;

        let response: Value = self
            .client
            .post(self.url.as_str())
            .form(&form)
            .send()
            .await?
            .json()
            .await?;
        let obj_ra = response["resolvedCoordinate"][0]["ra"].as_f64().unwrap();
        let obj_dec = response["resolvedCoordinate"][0]["decl"].as_f64().unwrap();

        let radius = radius.into_degree().into_value();
        let request = Request::new(
            "Mast.Catalogs.Filtered.TIC.Position.Rows",
            json!(
                {
                        "columns":"ID,ra,dec,GAIAmag,gaiabp,gaiarp",
                        "ra": obj_ra,
                        "dec": obj_dec,
                        "radius": radius,
                        "filters": [
                            {
                                "paramName": "objType",
                                "values": ["STAR"]

                            }
                        ]
                    }
            ),
        );
        let form: HashMap<_, _> = request.try_into()?;
        let response: Value = self
            .client
            .post(self.url.as_str())
            .form(&form)
            .send()
            .await?
            .json()
            .await?;
        println!("Query status: {:#}", response["status"]);

        let objects: Vec<MastObject> = serde_json::from_value(response["data"].clone())?;
        Ok(MastObjects {
            target: object_id.into(),
            origin: (obj_ra, obj_dec),
            radius: radius.to_radians(),
            objects,
        })
    }
}

#[cfg(test)]
mod tests {
    use num_complex::Complex;

    use super::*;

    #[tokio::test]
    async fn query() {
        let mast = Mast::new();
        let mut objects = mast
            .query_region("NGC 6405", SkyAngle::Arcminute(6.))
            .await
            .unwrap();
        println!("Objects #: {}", objects.len());
        println!("Valid objects #: {}", objects.n_valid());
        println!("Objects #: {}", objects.only_valid().n_valid());
    }

    #[tokio::test]
    async fn local() {
        let mast = Mast::new();
        let mut objects = mast
            .query_region("NGC 6405", SkyAngle::Arcminute(1.))
            .await
            .unwrap();
        println!("Objects #: {}", objects.len());
        println!("Valid objects #: {}", objects.n_valid());
        println!("Objects #: {}", objects.only_valid().n_valid());

        let (ra, dec) = objects.origin;
        let zc = Complex::from_polar(dec.to_radians(), ra.to_radians());

        let distances: Vec<_> = objects
            .objects
            .iter()
            .take(24)
            .map(|object| {
                let zo = Complex::from_polar(object.dec.to_radians(), object.ra.to_radians());
                let dz = zo - zc;
                SkyAngle::Radian(dz.norm()).into_arcmin()
            })
            .collect();
        println!("{distances:?}")
    }
}
