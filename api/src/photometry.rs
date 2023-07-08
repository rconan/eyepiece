use std::{fmt::Display, ops::Deref};

use serde::Serialize;

/// Star photometry
///
/// Photometry is available for the following bands: V, R, I, J, H and K
/// ## Example
/// ```
/// use eyepiece::Photometry;
/// let photometry: Photometry = "V".into();
/// ```
#[derive(Debug, Clone, Copy, Serialize)]
pub enum Photometry {
    V(PhotometryData),
    R(PhotometryData),
    I(PhotometryData),
    J(PhotometryData),
    H(PhotometryData),
    K(PhotometryData),
}
impl Deref for Photometry {
    type Target = PhotometryData;

    fn deref(&self) -> &Self::Target {
        match self {
            Photometry::V(p) => p,
            Photometry::R(p) => p,
            Photometry::I(p) => p,
            Photometry::J(p) => p,
            Photometry::H(p) => p,
            Photometry::K(p) => p,
        }
    }
}
/// Photometric data
#[derive(Debug, Clone, Copy, Serialize)]
pub struct PhotometryData {
    pub wavelength: f64,
    zeropoint: f64,
    #[allow(dead_code)]
    spectral_bandwidth: f64,
}
impl Photometry {
    /// Returns the number of photon for the given magnitude
    pub fn n_photon(&self, magnitude: f64) -> f64 {
        self.zeropoint * 10f64.powf(-0.4 * magnitude)
    }
}

/// Astronomical photometric bands
pub struct PhotometricBands<'a>([&'a str; 6]);
impl<'a> Default for PhotometricBands<'a> {
    /// Returns the array `["V", "R", "I", "J", "H", "K"]`
    fn default() -> Self {
        Self(["V", "R", "I", "J", "H", "K"])
    }
}
impl<'a> IntoIterator for PhotometricBands<'a> {
    type Item = &'a str;

    type IntoIter = std::array::IntoIter<&'a str, 6>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl From<&str> for Photometry {
    /// Astronomical photometric bands
    ///
    /// Converts the bands V, R, I, J, H and K into star [Photometry]
    fn from(band: &str) -> Self {
        match band {
            "V" => Photometry::V(PhotometryData {
                wavelength: 0.55e-6,
                zeropoint: 8.97e9,
                spectral_bandwidth: 0.09e-6,
            }),
            "R" => Photometry::R(PhotometryData {
                wavelength: 0.64e-6,
                zeropoint: 10.87e9,
                spectral_bandwidth: 0.15e-6,
            }),
            "I" => Photometry::I(PhotometryData {
                wavelength: 0.79e-6,
                zeropoint: 7.34e9,
                spectral_bandwidth: 0.15e-6,
            }),
            "J" => Photometry::J(PhotometryData {
                wavelength: 1.215e-6,
                zeropoint: 5.16e9,
                spectral_bandwidth: 0.26e-6,
            }),
            "H" => Photometry::H(PhotometryData {
                wavelength: 1.654e-6,
                zeropoint: 2.99e9,
                spectral_bandwidth: 0.29e-6,
            }),
            "K" => Photometry::K(PhotometryData {
                wavelength: 2.179e-6,
                zeropoint: 1.90e9,
                spectral_bandwidth: 0.41e-6,
            }),
            _ => panic!("expected the photometric band: V, R, I, J, H or K, found {band}"),
        }
    }
}
impl From<&String> for Photometry {
    fn from(band: &String) -> Self {
        band.as_str().into()
    }
}
impl From<String> for Photometry {
    fn from(band: String) -> Self {
        band.as_str().into()
    }
}
impl Display for Photometry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Photometry::V(_) => "V",
                Photometry::R(_) => "R",
                Photometry::I(_) => "I",
                Photometry::J(_) => "J",
                Photometry::H(_) => "H",
                Photometry::K(_) => "K",
            }
        )
    }
}
