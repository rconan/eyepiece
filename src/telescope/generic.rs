use crate::Observer;

/// Generic circular telescope
#[derive(Debug)]
pub struct Telescope {
    pub diameter: f64,
    obscuration: Option<f64>,
    resolution: f64,
}

impl Default for Telescope {
    fn default() -> Self {
        Self {
            diameter: 1f64,
            obscuration: Default::default(),
            resolution: 2.5e-2,
        }
    }
}

pub struct TelescopeBuilder {
    diameter: f64,
    obscuration: Option<f64>,
}

impl Telescope {
    pub fn new(diameter: f64) -> TelescopeBuilder {
        TelescopeBuilder {
            diameter,
            obscuration: None,
        }
    }
}

impl TelescopeBuilder {
    /// Sets the diameter of the telescope central obscuration
    pub fn obscuration(mut self, obscuration: f64) -> Self {
        self.obscuration = Some(obscuration);
        self
    }
    /// Build the telescope
    pub fn build(self) -> Telescope {
        Telescope {
            diameter: self.diameter,
            obscuration: self.obscuration,
            ..Default::default()
        }
    }
}

impl Observer for Telescope {
    fn diameter(&self) -> f64 {
        self.diameter
    }

    fn resolution(&self) -> f64 {
        self.resolution
    }

    fn inside_pupil(&self, x: f64, y: f64) -> bool {
        let r_outer = self.diameter * 0.5;
        let r_inner = self.obscuration.unwrap_or_default();
        let r = x.hypot(y);
        r >= r_inner && r <= r_outer
    }
}
