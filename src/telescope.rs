use crate::{photometry, Objects, Observer, Photometry, ZpDft};
use image::{ImageResult, Rgb, RgbImage};
use num_complex::Complex;
use rustfft::num_traits::Zero;

#[derive(Debug)]
pub struct Telescope {
    pub diameter: f64,
    pupil_size: f64,
    obscuration: Option<f64>,
    resolution: f64,
}

impl Default for Telescope {
    fn default() -> Self {
        Self {
            diameter: 1f64,
            obscuration: Default::default(),
            pupil_size: 2f64,
            resolution: 2.5e-2,
        }
    }
}

pub struct TelescopeBuilder {
    diameter: f64,
    obscuration: Option<f64>,
}

impl Telescope {
    pub fn new(diameter: f64) -> TelescopeBuilder {
        TelescopeBuilder {
            diameter,
            obscuration: None,
        }
    }
    pub fn pupil(&self, shift: Option<(f64, f64)>) -> Vec<Complex<f64>> {
        let n_px = (self.diameter / self.resolution).round() as usize + 1;
        let l = (n_px - 1) as f64;
        let r_outer = self.diameter * 0.5;
        let r_inner = self.obscuration.unwrap_or_default();
        let mut buffer: Vec<Complex<f64>> = vec![Complex::zero(); n_px * n_px];
        if let Some((hx, hy)) = shift {
            for i in 0..n_px {
                let x = (i as f64 / l - 0.5) * self.diameter;
                for j in 0..n_px {
                    let y = (j as f64 / l - 0.5) * self.diameter;
                    let r = x.hypot(y);
                    if r >= r_inner && r <= r_outer {
                        let k = i * n_px + j;
                        buffer[k] = Complex::new(1f64, 0f64)
                            * Complex::from_polar(
                                1f64,
                                -2. * std::f64::consts::PI * (x * hx + y * hy),
                            );
                    }
                }
            }
        } else {
            for i in 0..n_px {
                let x = (i as f64 / l - 0.5) * self.diameter;
                for j in 0..n_px {
                    let y = (j as f64 / l - 0.5) * self.diameter;
                    let r = x.hypot(y);
                    if r >= r_inner && r <= r_outer {
                        let k = i * n_px + j;
                        buffer[k] = Complex::new(1f64, 0f64);
                    }
                }
            }
        }
        buffer
    }
}

impl TelescopeBuilder {
    /// Sets the diameter of the telescope central obscuration
    pub fn obscuration(mut self, obscuration: f64) -> Self {
        self.obscuration = Some(obscuration);
        self
    }
    /// Build the telescope
    pub fn build(self) -> Telescope {
        Telescope {
            diameter: self.diameter,
            obscuration: self.obscuration,
            pupil_size: 2. * self.diameter,
            ..Default::default()
        }
    }
}

impl Observer for Telescope {
    fn diameter(&self) -> f64 {
        self.diameter
    }

    fn set_pupil_size(&mut self, size: f64) {
        self.pupil_size = dbg!(size);
    }

    fn intensity(&mut self, pupil_size: f64, intensity_sampling: usize, b: f64) -> Vec<f64> {
        let mut n_dft = (pupil_size / self.resolution).ceil() as usize;
        if intensity_sampling > n_dft && intensity_sampling % 2 != n_dft % 2 {
            n_dft += 1;
        }
        let shift = (intensity_sampling % 2 == 0)
            .then_some(0.5 / pupil_size)
            .map(|h| (h, h));

        let mut zp_dft = ZpDft::forward(dbg!(n_dft));
        let intensity = zp_dft
            .process(self.pupil(shift).as_slice())
            .resize(intensity_sampling)
            .norm_sqr();
        /*         let mut buffer = vec![0f64; n_dft * n_dft];
        let n_dft = n_dft as i32;
        for star in objects.iter() {
            let (x, y) = star.coordinates;
            let i0 = (x / alpha).round() as i32;
            let j0 = (y / alpha).round() as i32;
            for i in 0..n_dft {
                let ii = i0 + i;
                if ii < 0 || ii >= n_dft {
                    continue;
                }
                for j in 0..n_dft {
                    let jj = j0 + j;
                    if jj < 0 || jj >= n_dft {
                        continue;
                    }
                    let k = i * n_dft + j;
                    let kk = ii * n_dft + jj;
                    buffer[kk as usize] += intensity[k as usize];
                }
            }
        } */

        let m = b as usize;
        if m == 1 {
            return intensity;
        }
        dbg!(intensity.len());
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
                        bin += intensity[kk];
                    }
                }
                let k = i * n + j;
                image[k] = bin;
            }
        }
        image
    }

    fn show_pupil(&self) -> ImageResult<()> {
        let n = (self.diameter / self.resolution).round() as u32 + 1;
        let mut img = RgbImage::new(n, n);
        img.pixels_mut()
            .zip(self.pupil(None).into_iter())
            .filter(|(px, pup)| pup.norm() > 0f64)
            .for_each(|(px, _)| *px = Rgb([255, 255, 255]));
        img.save("telescope_pupil.png")
    }
}
