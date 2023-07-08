use std::fmt::Display;

use serde::Serialize;
use skyangle::SkyAngle;

use crate::{AdaptiveOpticsCorrection, Photometry, Star};

/// Atmospheric seeing builder
///
/// # Example
/// ```
/// use eyepiece::SeeingBuilder;
/// use skyangle::SkyAngle;
///
/// let seeing = SeeingBuilder::new(16e-2)
///     .zenith_angle(SkyAngle::Degree(30.))
///     .outer_scale(30.);
/// ```
#[derive(Debug, Clone, Serialize)]
pub struct SeeingBuilder {
    pub(crate) fried_parameter: f64,
    pub(crate) outer_scale: f64,
    pub(crate) adaptive_optics: Option<AdaptiveOpticsCorrection>,
}
impl Display for SeeingBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "seeing limited:")?;
        writeln!(f, " . Fried parameter: {:.3}cm", self.fried_parameter * 1e2)?;
        writeln!(f, " . outer scale: {:.3}m", self.outer_scale)?;
        if let Some(ao) = &self.adaptive_optics {
            write!(f, r" . with {}", ao)?;
        }
        Ok(())
    }
}
impl SeeingBuilder {
    /// Creates a new atmospheric seeing builder by setting the Fried parameter in meters
    ///
    /// The outer scale is set to 25m.
    pub fn new(fried_parameter: f64) -> Self {
        Self {
            fried_parameter,
            outer_scale: 25.,
            adaptive_optics: None,
        }
    }
    /// Sets the atmosphere outer scale on meters
    pub fn outer_scale(self, outer_scale: f64) -> Self {
        Self {
            outer_scale,
            ..self
        }
    }
    /// Scales the Fried parameter according to the zenith angle
    pub fn zenith_angle(self, zenith_angle: SkyAngle<f64>) -> Self {
        Self {
            fried_parameter: self.fried_parameter
                * zenith_angle.to_radians().cos().powf(3_f64 / 5_f64),
            ..self
        }
    }
    /// Reduces the seeing FWHM by the given fraction
    pub fn glao(self, corrected_fraction: f64) -> Self {
        assert!(
            corrected_fraction < 1f64,
            "GLAO fraction of correction should be less that 1"
        );
        Self {
            fried_parameter: self.fried_parameter / (1. - corrected_fraction),
            ..self
        }
    }
    /// Scales the Fried parameter according to the wavelength of the [photometric bands](crate::Photometry)
    pub(crate) fn wavelength<P: Into<Photometry>>(self, band: P) -> Self {
        let photometry: Photometry = band.into();
        Self {
            fried_parameter: self.fried_parameter
                * (photometry.wavelength / Photometry::from("V").wavelength).powf(1.2_f64),
            ..self
        }
    }
    /// Corrects the seeing with a Natural Guide Star Adaptive Optics system
    ///
    /// Only the fitting and anisoplanatism errors of the NGAO system are modeled.
    /// The fitting error is set according to the Strehl ratio (≥ 0.5).
    /// The anisoplanatism error is set only if a guide star is given.
    pub fn ngao(self, strehl_ratio: f64, guide_star: Option<Star>) -> Self {
        if strehl_ratio < 0.5 {
            panic!("Strel ratio must be at least 0.5 or higher");
        }
        Self {
            adaptive_optics: Some(AdaptiveOpticsCorrection::ngao(strehl_ratio, guide_star)),
            ..self
        }
    }
    /// Corrects the seeing with a Laser Guide Star Adaptive Optics system
    ///
    /// Only the fitting and anisoplanatism errors of the LTAO system are modeled.
    /// The fitting error is set according to the Strehl ratio (≥ 0.5).
    /// The anisoplanatism error is set only outside the Laser guide stars radius.
    pub fn ltao(self, strehl_ratio: f64, laser_guide_star_radius: SkyAngle<f64>) -> Self {
        if strehl_ratio < 0.5 {
            panic!("Strel ratio must be at least 0.5 or higher");
        }
        Self {
            adaptive_optics: Some(AdaptiveOpticsCorrection::ltao(
                strehl_ratio,
                laser_guide_star_radius,
            )),
            ..self
        }
    }
}
