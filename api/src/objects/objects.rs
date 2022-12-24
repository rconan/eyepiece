use serde::Serialize;
use skyangle::{Conversion, SkyAngle};
use std::{
    fmt::Display,
    ops::{Deref, DerefMut},
};

type SkyCoordinates = (SkyAngle<f64>, SkyAngle<f64>);

/// A star object
#[derive(Debug, Clone, Copy, Serialize)]
pub struct Star {
    pub coordinates: SkyCoordinates,
    pub magnitude: f64,
}
impl Display for Star {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (x, y) = self.coordinates;
        write!(
            f,
            "star @({:.3},{:.3})arsec with {:.3} magnitude",
            x.into_arcsec().into_value(),
            y.into_arcsec().into_value(),
            self.magnitude
        )
    }
}
impl Star {
    pub fn distance(&self) -> f64 {
        let (x, y) = self.coordinates;
        x.to_radians().hypot(y.to_radians())
    }
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

#[derive(Debug, Clone, Serialize)]
/// A collection of stars
pub struct Objects(pub(super) Vec<Star>);
impl Objects {
    /// Returns the brightest star
    pub fn brightest(&self) -> Star {
        let mut objects = self.0.clone();
        objects.sort_by(|a, b| {
            let ma = a.magnitude;
            let mb = b.magnitude;
            ma.partial_cmp(&mb).unwrap()
        });
        objects[0].clone()
    }
    /// Returns the faintest star
    pub fn faintest(&self) -> Star {
        let mut objects = self.0.clone();
        objects.sort_by(|a, b| {
            let ma = a.magnitude;
            let mb = b.magnitude;
            mb.partial_cmp(&ma).unwrap()
        });
        objects[0].clone()
    }
    /// Returns the star the closed to the optical axis
    pub fn closest(&self) -> Star {
        let mut objects = self.0.clone();
        objects.sort_by(|a, b| a.distance().partial_cmp(&b.distance()).unwrap());
        objects[0].clone()
    }
    /// Returns the star the furthest to the optical axis
    pub fn furthest(&self) -> Star {
        let mut objects = self.0.clone();
        objects.sort_by(|a, b| b.distance().partial_cmp(&a.distance()).unwrap());
        objects[0].clone()
    }
}
impl Display for Objects {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{} stars", self.len())?;
        writeln!(
            f,
            " . magnitudes {:.3?}",
            (self.brightest().magnitude, self.faintest().magnitude)
        )?;
        writeln!(
            f,
            " . distances {:.3?}arcsec",
            (
                self.closest().distance().to_arcsec(),
                self.furthest().distance().to_arcsec()
            )
        )
    }
}
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
impl From<&Objects> for Objects {
    fn from(stars: &Objects) -> Self {
        Self(stars.iter().cloned().collect())
    }
}
