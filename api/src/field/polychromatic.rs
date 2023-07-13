use std::path::Path;

use image::{ImageResult, Rgb, RgbImage};
use indicatif::{ProgressBar, ProgressStyle};

use crate::Observer;

use super::{
    Builder, DiffractionLimited, Field, FieldBuilder, Observing, SaveOptions, SeeingLimited,
};

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
impl<T> PolychromaticField<T>
where
    T: Observer + Sync + Send,
{
    /// Return the # of monochromatic filters
    pub fn len(&self) -> usize {
        self.0.photometry.len()
    }
    /// Computes image and save it to file
    pub fn save<P: AsRef<Path>>(&mut self, path: P, save_options: SaveOptions) -> ImageResult<()> {
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
            let bar = save_options
                .mbar
                .as_ref()
                .map(|mbar| mbar.add(ProgressBar::new(objects.len() as u64)));
            bar.as_ref().map(|bar| {
                bar.set_style(
                    ProgressStyle::with_template(&format!(
                        "{}",
                        "[{eta:>4}] {bar:40.cyan/blue} {pos:>5}/{len:5}"
                    ))
                    .unwrap(),
                )
            });
            let mut intensity = if seeing.is_none() {
                let mut field: Field<T, DiffractionLimited> = Field {
                    pixel_scale,
                    field_of_view,
                    photometry: field_photometry,
                    objects,
                    exposure,
                    poisson_noise,
                    observer,
                    observing_mode: Observing::diffraction_limited(),
                    flux,
                    intensity_sampling: None,
                };
                field.intensity(None)
            } else {
                let mut field: Field<T, SeeingLimited> = Field {
                    pixel_scale,
                    field_of_view,
                    photometry: field_photometry,
                    objects,
                    exposure,
                    poisson_noise,
                    observer,
                    observing_mode: Observing::seeing_limited(
                        seeing.map(|seeing| seeing.wavelength(field_photometry)),
                    ),
                    flux,
                    intensity_sampling: None,
                };
                field.intensity(bar)
            };
            if let Some(lufn) = save_options.lufn {
                intensity.iter_mut().for_each(|i| *i = lufn(*i));
            }
            intensities.push(intensity);
        }

        let threshold = save_options
            .saturation
            .threshold(intensities.iter().flatten());
        intensities
            .iter_mut()
            .for_each(|intensity| intensity.iter_mut().for_each(|i| *i /= threshold));

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
