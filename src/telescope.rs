use crate::Observer;
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

    fn resolution(&self) -> f64 {
        self.resolution
    }

    fn pupil(&self, shift: Option<(f64, f64)>) -> Vec<Complex<f64>> {
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
