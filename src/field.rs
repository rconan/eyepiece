use image::{ImageResult, Rgb, RgbImage};
use indicatif::{ProgressBar, ProgressStyle};
use skyangle::SkyAngle;
use std::path::Path;

use crate::{Objects, Observer, Photometry, ZpDft};

/// Observer field of regard
pub struct Field<T>
where
    T: Observer,
{
    resolution: SkyAngle<f64>,
    field_of_view: SkyAngle<f64>,
    photometry: Photometry,
    objects: Objects,
    pub observer: T,
}
impl<T> Field<T>
where
    T: Observer,
{
    /// Creates a new field
    pub fn new<P: Into<Photometry>, O: Into<Objects>>(
        resolution: SkyAngle<f64>,
        field_of_view: SkyAngle<f64>,
        photometric_band: P,
        objects: O,
        observer: T,
    ) -> Self {
        Self {
            resolution,
            field_of_view,
            photometry: photometric_band.into(),
            objects: objects.into(),
            observer,
        }
    }
    /// Computes field-of-view intensity map
    pub fn intensity(&self) -> Vec<f64> {
        // Telescope Nyquist-Shannon sampling criteria
        let nyquist = 0.5 * self.photometry.wavelength / self.observer.diameter();
        // Image resolution to sampling criteria ratio
        let b = if (self.resolution.to_radians() - nyquist)
            <= SkyAngle::MilliArcsec(1f64).to_radians()
        {
            1f64
        } else {
            (self.resolution.to_radians() / nyquist).round() // todo: should be ceil
        };
        // Intensity sampling (oversampled wrt. image by factor b>=1)
        let intensity_sampling = (b * (self.field_of_view / self.resolution)).ceil() as usize;
        // Pupil size according to intensity angular resolution
        let pupil_size = b * self.photometry.wavelength / self.resolution.to_radians();
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
        let alpha = self.resolution / b;
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
        let n_px = (self.field_of_view / self.resolution).ceil() as usize;
        let mut img = RgbImage::new(n_px as u32, n_px as u32);
        img.pixels_mut().zip(&intensity).for_each(|(p, i)| {
            *p = Rgb(lut.eval_continuous(*i).into_array());
        });
        img.save(path)
    }
}
