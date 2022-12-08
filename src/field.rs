use std::{ops::Deref, path::Path};

use image::{ImageResult, Rgb, RgbImage};
use skyangle::SkyAngle;

use crate::{Objects, Observer, Photometry, Star, ZpDft};

/// Field of view
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
    /// Create a new field
    pub fn new<P: Into<Photometry>, O: Into<Objects>>(
        resolution: SkyAngle<f64>,
        field_of_view: SkyAngle<f64>,
        photometric_band: P,
        objects: O,
        observer: T,
    ) -> Self {
        let photometry: Photometry = photometric_band.into();
        Self {
            resolution,
            field_of_view,
            photometry,
            objects: objects.into(),
            observer,
        }
    }
    pub fn intensity(&self) -> Vec<f64> {
        let nyquist = 0.5 * self.photometry.wavelength / self.observer.diameter();
        let b = if (self.resolution.to_radians() - dbg!(nyquist))
            <= SkyAngle::MilliArcsec(1f64).to_radians()
        {
            1f64
        } else {
            (self.resolution.to_radians() / nyquist).round()
        };
        let intensity_sampling = (b * (self.field_of_view / self.resolution)).ceil() as usize;
        let pupil_size = b * self.photometry.wavelength / self.resolution.to_radians();

        let mut n_dft = (pupil_size / self.observer.resolution()).ceil() as usize;
        if intensity_sampling > n_dft && intensity_sampling % 2 != n_dft % 2 {
            n_dft += 1;
        }

        let mut zp_dft = ZpDft::forward(dbg!(n_dft));

        // stacking
        let mut buffer = vec![0f64; intensity_sampling.pow(2)];
        let n = intensity_sampling as i32;
        let alpha = self.resolution / b;
        dbg!(alpha);
        for star in self.objects.iter() {
            let (x, y) = star.coordinates;
            let x0 = -(y / alpha).round();
            let y0 = (x / alpha).round();
            let fr_x0 = -y.to_radians() - x0 * alpha;
            let fr_y0 = x.to_radians() - y0 * alpha;

            let shift = if (intensity_sampling % 2 == 0) {
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
            dbg!(shift);

            let intensity = zp_dft
                .reset()
                .process(self.observer.pupil(shift).as_slice())
                .resize(intensity_sampling)
                .norm_sqr();

            let i0 = x0 as i32;
            let j0 = y0 as i32;
            dbg!((i0, j0));

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
                    buffer[kk as usize] += intensity[k as usize];
                }
            }
        }

        let m = b as usize;
        if m == 1 {
            return buffer;
        }
        // binning
        dbg!(buffer.len());
        let n = intensity_sampling / m;
        let mut image = vec![0f64; dbg!(n) * n];
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
