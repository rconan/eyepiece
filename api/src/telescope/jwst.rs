use std::ops::Deref;

use num_complex::Complex;

use crate::Observer;

/// Hexagonal pupil
#[derive(Debug, Clone)]
pub struct Hexagon {
    origin: (f64, f64),
    flat_to_flat: f64,
}
impl Hexagon {
    /// Creates a new hexagon
    ///
    /// The hexagon is set by its origin and flat-to-flat size
    pub fn new(origin: (f64, f64), flat_to_flat: f64) -> Self {
        Self {
            origin,
            flat_to_flat,
        }
    }
}

impl Observer for Hexagon {
    fn diameter(&self) -> f64 {
        let (cx, cy) = self.origin;
        2. * (cx.hypot(cy) + 0.5 * self.flat_to_flat / 30f64.to_radians().cos())
    }

    fn inside_pupil(&self, x: f64, y: f64) -> bool {
        let (cx, cy) = self.origin;
        let d = self.flat_to_flat * 0.5;
        for o in [-30f64.to_radians(), 30f64.to_radians()] {
            let (so, co) = o.sin_cos();
            let xp = (x - cx) * co + (y - cy) * so;
            if xp.abs() > d {
                return false;
            }
        }
        if (y - cy).abs() > d {
            return false;
        }
        true
    }
}

/// James Webb Telescope
///
/// <img src="https://raw.githubusercontent.com/rconan/eyepiece/main/examples/jwst/telescope_pupil.png" width="20%" alt="JWST pupil">
#[derive(Debug, Clone)]
pub struct Jwst(Vec<Hexagon>);
impl Deref for Jwst {
    type Target = Vec<Hexagon>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl Jwst {
    /// Creates a James Webb Telescope object
    pub fn new() -> Self {
        let f2f = 1.32;
        Self(
            (0..6)
                .map(|i| {
                    let o = (30. + i as f64 * 60.).to_radians();
                    let z = Complex::from_polar(f2f, o);
                    Hexagon::new((z.re, z.im), f2f)
                })
                .chain((0..6).map(|i| {
                    let o = (i as f64 * 60.).to_radians();
                    let z = Complex::from_polar(3. * f2f / 3f64.sqrt(), o);
                    Hexagon::new((z.re, z.im), f2f)
                }))
                .chain((0..6).map(|i| {
                    let o = (30. + i as f64 * 60.).to_radians();
                    let z = Complex::from_polar(2. * f2f, o);
                    Hexagon::new((z.re, z.im), f2f)
                }))
                .collect(),
        )
    }
}

impl Observer for Jwst {
    fn diameter(&self) -> f64 {
        6.6
    }

    fn inside_pupil(&self, x: f64, y: f64) -> bool {
        for hex in self.iter() {
            if hex.inside_pupil(x, y) {
                return true;
            }
        }
        false
    }
}
