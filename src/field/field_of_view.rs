use skyangle::SkyAngle;

use super::Field;
use crate::{Observer, Photometry};

#[derive(Debug, Clone)]
/// Field-of-view possible representations ...
pub enum FieldOfView {
    /// ... as a multiple of the pixel scale
    PixelScale(usize),
    /// ... as a multiple of the pixel scale in the given [photometric band](crate::PhotometricBands)
    PixelScaleAt(usize, String),
    /// ... as an [angle in the sky](https://docs.rs/skyangle/)
    SkyAngle(SkyAngle<f64>),
}
impl From<SkyAngle<f64>> for FieldOfView {
    fn from(alpha: SkyAngle<f64>) -> Self {
        FieldOfView::SkyAngle(alpha)
    }
}
impl From<usize> for FieldOfView {
    fn from(n: usize) -> Self {
        FieldOfView::PixelScale(n)
    }
}
impl<T: Observer, M> From<&Field<T, M>> for FieldOfView {
    fn from(field: &Field<T, M>) -> Self {
        FieldOfView::SkyAngle(SkyAngle::Radian(field.field_of_view()))
    }
}
impl FieldOfView {
    pub(super) fn get<T: Observer, M>(&self, field: &Field<T, M>) -> f64 {
        match self {
            FieldOfView::PixelScale(n) => field.resolution() * *n as f64,
            FieldOfView::PixelScaleAt(n, band) => {
                let photometry: Photometry = band.into();
                field.pixel_scale.get(&field.observer, &photometry) * *n as f64
            }
            FieldOfView::SkyAngle(val) => val.to_radians(),
        }
    }
    pub(super) fn to_pixelscale_ratio<T: Observer, M>(&self, field: &Field<T, M>) -> f64 {
        match self {
            FieldOfView::PixelScale(n) => *n as f64,
            FieldOfView::PixelScaleAt(..) => self.get(field) / field.resolution(),
            FieldOfView::SkyAngle(val) => val.to_radians() / field.resolution(),
        }
    }
}
