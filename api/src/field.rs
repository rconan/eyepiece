mod field;
pub use field::{Field, Saturation, SaveOptions};
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
use serde::Serialize;
mod intensity;
mod serpkl;

/// [FieldBuilder] to [Field] interface
pub trait Builder<F> {
    fn build(self) -> F;
}

/// Diffraction limited observing mode
#[derive(Debug, Serialize)]
pub enum DiffractionLimited {}
/// Seeing limited observing mode
#[derive(Debug, Serialize)]
pub enum SeeingLimited {}
/// Adaptive optics observing mode
#[derive(Debug, Serialize)]
pub enum AdaptiveOptics {}

/// Trait defining the observing modes
pub trait ObservingModes: Serialize {}
impl ObservingModes for DiffractionLimited {}
impl ObservingModes for SeeingLimited {}
impl ObservingModes for AdaptiveOptics {}

/// Trait limiting the observing modes to the seeing limited modes (including Adaptive Optics)
pub trait DiffractionModes: ObservingModes {}
impl DiffractionModes for DiffractionLimited {}

/// Trait limiting the observing modes to the seeing limited modes (including Adaptive Optics)
pub trait SeeingModes: ObservingModes {}
impl SeeingModes for SeeingLimited {}
impl SeeingModes for AdaptiveOptics {}
