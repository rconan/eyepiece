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
#[derive(Debug, Clone)]
pub struct SeeingBuilder {
    pub fried_parameter: f64,
    pub outer_scale: f64,
    pub adaptive_optics: Option<AdaptiveOpticsCorrection>,
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
                * (photometry.wavelength / 0.5e-6_f64).powf(1.2_f64),
            ..self
        }
    }
    /// Corrects the seeing with a Natural Guide Star Adaptive Optics system
    ///
    /// Only the fitting and anisoplanatism errors of the NGAO system are modeled.
    /// The fitting error is set according to the Strehl ratio.
    /// The anisoplanatism error is set only if a guide star is given.
    pub fn ngao(self, strehl_ratio: f64, guide_star: Option<Star>) -> Self {
        if strehl_ratio < 0.5 {
            panic!("Strel ratio must be at least 0.5 or higher");
        }
        Self {
            adaptive_optics: Some(AdaptiveOpticsCorrection::new(strehl_ratio, guide_star)),
            ..self
        }
    }
}
