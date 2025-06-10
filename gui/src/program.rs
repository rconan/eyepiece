use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};

use epaint::ColorImage;
use eyepiece::FieldImage;

use crate::Observation;

#[derive(Debug, Default)]
pub struct Archive {
    pub observations: Vec<Observation>,
}
impl From<Vec<Observation>> for Archive {
    fn from(value: Vec<Observation>) -> Self {
        Self {
            observations: value,
        }
    }
}

#[derive(Debug, Clone)]
pub enum State {
    Idle,
    Building,
    Observing,
}

#[derive(Debug)]
pub struct Program {
    pub observation: Observation,
    pub archive: Option<Archive>,
    pub state: State,
    field: Arc<Mutex<FieldImage>>,
    flag: Arc<AtomicBool>,
}

impl Default for Program {
    fn default() -> Self {
        Self {
            observation: Default::default(),
            archive: None,
            state: State::Idle,
            field: Default::default(),
            flag: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl Program {
    pub fn build(&mut self) {
        self.state = State::Building;
        self.observation
            .build(self.field.clone(), self.flag.clone());
    }
    pub fn is_built(&mut self) -> bool {
        if self.flag.load(Ordering::Relaxed) {
            let image = &*self.field.lock().unwrap();
            let pixels = image.pixels();
            self.observation.image = Some(ColorImage::from_rgb(image.resolution(), &pixels));
            self.archive
                .get_or_insert(Default::default())
                .observations
                .push(self.observation.clone());
            self.state = State::Observing;
            self.flag.store(false, Ordering::Relaxed);
            true
        } else {
            false
        }
    }
}
