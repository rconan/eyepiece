use skyangle::SkyAngle;

use crate::{Observer, Photometry};

#[derive(Debug, Clone)]
/// Pixel scale possible representations ...
pub enum PixelScale {
    /// ... as a multiple of the Nyquist sampling criteria λ/2D
    Nyquist(u32),
    /// ... as a multiple of the Nyquist sampling criteria λ/2D in the given [photometric band](crate::PhotometricBands)
    NyquistAt(u32, String),
    /// ... as a fraction of the Nyquist sampling criteria λ/2D
    NyquistFraction(u32),
    /// ... as a fraction of the Nyquist sampling criteria λ/2D in the given [photometric band](crate::PhotometricBands)
    NyquistFractionAt(u32, String),
    /// ... as an [angle in the sky](https://docs.rs/skyangle/)
    SkyAngle(SkyAngle<f64>),
}
impl Default for PixelScale {
    fn default() -> Self {
        Self::Nyquist(1)
    }
}
impl From<SkyAngle<f64>> for PixelScale {
    fn from(alpha: SkyAngle<f64>) -> Self {
        PixelScale::SkyAngle(alpha)
    }
}
impl PixelScale {
    pub(super) fn get<T: Observer>(&self, observer: &T, photometry: &Photometry) -> f64 {
        match self {
            PixelScale::NyquistFraction(n) => {
                0.5 * photometry.wavelength / observer.diameter() / *n as f64
            }
            PixelScale::NyquistFractionAt(n, band) => {
                let photometry: Photometry = band.into();
                0.5 * photometry.wavelength / observer.diameter() / *n as f64
            }
            PixelScale::Nyquist(n) => 0.5 * photometry.wavelength / observer.diameter() * *n as f64,
            PixelScale::NyquistAt(n, band) => {
                let photometry: Photometry = band.into();
                0.5 * photometry.wavelength / observer.diameter() * *n as f64
            }
            PixelScale::SkyAngle(val) => val.to_radians(),
        }
    }
    pub(super) fn to_nyquist_clamped_ratio<T: Observer>(
        &self,
        observer: &T,
        photometry: &Photometry,
    ) -> f64 {
        match self {
            PixelScale::NyquistFraction(_) => 1f64,
            PixelScale::NyquistFractionAt(..) => 1f64,
            PixelScale::Nyquist(n) => *n as f64,
            PixelScale::NyquistAt(n, _) => *n as f64,
            PixelScale::SkyAngle(val) => {
                (2. * val.to_radians() * observer.diameter() / photometry.wavelength).ceil()
            }
        }
    }
}
