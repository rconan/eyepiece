use image::{ImageResult, Rgb, RgbImage};
use indicatif::ProgressBar;
use rand_distr::{Distribution, Poisson};
use skyangle::Conversion;
use std::{fmt::Display, path::Path};

use super::{FieldOfView, PixelScale};
use crate::{Objects, Observer, Photometry, ZpDft};

/// Observer field of regard
pub struct Field<T>
where
    T: Observer,
{
    pub(super) pixel_scale: PixelScale,
    field_of_view: FieldOfView,
    photometry: Photometry,
    objects: Objects,
    exposure: f64,
    poisson_noise: bool,
    pub observer: T,
}
impl<T: Observer> Display for Field<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Field in {} band", self.photometry)?;
        writeln!(f, " . pixel scale: {:.3}mas", self.resolution().to_mas())?;
        writeln!(
            f,
            " . field-of-view: {:.3}arcsec",
            self.field_of_view.get(self).to_arcsec()
        )?;
        writeln!(f, " . pupil area: {:.3}m^2", self.observer.area())?;
        let n_star: usize = self
            .objects
            .iter()
            .filter_map(|star| {
                let half_fov = self.field_of_view.get(self) * 0.5;
                let (x, y) = star.coordinates;
                if x.to_radians().abs() <= half_fov && y.to_radians().abs() <= half_fov {
                    Some(1)
                } else {
                    None
                }
            })
            .sum();
        writeln!(f, " . star #: {n_star}")?;
        let magnitude_max = self
            .objects
            .iter()
            .fold(f64::NEG_INFINITY, |a, s| a.max(s.magnitude));
        let magnitude_min = self
            .objects
            .iter()
            .fold(f64::INFINITY, |a, s| a.min(s.magnitude));
        writeln!(
            f,
            " . star magnitudes: [{magnitude_min:.1},{magnitude_max:.1}]"
        )?;
        writeln!(f, " . exposure time: {}s", self.exposure)
    }
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
            exposure: 1f64,
            observer,
            poisson_noise: false,
        }
    }
    /// Sets the exposure time in seconds
    pub fn exposure(mut self, value: f64) -> Self {
        self.exposure = value;
        self
    }
    /// Adds photon noise to the image
    pub fn photon_noise(mut self) -> Self {
        self.poisson_noise = true;
        self
    }
    pub fn resolution(&self) -> f64 {
        self.pixel_scale.get(&self.observer, &self.photometry)
    }
    pub fn field_of_view(&self) -> f64 {
        self.field_of_view.get(self)
    }
    /// Computes field-of-view intensity map
    pub fn intensity(&self, bar: Option<ProgressBar>) -> Vec<f64> {
        // Telescope Nyquist-Shannon sampling criteria
        // let nyquist = 0.5 * self.photometry.wavelength / self.observer.diameter();
        // Image resolution to sampling criteria ratio
        let b = self
            .pixel_scale
            .to_nyquist_clamped_ratio(&self.observer, &self.photometry);
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
        let mut rng = rand::thread_rng();
        for star in self.objects.iter() {
            bar.as_ref().map(|b| b.inc(1));
            // todo: check if star is within FOV (rejection criteria?)
            if !star.inside_box(self.field_of_view() + self.resolution() * 2.) {
                continue;
            }
            let n_photon =
                self.photometry.n_photon(star.magnitude) * self.observer.area() * self.exposure;
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
            let mut intensity = zp_dft
                .reset()
                .process(self.observer.pupil(shift).as_slice())
                .resize(intensity_sampling)
                .norm_sqr();
            // intensity set to # of photon & Poisson noise
            let intensity_sum: f64 = intensity.iter().cloned().sum();
            let inorm = n_photon / intensity_sum;
            if self.poisson_noise {
                intensity.iter_mut().for_each(|i| {
                    let poi = Poisson::new(*i * inorm).unwrap();
                    *i = poi.sample(&mut rng)
                })
            } else {
                intensity.iter_mut().for_each(|i| {
                    *i *= inorm;
                })
            };
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
        bar.as_ref().map(|b| b.finish());

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
    pub fn save<P: AsRef<Path>>(&mut self, path: P, bar: Option<ProgressBar>) -> ImageResult<()> {
        let mut intensity = self.intensity(bar);
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
