mod field;

pub use field::Field;
mod pixel_scale;
pub use pixel_scale::PixelScale;
mod field_of_view;
pub use field_of_view::FieldOfView;
mod field_builder;
pub use field_builder::{Builder, FieldBuilder, PolychromaticField};

/// Observing configurations
pub enum ObservingMode {
    /// Diffraction limited images
    DiffractionLimited,
    /// Seeing limited images
    SeeingLimited {
        fried_parameter: f64,
        outer_scale: f64,
    },
}
