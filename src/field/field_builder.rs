use crate::{FieldOfView, Objects, Observer, Photometry, PixelScale};

use super::Field;

pub struct FieldBuilder<T: Observer> {
    field: Field<T>,
}
impl<T: Observer> FieldBuilder<T> {
    /// Creates a new field builder
    pub fn new<X, F, P, O>(
        resolution: X,
        field_of_view: F,
        photometric_band: P,
        objects: O,
        observer: T,
    ) -> Self
    where
        X: Into<PixelScale>,
        F: Into<FieldOfView>,
        P: Into<Photometry>,
        O: Into<Objects>,
    {
        Self {
            field: Field::new(
                resolution,
                field_of_view,
                photometric_band,
                objects,
                observer,
            ),
        }
    }
    /// Sets the exposure time in seconds
    pub fn exposure(mut self, value: f64) -> Self {
        self.field.exposure = value;
        self
    }
    /// Adds photon noise to the image
    pub fn photon_noise(mut self) -> Self {
        self.field.poisson_noise = true;
        self
    }
    /// Creates a new field
    pub fn build(self) -> Field<T> {
        self.field
    }
}
