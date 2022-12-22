use super::{FieldOfView, PixelScale};
use crate::{Objects, Observer, Photometry, SeeingBuilder, Star};

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
    pub(super) pixel_scale: PixelScale,
    pub(super) field_of_view: FieldOfView,
    pub(super) photometry: Vec<Photometry>,
    pub(super) objects: Objects,
    pub(super) exposure: f64,
    pub(super) poisson_noise: bool,
    pub(super) observer: T,
    pub(super) seeing: Option<SeeingBuilder>,
    pub(super) flux: Option<f64>,
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
