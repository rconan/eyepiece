//! # Eyepiece
//!
//! A crate to generate star fields as seen with different telescopes.
//!
//! ## Example
//! A single star in the center of a 21x21 pixels field as seen by the Hubble Space Telescope
//! with a pixel scale at half Nyquist (i.e. (λ/2D)/2) in V band
//! ```
//! use eyepiece::{Builder, Hst, Field, FieldBuilder, PixelScale, Star};
//!
//! let hst = Hst::new();
//! let mut field: Field<Hst> = FieldBuilder::new(hst)
//!     .pixel_scale(PixelScale::NyquistFraction(2))
//!     .field_of_view(21)
//!     .build();
//! ```
//! The field intensity map is computed and saved with
//! ```no_run
//! # use eyepiece::{Builder, Hst, Field, FieldBuilder, PixelScale, Star};
//! # let hst = Hst::new();
//! # let mut field: Field<Hst> = FieldBuilder::new(hst)
//! #    .pixel_scale(PixelScale::NyquistFraction(2))
//! #    .field_of_view(21)
//! #    .build();
//! field.save("field.png", Default::default()).unwrap();
//! ```
//!
//! More examples can be found [here](https://github.com/rconan/eyepiece/tree/main/examples)

use std::path::Path;

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
pub use field::*;
mod objects;
pub use objects::{MagnitudeDistribution, Objects, Star, StarDistribution};
mod seeing;
pub use seeing::SeeingBuilder;
mod adaptive_optics;
pub(crate) use adaptive_optics::AdaptiveOpticsCorrection;
mod bessel_knu;
mod optust;

/// Methods common to all telescopes
pub trait Observer: Clone {
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
    fn show_pupil<P: AsRef<Path>>(&self, path: Option<P>) -> ImageResult<()> {
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

pub(crate) fn atmosphere_transfer_function(r0: f64, big_l0: f64, d: f64, n_otf: usize) -> Vec<f64> {
    let mut atmosphere_otf: Vec<f64> = vec![0f64; n_otf * n_otf];
    for i in 0..n_otf {
        let q = i as i32 - n_otf as i32 / 2;
        let x = q as f64 * d;
        let ii = if q < 0i32 {
            (q + n_otf as i32) as usize
        } else {
            q as usize
        };
        for j in 0..n_otf {
            let q = j as i32 - n_otf as i32 / 2;
            let y = q as f64 * d;
            let jj = if q < 0i32 {
                (q + n_otf as i32) as usize
            } else {
                q as usize
            };
            let r = x.hypot(y);
            let kk = ii * n_otf + jj;
            atmosphere_otf[kk] = optust::phase::transfer_function(r, r0, big_l0);
        }
    }
    atmosphere_otf
}
