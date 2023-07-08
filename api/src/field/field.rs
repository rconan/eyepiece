use image::{ImageResult, Rgb, RgbImage};
use indicatif::{MultiProgress, ProgressBar};
use skyangle::Conversion;
use std::{fmt::Display, path::Path};

use super::{
    AdaptiveOptics, Builder, DiffractionLimited, FieldBuilder, FieldOfView, Intensity, Observing,
    PixelScale, SeeingLimited,
};
use crate::{Objects, Observer, ObservingModes, Photometry};

/// Observer field of regard
#[derive(Debug)]
pub struct Field<T, Mode = DiffractionLimited>
where
    T: Observer,
    Mode: ObservingModes,
{
    pub(super) pixel_scale: PixelScale,
    pub(super) field_of_view: FieldOfView,
    pub(super) photometry: Photometry,
    pub(super) objects: Objects,
    pub(super) exposure: f64,
    pub(super) poisson_noise: bool,
    pub(crate) observer: T,
    pub(super) observing_mode: Observing<Mode>,
    pub(super) flux: Option<f64>,
}

impl<T: Observer + Display, Mode: ObservingModes> Display for Field<T, Mode> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Field in {} band", self.photometry)?;
        writeln!(f, " . pixel scale: {:.3}mas", self.resolution().to_mas())?;
        writeln!(
            f,
            " . field-of-view: {:.3}arcsec",
            self.field_of_view.get(self).to_arcsec()
        )?;
        writeln!(f, " . {}", self.observer)?;
        write!(f, " . {}", self.observing_mode)?;
        // if let Some(seeing) = self.seeing() {
        //     writeln!(f, " . seeing: {:.3}arcsec", seeing.to_arcsec())?;
        // }
        let n_star: usize = self
            .objects
            .iter()
            .filter_map(|star| {
                let half_fov = self.field_of_view.get(self) * 0.5;
                let (x, y) = star.coordinates;
                if x.to_radians().abs() <= half_fov && y.to_radians().abs() <= half_fov {
                    Some(1)
                } else {
                    None
                }
            })
            .sum();
        writeln!(f, " . star #: {n_star}")?;
        let magnitude_max = self
            .objects
            .iter()
            .fold(f64::NEG_INFINITY, |a, s| a.max(s.magnitude));
        let magnitude_min = self
            .objects
            .iter()
            .fold(f64::INFINITY, |a, s| a.min(s.magnitude));
        writeln!(
            f,
            " . star magnitudes: [{magnitude_min:.1},{magnitude_max:.1}]"
        )?;
        writeln!(f, " . exposure time: {}s", self.exposure)
    }
}
impl<T: Observer> Builder<Field<T, DiffractionLimited>> for FieldBuilder<T> {
    /// Creates a new field
    fn build(self) -> Field<T, DiffractionLimited> {
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
        } = self;
        Field {
            pixel_scale,
            field_of_view,
            photometry: photometry[0],
            objects,
            exposure,
            poisson_noise,
            observer,
            observing_mode: Observing::diffraction_limited(),
            flux,
        }
    }
}
impl<T: Observer> Builder<Field<T, SeeingLimited>> for FieldBuilder<T> {
    /// Creates a new field
    fn build(self) -> Field<T, SeeingLimited> {
        let FieldBuilder {
            pixel_scale,
            field_of_view,
            photometry,
            objects,
            exposure,
            poisson_noise,
            observer,
            seeing,
            flux,
        } = self;
        Field {
            pixel_scale,
            field_of_view,
            photometry: photometry[0],
            objects,
            exposure,
            poisson_noise,
            observer,
            observing_mode: Observing::seeing_limited(
                seeing.map(|seeing| seeing.wavelength(photometry[0])),
            ),
            flux,
        }
    }
}
impl<T: Observer> Builder<Field<T, AdaptiveOptics>> for FieldBuilder<T> {
    /// Creates a new field
    fn build(self) -> Field<T, AdaptiveOptics> {
        let FieldBuilder {
            pixel_scale,
            field_of_view,
            photometry,
            objects,
            exposure,
            poisson_noise,
            observer,
            seeing,
            flux,
        } = self;

        Field {
            pixel_scale,
            field_of_view,
            photometry: photometry[0],
            objects,
            exposure,
            poisson_noise,
            observer,
            observing_mode: Observing::seeing_limited(
                seeing.map(|seeing| seeing.wavelength(photometry[0])),
            ),
            flux,
        }
    }
}
impl<T, Mode: ObservingModes> Field<T, Mode>
where
    T: Observer,
{
    /// Returns the field pixel resolution in radians
    pub fn resolution(&self) -> f64 {
        self.pixel_scale.get(&self.observer, &self.photometry)
    }
    /// Returns the size of the field-of-view in radians
    pub fn field_of_view(&self) -> f64 {
        self.field_of_view.get(self)
    }
}

impl<T, Mode> Field<T, Mode>
where
    T: Observer + Sync + Send,
    Mode: Send + ObservingModes,
    Observing<Mode>: Intensity,
{
    /// Computes image and save it to file
    pub fn save<P: AsRef<Path>>(&mut self, path: P, save_options: SaveOptions) -> ImageResult<()> {
        let mut intensity = self.intensity(save_options.bar);
        match path.as_ref().extension().and_then(|p| p.to_str()) {
            /*             Some("pkl") => {
                #[derive(serde::Serialize)]
                struct Data<'a, T, Mode>
                where
                    T: Observer + Sync + Send,
                    Mode: ObservingModes + Send,
                {
                    field: &'a Field<T, Mode>,
                    intensity: Vec<f64>,
                }
                let data = Data {
                    field: self,
                    intensity,
                };
                serde_pickle::to_writer(
                    &mut File::create(path.as_ref())?,
                    &data,
                    Default::default(),
                )
                .expect(&format!(
                    "failed to write field intensity into pickle file {:?}",
                    path.as_ref()
                ))
            } */
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

/// Intensity saturation setting
///
/// The default is [Saturation::Max], meaning that there is no saturation.
/// [Saturation::LogSigma]`(g)` sets the saturation threshold to `exp(m + g s)` where
/// `m` and `s` are the mean and standart deviation, respectively, of the log of the intensity.
#[derive(Default)]
pub enum Saturation {
    #[default]
    Max,
    LogSigma(f64),
}
impl Saturation {
    /// Returns the intensity saturation threshold
    pub fn threshold<'a, I: Iterator<Item = &'a f64>>(&self, data: I) -> f64 {
        match self {
            Saturation::Max => data.fold(f64::NEG_INFINITY, |m, &d| m.max(d)),
            Saturation::LogSigma(_) => {
                let ln_intensity: Vec<_> = data.filter(|&&i| i > 0f64).map(|&i| i.ln()).collect();
                let mean_ln_intensity =
                    ln_intensity.iter().cloned().sum::<f64>() / ln_intensity.len() as f64;
                let var_ln_intensity = ln_intensity
                    .iter()
                    .map(|ln_i| (ln_i - mean_ln_intensity).powi(2))
                    .sum::<f64>()
                    / ln_intensity.len() as f64;
                (mean_ln_intensity + 3. * var_ln_intensity.sqrt()).exp()
            }
        }
    }
}

/// Field [Field::save] options
#[derive(Default)]
pub struct SaveOptions {
    pub(super) bar: Option<ProgressBar>,
    pub(super) mbar: Option<MultiProgress>,
    pub(super) saturation: Saturation,
    pub(super) lufn: Option<fn(f64) -> f64>,
}
impl SaveOptions {
    /// Returns the default option
    ///
    /// The default options are:
    ///  * no saturation
    ///  * no progress bar
    ///  * no colormap look-up function
    pub fn new() -> Self {
        Default::default()
    }
    /// Sets the [progress bar](indicatif::ProgressBar)
    pub fn progress(self, bar: ProgressBar) -> Self {
        Self {
            bar: Some(bar),
            ..self
        }
    }
    /// Sets the [multi progress bar](indicatif::MultiProgress) holder
    pub fn saturation(self, saturation: Saturation) -> Self {
        Self { saturation, ..self }
    }
    /// Image colormap look-up function
    pub fn lufn(self, lufn: fn(f64) -> f64) -> Self {
        Self {
            lufn: Some(lufn),
            ..self
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::SeeingBuilder;

    use super::*;

    type Tel = crate::Hst;
    fn builder() -> FieldBuilder<Tel> {
        let tel = Tel::new();
        let seeing = SeeingBuilder::new(16e-2).ngao(0.75, None);
        FieldBuilder::new(tel).seeing_limited(seeing)
    }

    #[test]
    fn ser_diffraction() {
        let mut field: Field<Tel, DiffractionLimited> = builder().build();
        println!("{field}");
        field.dump("diffraction.pkl").unwrap();
    }

    #[test]
    fn ser_seeing() {
        let mut field: Field<Tel, SeeingLimited> = builder().build();
        println!("{field}");
        field.dump("seeing.pkl").unwrap();
    }

    #[test]
    fn ser_ao() {
        let mut field: Field<Tel, AdaptiveOptics> = builder().build();
        println!("{field}");
        field.dump("ao.pkl").unwrap();
    }
}
