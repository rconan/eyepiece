use std::ops::Deref;

mod zpdft;
use num_complex::Complex;
use skyangle::SkyAngle;
pub use zpdft::ZpDft;
mod telescope;
pub use telescope::Telescope;
mod photometry;
pub use photometry::Photometry;
mod field;
pub use field::Field;

type SkyCoordinates = (SkyAngle<f64>, SkyAngle<f64>);

#[derive(Debug)]
pub struct Star {
    pub coordinates: SkyCoordinates,
    magnitude: f64,
}
impl Default for Star {
    fn default() -> Self {
        Self {
            coordinates: (SkyAngle::Arcsecond(0f64), SkyAngle::Arcsecond(0f64)),
            magnitude: Default::default(),
        }
    }
}
impl Star {
    pub fn new(coordinates: SkyCoordinates) -> Self {
        Self {
            coordinates,
            ..Default::default()
        }
    }
}
pub struct Objects(Vec<Star>);
impl Deref for Objects {
    type Target = Vec<Star>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl Default for Objects {
    fn default() -> Self {
        Self(Default::default())
    }
}
impl From<Star> for Objects {
    fn from(star: Star) -> Self {
        Self(vec![star])
    }
}
impl From<Vec<Star>> for Objects {
    fn from(stars: Vec<Star>) -> Self {
        Self(stars)
    }
}
pub trait Observer {
    fn diameter(&self) -> f64;
    fn resolution(&self) -> f64;
    fn pupil(&self, shift: Option<(f64, f64)>) -> Vec<Complex<f64>>;
    fn show_pupil(&self) -> image::ImageResult<()>;
}
