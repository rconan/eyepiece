mod field;

pub use field::Field;
mod pixel_scale;
pub use pixel_scale::PixelScale;
mod field_of_view;
pub use field_of_view::FieldOfView;
mod field_builder;
pub use field_builder::FieldBuilder;
mod polychromatic;
pub use polychromatic::PolychromaticField;
mod seeing_limited;
pub use seeing_limited::SeeingLimitedField;
mod observing_mode;
pub use observing_mode::{Intensity, Observing};

/// [FieldBuilder] to [Field] interface
pub trait Builder<F> {
    fn build(self) -> F;
}

/// Diffraction limited observing mode
pub enum DiffractionLimited {}
/// Seeing limited observing mode
pub enum SeeingLimited {}
/// Adaptive optics observing mode
pub enum AdaptiveOptics {}
