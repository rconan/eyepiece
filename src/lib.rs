use image::{ImageResult, Rgb, RgbImage};
use skyangle::SkyAngle;
use std::{ops::Deref, path::Path};

mod zpdft;
pub use zpdft::ZpDft;
mod telescope;
pub use telescope::Telescope;
mod photometry;
pub use photometry::Photometry;

#[derive(Debug, Default)]
pub struct Star {
    pub coordinates: (f64, f64),
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
    fn intensity(&mut self, size: f64, n: usize, b: f64) -> Vec<f64>;
    fn show_pupil(&self) -> ImageResult<()>;
}

/// Field of view
pub struct Field<T: Observer> {
    resolution: SkyAngle<f64>,
    field_of_view: SkyAngle<f64>,
    photometry: Photometry,
    objects: Objects,
    pub observer: T,
}
impl<T: Observer> Field<T> {
    /// Create a new field
    pub fn new<P: Into<Photometry>>(
        resolution: SkyAngle<f64>,
        field_of_view: SkyAngle<f64>,
        photometric_band: P,
        objects: Objects,
        mut observer: T,
    ) -> Self {
        let photometry: Photometry = photometric_band.into();
        // let nyquist = 0.5 * photometry.wavelength / observer.diameter();
        // let pupil_size =
        //     if (resolution.to_radians() - nyquist) < SkyAngle::MilliArcsec(1.).to_radians() {
        //         photometry.wavelength / resolution.to_radians()
        //     } else {
        //         let beta = (resolution.to_radians() / alpha).ceil();
        //         let alpha = resolution.to_radians() / dbg!(beta);
        //         photometry.wavelength / alpha
        //     };
        // observer.set_pupil_size(pupil_size);
        Self {
            resolution,
            field_of_view,
            photometry,
            objects,
            observer,
        }
    }
    pub fn save<P: AsRef<Path>>(&mut self, path: P) -> ImageResult<()> {
        let nyquist = 0.5 * self.photometry.wavelength / self.observer.diameter();
        let b = if (self.resolution.to_radians() - dbg!(nyquist))
            <= SkyAngle::MilliArcsec(1f64).to_radians()
        {
            1f64
        } else {
            (self.resolution.to_radians() / nyquist).round()
        };
        let n_px = (b * (self.field_of_view / self.resolution)).ceil() as usize;
        let pupil_size = b * self.photometry.wavelength / self.resolution.to_radians();

        let mut intensity = self
            .observer
            .intensity(dbg!(pupil_size), dbg!(n_px), dbg!(b));
        let max_intensity = intensity.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        intensity.iter_mut().for_each(|i| *i /= max_intensity);

        let lut = colorous::CUBEHELIX;
        let n_px = (self.field_of_view / self.resolution).ceil() as usize;
        let mut img = RgbImage::new(n_px as u32, n_px as u32);
        img.pixels_mut().zip(&intensity).for_each(|(p, i)| {
            *p = Rgb(lut.eval_continuous(*i).into_array());
        });
        img.save(path)
    }
}
