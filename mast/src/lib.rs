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
//! let mast = Mast::new("J");
//! let mut objects = mast
//!     .query_region("NGC 6405", SkyAngle::Arcminute(1.))
//!     .await
//!     .unwrap();
//! # })
//! ```

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use skyangle::SkyAngle;
use std::{collections::HashMap, fmt::Display};

const MAST_URL: &str = "https://mast.stsci.edu/api/v0/invoke";

mod objects;
pub use objects::{MastObject, MastObjects};
mod photometry;
pub(crate) use photometry::GaiaPhotometry;

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

/// MAST interface
pub struct Mast {
    client: reqwest::Client,
    #[allow(dead_code)]
    format: String,
    url: String,
    photometry: GaiaPhotometry,
}
impl Mast {
    /// Creates a new MAST interface
    pub fn new<P: Into<GaiaPhotometry>>(band: P) -> Self {
        Self {
            client: reqwest::Client::new(),
            format: "json".to_string(),
            url: MAST_URL.to_string(),
            photometry: band.into(),
        }
    }
    /// Queries a circular region aroud a given object
    pub async fn query_region<S: Into<String> + Serialize + Display>(
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

        println!("Querying {object_id} ...");
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

        let objects: Vec<MastObject> = serde_json::from_value(response["data"].clone())?;
        println!(
            "MAST query status: {:#} ({} objects)",
            response["status"],
            objects.len()
        );

        Ok(MastObjects {
            target: object_id.into(),
            origin: (obj_ra, obj_dec),
            radius: radius.to_radians(),
            objects: objects
                .into_iter()
                .filter(|object| object.is_valid())
                .collect(),
            photometry: self.photometry,
        })
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[tokio::test]
    async fn query() {
        let mast = Mast::new("V");
        let objects = mast
            .query_region("NGC 6405", SkyAngle::Arcminute(6.))
            .await
            .unwrap();
        println!("Objects #: {}", objects.len());
    }

    #[tokio::test]
    async fn local() {
        let mast = Mast::new("V");
        let objects = mast
            .query_region("NGC 6405", SkyAngle::Arcminute(1.))
            .await
            .unwrap();
        println!("Objects #: {}", objects.len());

        let origin = MastObject::from(objects.origin);

        let distances: Vec<_> = objects
            .objects
            .iter()
            // .take(24)
            .filter_map(|object| {
                let o = object.offsets(&origin);
                let a = o.0.to_radians().hypot(o.1.to_radians());
                if a > 1f64 {
                    Some(a)
                } else {
                    None
                }
            })
            .collect();
        println!("{distances:?}")
    }
}
