use serde::{Deserialize, Serialize};

mod app;
mod observation;
mod program;

use eyepiece::PixelScale;
use observation::Observation;
pub use program::Program;
use skyangle::SkyAngle;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct CameraSkyPixel(SkyAngle<f64>);

impl From<CameraSkyPixel> for PixelScale {
    fn from(value: CameraSkyPixel) -> Self {
        PixelScale::SkyAngle(value.0)
    }
}
