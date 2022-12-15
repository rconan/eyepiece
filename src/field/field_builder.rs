use std::path::Path;

use image::{ImageResult, Rgb, RgbImage};
use indicatif::ProgressBar;

use crate::{Objects, Observer, Photometry, SeeingBuilder, Star};

use super::{Field, FieldOfView, ObservingMode, PixelScale};

#[derive(Clone, Debug)]
/// Field builder
///
/// # Example
/// ```
/// use eyepiece::{
///     Builder, FieldBuilder, Observer, PhotometricBands, PolychromaticField, SeeingBuilder, Telescope,
/// };
/// use skyangle::SkyAngle;
///
/// let tel = Telescope::new(8.).build();
/// let mut field: PolychromaticField<Telescope> = FieldBuilder::new(tel)
///     .pixel_scale(SkyAngle::Arcsecond(0.01))
///     .field_of_view(200)
///     .polychromatic(PhotometricBands::default().into_iter().collect())
///     .seeing_limited(SeeingBuilder::new(16e-2).zenith_angle(SkyAngle::Degree(30.)))
///     .build();
/// ```
pub struct FieldBuilder<T: Observer> {
    pixel_scale: PixelScale,
    field_of_view: FieldOfView,
    photometry: Vec<Photometry>,
    objects: Objects,
    exposure: f64,
    poisson_noise: bool,
    observer: T,
    seeing: Option<SeeingBuilder>,
    flux: Option<f64>,
}
impl<T: Observer> FieldBuilder<T> {
    /// Creates a default field
    ///
    /// The field is Nyquist sampled with 101x101 pixels and a single star in V band.
    /// The exposure is set to 1s and it is noiseless
    pub fn new(observer: T) -> Self {
        Self {
            pixel_scale: PixelScale::Nyquist(1),
            field_of_view: 101.into(),
            photometry: vec!["V".into()],
            objects: Star::default().into(),
            exposure: 1f64,
            poisson_noise: false,
            observer,
            seeing: None,
            flux: None,
        }
    }
    /// Sets the [pixel scale](PixelScale)
    pub fn pixel_scale<X: Into<PixelScale>>(self, pixel_scale: X) -> Self {
        Self {
            pixel_scale: pixel_scale.into(),
            ..self
        }
    }
    /// Sets the [field of view](FieldOfView)
    pub fn field_of_view<F: Into<FieldOfView>>(self, field_of_view: F) -> Self {
        Self {
            field_of_view: field_of_view.into(),
            ..self
        }
    }
    /// Sets the [photometry](Photometry)
    pub fn photometry<P: Into<Photometry>>(self, photometry: P) -> Self {
        Self {
            photometry: vec![photometry.into()],
            ..self
        }
    }
    /// Sets the [photometry](Photometry)
    pub fn polychromatic<P: Into<Photometry>>(self, photometry: Vec<P>) -> Self {
        Self {
            photometry: photometry.into_iter().map(|p| p.into()).collect(),
            ..self
        }
    }
    /// Sets the [objects](Objects)
    pub fn objects<O: Into<Objects>>(self, objects: O) -> Self {
        Self {
            objects: objects.into(),
            ..self
        }
    }
    /// Sets image total flux
    pub fn flux(self, flux: f64) -> Self {
        Self {
            flux: Some(flux),
            ..self
        }
    }
    /// Sets the exposure time in seconds
    pub fn exposure(self, exposure: f64) -> Self {
        Self { exposure, ..self }
    }

    /// Adds photon noise to the image
    pub fn photon_noise(self) -> Self {
        Self {
            poisson_noise: true,
            ..self
        }
    }
    /// Sets the [seeing](SeeingBuilder)
    pub fn seeing_limited(self, seeing_builder: SeeingBuilder) -> Self {
        Self {
            seeing: Some(seeing_builder),
            ..self
        }
    }
}

/// [FieldBuilder] to [Field] interface
pub trait Builder<F> {
    fn build(self) -> F;
}
impl<T: Observer> Builder<Field<T>> for FieldBuilder<T> {
    /// Creates a new field
    fn build(self) -> Field<T> {
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
            observing_mode: seeing.map_or_else(
                || ObservingMode::DiffractionLimited,
                |seeing| seeing.wavelength(photometry[0]).build(),
            ),
            flux,
        }
    }
}

/// Multi-field container
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
