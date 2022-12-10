use skyangle::SkyAngle;

use super::Field;
use crate::Observer;

pub enum FieldOfView {
    PixelScale(usize),
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
impl FieldOfView {
    #[allow(dead_code)]
    fn get<T: Observer>(&self, field: &Field<T>) -> f64 {
        match self {
            FieldOfView::PixelScale(n) => field.resolution() * *n as f64,
            FieldOfView::SkyAngle(val) => val.to_radians(),
        }
    }
    pub(super) fn to_pixelscale_ratio<T: Observer>(&self, field: &Field<T>) -> f64 {
        match self {
            FieldOfView::PixelScale(n) => *n as f64,
            FieldOfView::SkyAngle(val) => val.to_radians() / field.resolution(),
        }
    }
}
