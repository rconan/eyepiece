use std::path::Path;

use image::{ImageResult, Rgb, RgbImage};
use indicatif::{ProgressBar, ProgressStyle};

use super::{Builder, Field, FieldBuilder, Observing, SeeingLimited};
use crate::{AdaptiveOptics, Observer, SaveOptions, SeeingBuilder};

/**
Seeing limited fields container

A set of seeing limited fields with the same setup but for the seeing conditions.
## Example
```
use eyepiece::{Builder, FieldBuilder, SeeingBuilder, SeeingLimitedField, Telescope};
use skyangle::SkyAngle;

let tel = Telescope::new(8.).build();

let seeing: Vec<_> = (1..=5)
    .map(|i| {
        SeeingBuilder::new(16e-2)
            .outer_scale(5f64 * i as f64)
            .zenith_angle(SkyAngle::Degree(30.))
    })
    .collect();
let mut field: SeeingLimitedField<Telescope> = (
    FieldBuilder::new(tel)
        .pixel_scale(SkyAngle::Arcsecond(0.01))
        .field_of_view(200),
    seeing,
)
    .build();
field.save("seeing_vs_outer-scale.png", Default::default()).unwrap();
```
*/
pub struct SeeingLimitedField<T: Observer> {
    field_builder: FieldBuilder<T>,
    seeing_builders: Vec<SeeingBuilder>,
}
impl<T: Observer> Builder<SeeingLimitedField<T>> for (FieldBuilder<T>, Vec<SeeingBuilder>) {
    /// Creates a set of seeing limited fields from a [FieldBuilder] and a [Vec] of [SeeingBuilder]
    fn build(self) -> SeeingLimitedField<T> {
        SeeingLimitedField {
            field_builder: self.0,
            seeing_builders: self.1,
        }
    }
}
impl<T: Observer> Builder<SeeingLimitedField<T>> for (FieldBuilder<T>, SeeingBuilder) {
    /// Creates a set of seeing limited fields from a [FieldBuilder] and a [SeeingBuilder]
    fn build(self) -> SeeingLimitedField<T> {
        <(FieldBuilder<T>, Vec<SeeingBuilder>) as Builder<SeeingLimitedField<T>>>::build((
            self.0,
            vec![self.1],
        ))
    }
}
impl<T: Observer> SeeingLimitedField<T> {
    /// Return the # of seeing conditions
    pub fn len(&self) -> usize {
        self.seeing_builders.len()
    }
    /// Computes image and save it to file
    pub fn save<P: AsRef<Path>>(&mut self, path: P, save_options: SaveOptions) -> ImageResult<()> {
        let mut intensities = vec![];
        for seeing_builder in self.seeing_builders.iter() {
            let FieldBuilder {
                pixel_scale,
                field_of_view,
                photometry,
                objects,
                exposure,
                poisson_noise,
                observer,
                seeing: _,
                flux,
            } = self.field_builder.clone();
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
            let mut intensity = if seeing_builder.adaptive_optics.is_none() {
                let mut field: Field<T, SeeingLimited> = Field {
                    pixel_scale,
                    field_of_view,
                    photometry: photometry[0],
                    objects,
                    exposure,
                    poisson_noise,
                    observer,
                    observing_mode: Observing::seeing_limited(Some(
                        seeing_builder.clone().wavelength(photometry[0]),
                    )),
                    flux,
                };
                field.intensity(bar)
            } else {
                let mut field: Field<T, AdaptiveOptics> = Field {
                    pixel_scale,
                    field_of_view,
                    photometry: photometry[0],
                    objects,
                    exposure,
                    poisson_noise,
                    observer,
                    observing_mode: Observing::seeing_limited(Some(
                        seeing_builder.clone().wavelength(photometry[0]),
                    )),
                    flux,
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
