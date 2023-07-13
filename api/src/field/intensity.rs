use std::path::Path;

use image::{ImageResult, Rgb, RgbImage};

use crate::{Field, Intensity, Observer, Observing, ObservingModes, PixelScale, SaveOptions};

fn shift_and_add(buffer: &mut [f64], x0: f64, y0: f64, n: i32, intensity: Vec<f64>) {
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
            buffer[kk as usize] += intensity[k as usize];
        }
    }
}

fn binning(intensity_sampling: usize, m: usize, buffer: Vec<f64>) -> Vec<f64> {
    let n = intensity_sampling / m;
    let n_buffer = n * m;
    let h = (intensity_sampling - n_buffer) / 2;
    let mut image = vec![0f64; n * n];
    let rows: Vec<_> = buffer
        .chunks(intensity_sampling)
        .flat_map(|r| r.iter().skip(h).take(n_buffer))
        .collect();
    let matched_buffer: Vec<_> = (0..n_buffer)
        .flat_map(|k| {
            rows.iter()
                .skip(k + h * n_buffer)
                .step_by(n_buffer)
                .take(n_buffer)
                .collect::<Vec<_>>()
        })
        .collect();
    for i in 0..n {
        let ii = i * m;
        for j in 0..n {
            let jj = j * m;
            let mut bin = 0f64;
            for ib in 0..m {
                for jb in 0..m {
                    let kk = (ii + ib) * n_buffer + jj + jb;
                    bin += **matched_buffer[kk];
                }
            }
            let k = i * n + j;
            image[k] = bin;
        }
    }
    image
}

#[cfg(feature = "parallel")]
mod parallel;
#[cfg(not(feature = "parallel"))]
mod serial;

#[derive(Debug, Clone)]
pub struct FieldImage {
    #[allow(dead_code)]
    pixel_scale: PixelScale,
    resolution: (usize, usize),
    pixels: Vec<f64>,
}
impl<T, M> From<Field<T, M>> for FieldImage
where
    T: Observer,
    M: ObservingModes + Send,
    Observing<M>: Intensity,
{
    fn from(mut field: Field<T, M>) -> Self {
        let pixels = field.intensity(None);
        let n = field.intensity_sampling.unwrap();
        FieldImage {
            pixel_scale: field.pixel_scale,
            resolution: (n, n),
            pixels,
        }
    }
}
impl FieldImage {
    pub fn masked(&mut self, mask: &impl Observer) -> &mut Self {
        let (n, m) = self.resolution;
        for i in 0..n {
            let y = i as f64 - 0.5 * (n - 1) as f64;
            for j in 0..m {
                let x = j as f64 - 0.5 * (m - 1) as f64;
                if !mask.inside_pupil(x, y) {
                    self.pixels[i * m + j] = 0f64;
                }
            }
        }
        self
    }
    pub fn flux(&self) -> f64 {
        self.pixels.iter().sum()
    }
    pub fn save<P: AsRef<Path>>(&self, path: P, save_options: SaveOptions) -> ImageResult<()> {
        let mut intensity = self.pixels.clone();
        match path.as_ref().extension().and_then(|p| p.to_str()) {
            Some("png" | "jpg" | "tiff") => {
                if let Some(lufn) = save_options.lufn {
                    intensity.iter_mut().for_each(|i| *i = lufn(*i));
                }

                let threshold = save_options.saturation.threshold(intensity.iter());
                intensity.iter_mut().for_each(|i| *i /= threshold);

                let lut = colorous::CUBEHELIX;
                let n_px = (intensity.len() as f64).sqrt() as usize;
                let mut img = RgbImage::new(n_px as u32, n_px as u32);
                img.pixels_mut().zip(&intensity).for_each(|(p, i)| {
                    *p = Rgb(lut.eval_continuous(*i).into_array());
                });
                img.save(path.as_ref()).expect(&format!(
                    "failed to write field intensity into image {:?}",
                    path.as_ref()
                ))
            }
            _ => unimplemented!(),
        };
        Ok(())
    }
}
