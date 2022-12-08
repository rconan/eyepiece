use num_complex::Complex;

mod zpdft;
pub use zpdft::ZpDft;
mod telescope;
pub use telescope::Telescope;
mod photometry;
pub use photometry::Photometry;
mod field;
pub use field::Field;
mod objects;
pub use objects::{Objects, Star, StarDistribution};

pub trait Observer {
    fn diameter(&self) -> f64;
    fn resolution(&self) -> f64;
    fn pupil(&self, shift: Option<(f64, f64)>) -> Vec<Complex<f64>>;
    fn show_pupil<P: AsRef<std::path::Path>>(&self, path: Option<P>) -> image::ImageResult<()>;
}
