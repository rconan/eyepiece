use std::path::{Path, PathBuf};

use image::{ImageResult, Rgb, RgbImage};
use num_complex::Complex;
use num_traits::Zero;

mod zpdft;
pub use zpdft::ZpDft;
mod telescope;
pub use telescope::{Gmt, Hexagon, Hst, Jwst, Telescope, TelescopeBuilder};
mod photometry;
pub use photometry::{PhotometricBands, Photometry};
mod field;
pub use field::{Field, FieldOfView, PixelScale};
mod objects;
pub use objects::{MagnitudeDistribution, Objects, Star, StarDistribution};

pub trait Observer {
    /// Returns telescope diameter
    fn diameter(&self) -> f64;
    /// Returns pupil resolution
    fn resolution(&self) -> f64 {
        2.5e-2
    }
    /// Checks if a point is inside the pupil
    fn inside_pupil(&self, x: f64, y: f64) -> bool;
    /// Computes the pupil map
    fn pupil(&self, shift: Option<(f64, f64)>) -> Vec<Complex<f64>> {
        let diameter = self.diameter();
        let n_px = (diameter / self.resolution()).round() as usize + 1;
        let l = (n_px - 1) as f64;
        let mut buffer: Vec<Complex<f64>> = vec![Complex::zero(); n_px * n_px];
        if let Some((hx, hy)) = shift {
            for i in 0..n_px {
                let y = (i as f64 / l - 0.5) * diameter;
                for j in 0..n_px {
                    let x = (j as f64 / l - 0.5) * diameter;
                    if self.inside_pupil(x, y) {
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
                let y = (i as f64 / l - 0.5) * diameter;
                for j in 0..n_px {
                    let x = (j as f64 / l - 0.5) * diameter;
                    if self.inside_pupil(x, y) {
                        let k = i * n_px + j;
                        buffer[k] = Complex::new(1f64, 0f64);
                    }
                }
            }
        }
        buffer
    }
    /// Returns the pupil area
    fn area(&self) -> f64 {
        self.pupil(None)
            .into_iter()
            .filter_map(|p| (p.norm_sqr() > 0f64).then(|| 1f64))
            .sum::<f64>()
            * self.resolution().powi(2)
    }
    /// Saves the pupil in an image file
    fn show_pupil(&self, path: Option<PathBuf>) -> ImageResult<()> {
        let n = (self.diameter() / self.resolution()).round() as u32 + 1;
        let mut img = RgbImage::new(n, n);
        img.pixels_mut()
            .zip(self.pupil(None).into_iter())
            .filter(|(_, pup)| pup.norm() > 0f64)
            .for_each(|(px, _)| *px = Rgb([255, 255, 255]));
        img.save(
            path.as_ref()
                .map(|p| p.as_ref())
                .unwrap_or(Path::new("telescope_pupil.png")),
        )
    }
}
