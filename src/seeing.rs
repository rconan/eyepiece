use skyangle::SkyAngle;

use crate::{ObservingMode, Photometry};

#[derive(Clone, Debug, Copy)]
/// Atmospheric seeing builder
///
/// # Example
/// ```
/// use eyepiece::SeeingBuilder;
/// use skyangle::SkyAngle;
/// 
/// let seeing = SeeingBuilder::new(16e-2)
///     .zenith_angle(SkyAngle::Degree(30.))
///     .outer_scale(30.)
///     .build();
/// ```
pub struct SeeingBuilder {
    fried_parameter: f64,
    outer_scale: f64,
}
impl SeeingBuilder {
    /// Creates a new atmospheric seeing builder by setting the Fried parameter in meters
    pub fn new(fried_parameter: f64) -> Self {
        Self {
            fried_parameter,
            outer_scale: 25.,
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
    /// Scales the Fried parameter according to the wavelength of the [photometric bands](crate::Photometry)
    pub(crate) fn wavelength<P: Into<Photometry>>(self, band: P) -> Self {
        let photometry: Photometry = band.into();
        Self {
            fried_parameter: self.fried_parameter
                * (photometry.wavelength / 0.5e-6_f64).powf(1.2_f64),
            ..self
        }
    }
    /// Creates a seeing limited [ObservingMode]
    pub fn build(self) -> ObservingMode {
        log::debug!(
            r"
 . Fried parameter: {:.3}m
 . outer scale: {:.3}m 
        ",
            self.fried_parameter,
            self.outer_scale
        );
        ObservingMode::SeeingLimited {
            fried_parameter: self.fried_parameter,
            outer_scale: self.outer_scale,
        }
    }
}
