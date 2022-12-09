use image::{ImageResult, Rgb, RgbImage};
use indicatif::{ProgressBar, ProgressStyle};
use skyangle::SkyAngle;
use std::path::Path;

use crate::{Objects, Observer, Photometry, ZpDft};

pub enum PixelScale {
    Nyquist(u32),
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
impl From<u32> for PixelScale {
    fn from(n: u32) -> Self {
        PixelScale::Nyquist(n)
    }
}
impl PixelScale {
    fn get<T: Observer>(&self, observer: &T, photometry: &Photometry) -> f64 {
        match self {
            PixelScale::Nyquist(n) => 0.5 * photometry.wavelength / observer.diameter() / *n as f64,
            PixelScale::SkyAngle(val) => val.to_radians(),
        }
    }
    fn to_nyquist_ratio<T: Observer>(&self, observer: &T, photometry: &Photometry) -> f64 {
        match self {
            PixelScale::Nyquist(_) => 1f64,
            PixelScale::SkyAngle(val) => {
                (2. * val.to_radians() * observer.diameter() / photometry.wavelength).ceil()
            }
        }
    }
}

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
    fn to_pixelscale_ratio<T: Observer>(&self, field: &Field<T>) -> f64 {
        match self {
            FieldOfView::PixelScale(n) => *n as f64,
            FieldOfView::SkyAngle(val) => val.to_radians() / field.resolution(),
        }
    }
}
/// Observer field of regard
pub struct Field<T>
where
    T: Observer,
{
    pixel_scale: PixelScale,
    field_of_view: FieldOfView,
    photometry: Photometry,
    objects: Objects,
    pub observer: T,
}
impl<T> Field<T>
where
    T: Observer,
{
    /// Creates a new field
    pub fn new<X, F, P, O>(
        resolution: X,
        field_of_view: F,
        photometric_band: P,
        objects: O,
        observer: T,
    ) -> Self
    where
        X: Into<PixelScale>,
        F: Into<FieldOfView>,
        P: Into<Photometry>,
        O: Into<Objects>,
    {
        Self {
            pixel_scale: resolution.into(),
            field_of_view: field_of_view.into(),
            photometry: photometric_band.into(),
            objects: objects.into(),
            observer,
        }
    }
    fn resolution(&self) -> f64 {
        self.pixel_scale.get(&self.observer, &self.photometry)
    }
    /// Computes field-of-view intensity map
    pub fn intensity(&self) -> Vec<f64> {
        // Telescope Nyquist-Shannon sampling criteria
        // let nyquist = 0.5 * self.photometry.wavelength / self.observer.diameter();
        // Image resolution to sampling criteria ratio
        let b = self
            .pixel_scale
            .to_nyquist_ratio(&self.observer, &self.photometry);
        // Intensity sampling (oversampled wrt. image by factor b>=1)
        let intensity_sampling = (b * self.field_of_view.to_pixelscale_ratio(self)).ceil() as usize;
        // Pupil size according to intensity angular resolution
        let pupil_size = b * self.photometry.wavelength / self.resolution();
        // FFT sampling based on pupil spatial resolution
        let mut n_dft = (pupil_size / self.observer.resolution()).ceil() as usize;
        // Match parity of FFT and intensity sampling if the latter is larger
        if intensity_sampling > n_dft && intensity_sampling % 2 != n_dft % 2 {
            n_dft += 1;
        }
        // Zero-padding discrete Fourier transform
        let mut zp_dft = ZpDft::forward(n_dft);

        // star image stacking buffer
        let mut buffer = vec![0f64; intensity_sampling.pow(2)];
        let n = intensity_sampling as i32;
        let alpha = self.resolution() / b;
        let bar = ProgressBar::new(self.objects.len() as u64);
        bar.set_style(
            ProgressStyle::with_template("[{eta}] {bar:40.cyan/blue} {pos:>5}/{len:5}").unwrap(),
        );
        for star in self.objects.iter() {
            bar.inc(1);
            // todo: check if star is within FOV (rejection criteria?)
            let n_photon = self.photometry.n_photon(star.magnitude);
            // star coordinates
            let (x, y) = star.coordinates;
            // integer part
            let x0 = -(y / alpha).round();
            let y0 = (x / alpha).round();
            // fractional part
            let fr_x0 = -y.to_radians() - x0 * alpha;
            let fr_y0 = x.to_radians() - y0 * alpha;
            // image fractional translation by Fourier interpolation
            let shift = if intensity_sampling % 2 == 0 {
                Some((
                    0.5 / pupil_size + fr_x0 / self.photometry.wavelength,
                    0.5 / pupil_size + fr_y0 / self.photometry.wavelength,
                ))
            } else {
                Some((
                    fr_x0 / self.photometry.wavelength,
                    fr_y0 / self.photometry.wavelength,
                ))
            };
            // star intensity map
            let intensity = zp_dft
                .reset()
                .process(self.observer.pupil(shift).as_slice())
                .resize(intensity_sampling)
                .norm_sqr();
            // shift and add star images
            let i0 = x0 as i32;
            let j0 = y0 as i32;
            for i in 0..n {
                let ii = i0 + i;
                if ii < 0 || ii >= n {
                    continue;
                }
                for j in 0..n {
                    let jj = j0 + j;
                    if jj < 0 || jj >= n {
                        continue;
                    }
                    let k = i * n + j;
                    let kk = ii * n + jj;
                    buffer[kk as usize] += intensity[k as usize] * n_photon;
                }
            }
        }
        bar.finish();

        let m = b as usize;
        if m == 1 {
            return buffer;
        }
        // binning
        let n = intensity_sampling / m;
        let mut image = vec![0f64; n * n];
        for i in 0..n {
            let ii = i * m;
            for j in 0..n {
                let jj = j * m;
                let mut bin = 0f64;
                for ib in 0..m {
                    for jb in 0..m {
                        let kk = (ii + ib) * intensity_sampling + jj + jb;
                        bin += buffer[kk];
                    }
                }
                let k = i * n + j;
                image[k] = bin;
            }
        }
        image
    }
    /// Computes image and save it to file
    pub fn save<P: AsRef<Path>>(&mut self, path: P) -> ImageResult<()> {
        let mut intensity = self.intensity();
        let max_intensity = intensity.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        intensity.iter_mut().for_each(|i| *i /= max_intensity);

        let lut = colorous::CUBEHELIX;
        let n_px = self.field_of_view.to_pixelscale_ratio(self).ceil() as usize;
        let mut img = RgbImage::new(n_px as u32, n_px as u32);
        img.pixels_mut().zip(&intensity).for_each(|(p, i)| {
            *p = Rgb(lut.eval_continuous(*i).into_array());
        });
        img.save(path)
    }
}
