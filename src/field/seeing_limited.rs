use std::path::Path;

use image::{ImageResult, Rgb, RgbImage};
use indicatif::ProgressBar;

use super::{Builder, Field, FieldBuilder, Observing, SeeingLimited};
use crate::{Observer, SeeingBuilder};

pub struct SeeingLimitedFields<T: Observer> {
    field_builder: FieldBuilder<T>,
    seeing_builders: Vec<SeeingBuilder>,
}
impl<T: Observer> Builder<SeeingLimitedFields<T>> for (FieldBuilder<T>, Vec<SeeingBuilder>) {
    fn build(self) -> SeeingLimitedFields<T> {
        SeeingLimitedFields {
            field_builder: self.0,
            seeing_builders: self.1,
        }
    }
}

impl<T: Observer> SeeingLimitedFields<T> {
    /// Return the # of monochromatic filters
    pub fn len(&self) -> usize {
        self.seeing_builders.len()
    }
    /// Computes image and save it to file
    pub fn save<P: AsRef<Path>>(&mut self, path: P, _bar: Option<ProgressBar>) -> ImageResult<()> {
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
            let mut field: Field<T, SeeingLimited> = Field {
                pixel_scale,
                field_of_view,
                photometry: photometry[0],
                objects,
                exposure,
                poisson_noise,
                observer,
                observing_mode: Observing::seeing_limited(Some(
                    seeing_builder.wavelength(photometry[0]),
                )),
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
