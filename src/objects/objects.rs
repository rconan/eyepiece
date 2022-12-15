use skyangle::SkyAngle;
use std::ops::{Deref, DerefMut};

type SkyCoordinates = (SkyAngle<f64>, SkyAngle<f64>);

/// A star object
#[derive(Debug, Clone, Copy)]
pub struct Star {
    pub coordinates: SkyCoordinates,
    pub magnitude: f64,
}
impl Star {
    pub fn inside_box(&self, width: f64) -> bool {
        let (x, y) = self.coordinates;
        let h = width / 2.;
        x.to_radians().abs() <= h && y.to_radians().abs() < h
    }
}
impl Default for Star {
    fn default() -> Self {
        Self {
            coordinates: (SkyAngle::Arcsecond(0f64), SkyAngle::Arcsecond(0f64)),
            magnitude: Default::default(),
        }
    }
}
impl Star {
    /// Creates a new `Star` object
    pub fn new(coordinates: SkyCoordinates) -> Self {
        Self {
            coordinates,
            ..Default::default()
        }
    }
    /// Sets the star magnitude
    pub fn magnitude(mut self, magnitude: f64) -> Self {
        self.magnitude = magnitude;
        self
    }
}

#[derive(Debug, Clone)]
/// A collection of stars
pub struct Objects(pub(super) Vec<Star>);
impl Deref for Objects {
    type Target = Vec<Star>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for Objects {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl Default for Objects {
    fn default() -> Self {
        Self(Default::default())
    }
}
/// A collection from a single star
impl From<Star> for Objects {
    fn from(star: Star) -> Self {
        Self(vec![star])
    }
}
/// A collection from a set of stars
impl From<Vec<Star>> for Objects {
    fn from(stars: Vec<Star>) -> Self {
        Self(stars)
    }
}
