mod generic;
use std::ops::Deref;

pub use generic::{Telescope, TelescopeBuilder};
mod gmt;
pub use gmt::Gmt;
mod jwst;
pub use jwst::{Hexagon, Jwst};

use crate::Observer;

pub struct Hubble(Telescope);
impl Deref for Hubble {
    type Target = Telescope;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl Hubble {
    pub fn new() -> Self {
        Self(Telescope::new(2.4).obscuration(0.3).build())
    }
}
impl Observer for Hubble {
    fn diameter(&self) -> f64 {
        self.0.diameter()
    }

    fn inside_pupil(&self, x: f64, y: f64) -> bool {
        self.0.inside_pupil(x, y)
    }
}
