use eyepiece::Observer;

#[derive(Debug, Clone)]
pub struct Slit {
    /// width
    w: f64,
    // length
    l: f64,
}
impl Slit {
    pub fn new(w: f64, l: f64) -> Self {
        Self { w, l }
    }
}

impl Observer for Slit {
    fn diameter(&self) -> f64 {
        todo!()
    }

    fn inside_pupil(&self, x: f64, y: f64) -> bool {
        let Self { w, l } = *self;
        if x.abs() < w * 0.5 && y.abs() < l * 0.5 {
            true
        } else {
            false
        }
    }
}
