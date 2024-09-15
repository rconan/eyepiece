use core::f64;
use eyepiece::FieldBuilder;
use serde::{Deserialize, Serialize};
use skyangle::SkyAngle;
use std::fmt::Display;

use crate::CameraSkyPixel;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Camera {
    pub exposure: f64,
    pub pixel_scale: CameraSkyPixel,
    pub field_of_view: SkyAngle<f64>,
    pub spectral_filter: eyepiece::Photometry,
    pub photon_noise: bool,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            exposure: 1.0,
            pixel_scale: CameraSkyPixel(SkyAngle::MilliArcsec(10.0)),
            field_of_view: SkyAngle::Arcsecond(1f64),
            spectral_filter: "K".into(),
            photon_noise: false,
        }
    }
}

impl Display for Camera {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.spectral_filter, self.exposure)
    }
}

impl Camera {
    pub fn build<T: eyepiece::Observer>(&self, builder: FieldBuilder<T>) -> FieldBuilder<T> {
        let field_builder = builder
            .exposure(self.exposure)
            .pixel_scale(self.pixel_scale.clone())
            .field_of_view(self.field_of_view.clone())
            .photometry(self.spectral_filter);
        if self.photon_noise {
            field_builder.photon_noise()
        } else {
            field_builder
        }
    }
}
