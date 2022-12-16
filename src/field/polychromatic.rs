use std::path::Path;

use image::{ImageResult, Rgb, RgbImage};
use indicatif::ProgressBar;

use crate::Observer;

use super::{Builder, Field, FieldBuilder, ObservingMode};

/// Polychromatic field container
///
/// Each field has the same setup but for the photometry. See also [FieldBuilder].
pub struct PolychromaticField<T: Observer>(FieldBuilder<T>);
impl<T: Observer> Builder<PolychromaticField<T>> for FieldBuilder<T> {
    /// Creates a new field
    fn build(self) -> PolychromaticField<T> {
        PolychromaticField(self)
    }
}
impl<T: Observer> PolychromaticField<T> {
    /// Return the # of monochromatic filters
    pub fn len(&self) -> usize {
        self.0.photometry.len()
    }
    /// Computes image and save it to file
    pub fn save<P: AsRef<Path>>(&mut self, path: P, _bar: Option<ProgressBar>) -> ImageResult<()> {
        let mut intensities = vec![];
        for field_photometry in self.0.photometry.iter().cloned() {
            let FieldBuilder {
                pixel_scale,
                field_of_view,
                photometry: _,
                objects,
                exposure,
                poisson_noise,
                observer,
                seeing,
                flux,
            } = self.0.clone();
            let field = Field {
                pixel_scale,
                field_of_view,
                photometry: field_photometry,
                objects,
                exposure,
                poisson_noise,
                observer,
                observing_mode: seeing.map_or_else(
                    || ObservingMode::DiffractionLimited,
                    |seeing| seeing.wavelength(field_photometry).build(),
                ),
                flux,
            };
            intensities.push(field.intensity(None));
        }
        let max_intensity = intensities
            .iter()
            .map(|intensity| intensity.iter().cloned().fold(f64::NEG_INFINITY, f64::max))
            .fold(f64::NEG_INFINITY, f64::max);
        intensities
            .iter_mut()
            .for_each(|intensity| intensity.iter_mut().for_each(|i| *i /= max_intensity));

        let lut = colorous::CUBEHELIX;
        let n_px = (intensities[0].len() as f64).sqrt() as usize;
        let mut img = RgbImage::new((n_px * self.len()) as u32, n_px as u32);
        for mut px_row in img.rows_mut() {
            for intensity in intensities.iter_mut() {
                intensity.drain(..n_px).for_each(|i| {
                    **(px_row.next().as_mut().unwrap()) = Rgb(lut.eval_continuous(i).into_array());
                })
            }
        }
        img.save(path)
    }
}