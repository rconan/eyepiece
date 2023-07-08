use std::{fmt::Display, ops::Deref};

mod generic;
pub use generic::{Telescope, TelescopeBuilder};
mod gmt;
pub use gmt::Gmt;
mod jwst;
pub use jwst::{Hexagon, Jwst};

use crate::Observer;

#[derive(Debug, Clone, Copy)]
/// Hubble Space Telescope
///
/// <img src="https://raw.githubusercontent.com/rconan/eyepiece/main/examples/hst/telescope_pupil.png" width="20%" alt="HST pupil">
pub struct Hst(Telescope);
impl Deref for Hst {
    type Target = Telescope;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl Hst {
    /// Creates a Hubble Space Telescope object
    pub fn new() -> Self {
        Self(Telescope::new(2.4).obscuration(0.3).build())
    }
}
impl Observer for Hst {
    fn diameter(&self) -> f64 {
        self.0.diameter()
    }

    fn inside_pupil(&self, x: f64, y: f64) -> bool {
        self.0.inside_pupil(x, y)
    }
}
impl Display for Hst {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "HST: {}m diameter ({}m diameter obscuration), {:.3}m^2 collection area",
            self.diameter,
            self.obscuration.unwrap(),
            self.area()
        )
    }
}
use serde::ser::{Serialize, Serializer};
impl Serialize for Hst {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        "HST".serialize(serializer)
    }
}
