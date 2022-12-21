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

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use skyangle::SkyAngle;
use std::{collections::HashMap, ops::Deref};

const MAST_URL: &str = "https://mast.stsci.edu/api/v0/invoke";

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
pub struct MastObjects(Vec<MastObject>);
impl Deref for MastObjects {
    type Target = Vec<MastObject>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl MastObjects {
    /// Returns the number of valid objects
    ///
    /// Valid objects are objects with valid GAIA properties
    pub fn n_valid(&self) -> usize {
        self.0
            .iter()
            .filter(|object| object.is_valid())
            .map(|_| 1usize)
            .sum()
    }
    /// Keeps only valid objects
    pub fn only_valid(&mut self) -> &mut Self {
        self.0 = self
            .0
            .iter()
            .filter(|object| object.is_valid())
            .cloned()
            .collect();
        self
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
    pub async fn query_region<S: Serialize>(
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
        let obj_ra = &response["resolvedCoordinate"][0]["ra"];
        let obj_dec = &response["resolvedCoordinate"][0]["decl"];

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

        let objects: MastObjects = serde_json::from_value(response["data"].clone())?;
        Ok(objects)
    }
}

#[cfg(test)]
mod tests {
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
}
