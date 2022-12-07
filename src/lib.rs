use image::ImageResult;
use skyangle::SkyAngle;
use std::ops::Deref;

mod zpdft;
pub use zpdft::ZpDft;
mod telescope;
pub use telescope::Telescope;
mod photometry;
pub use photometry::Photometry;

#[derive(Debug, Default)]
pub struct Star {
    coordinates: (f64, f64),
    magnitude: f64,
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

pub trait Observer {
    fn diameter(&self) -> f64;
    fn set_pupil_size(&mut self, size: f64);
    fn intensity(&self) -> Vec<f64>;
    fn show_pupil(&self) -> ImageResult<()>;
}

pub struct Field<T: Observer> {
    resolution: SkyAngle<f64>,
    field_of_view: SkyAngle<f64>,
    photometry: Photometry,
    objects: Objects,
    pub observer: T,
}
impl<T: Observer> Field<T> {
    pub fn new<P: Into<Photometry>>(
        resolution: SkyAngle<f64>,
        field_of_view: SkyAngle<f64>,
        photometric_band: P,
        objects: Objects,
        mut observer: T,
    ) -> Self {
        let photometry: Photometry = photometric_band.into();
        let alpha = 0.5 * photometry.wavelength / observer.diameter();
        let pupil_size = if resolution.to_radians() <= alpha {
            photometry.wavelength / resolution.to_radians()
        } else {
            let beta = (resolution.to_radians() / alpha).ceil();
            let alpha = resolution.to_radians() / dbg!(beta);
            photometry.wavelength / alpha
        };
        observer.set_pupil_size(pupil_size);
        Self {
            resolution,
            field_of_view,
            photometry,
            objects,
            observer,
        }
    }
}
