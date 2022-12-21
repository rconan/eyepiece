use crate::Observer;

/// Generic circular telescope
///
/// # Example
/// ```
/// use eyepiece::Telescope;
/// let tel = Telescope::new(8.).build();
/// ```
#[derive(Debug, Clone, Copy)]
pub struct Telescope {
    /// Primary mirror diameter D (Nyquist sampling criteria: λ/2D)
    pub diameter: f64,
    obscuration: Option<f64>,
}

impl Default for Telescope {
    fn default() -> Self {
        Self {
            diameter: 1f64,
            obscuration: Default::default(),
        }
    }
}

/// Generic [Telescope] builder
pub struct TelescopeBuilder {
    diameter: f64,
    obscuration: Option<f64>,
}

impl Telescope {
    /// Creates a new telescope with the given `diameter`
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

    fn inside_pupil(&self, x: f64, y: f64) -> bool {
        let r_outer = self.diameter * 0.5;
        let r_inner = self.obscuration.unwrap_or_default();
        let r = x.hypot(y);
        r >= r_inner && r <= r_outer
    }
}
