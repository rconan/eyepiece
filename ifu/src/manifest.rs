use std::{f64::consts::PI, ops::Deref};

use eyepiece::{Hexagon, Observer};

/// Manifest IFU model
///
/// The IFU consist of 7 hexagons
#[derive(Debug, Clone)]
pub struct IFU(Vec<Hexagon>);
impl Deref for IFU {
    type Target = Vec<Hexagon>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl IFU {
    /// Creates a new IFU based on the size (flat-to-flat) of each hexagone
    pub fn new(f2f: f64) -> Self {
        let (s, c) = (PI / 6.).sin_cos();
        Self(vec![
            Hexagon::new((0f64, 0f64), f2f),
            Hexagon::new((0f64, f2f), f2f),
            Hexagon::new((0f64, -f2f), f2f),
            Hexagon::new((c * f2f, s * f2f), f2f),
            Hexagon::new((c * f2f, -s * f2f), f2f),
            Hexagon::new((-c * f2f, s * f2f), f2f),
            Hexagon::new((-c * f2f, -s * f2f), f2f),
        ])
    }
}
impl Observer for IFU {
    fn diameter(&self) -> f64 {
        todo!()
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
