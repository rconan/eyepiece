use std::thread::JoinHandle;

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
    handle: Option<JoinHandle<FieldImage>>,
}

impl Default for Program {
    fn default() -> Self {
        Self {
            observation: Default::default(),
            archive: None,
            state: State::Idle,
            handle: None,
        }
    }
}

impl Program {
    pub fn build(&mut self) {
        self.state = State::Building;
        self.handle = Some(self.observation.build());
    }
    pub fn is_built(&mut self) -> bool {
        match self.handle.take() {
            Some(h) => {
                if h.is_finished() {
                    let image = h.join().unwrap();
                    let pixels = image.pixels();
                    self.observation.image =
                        Some(ColorImage::from_rgb(image.resolution(), &pixels));
                    self.archive
                        .get_or_insert(Default::default())
                        .observations
                        .push(self.observation.clone());
                    self.state = State::Observing;
                    true
                } else {
                    self.handle = Some(h);
                    false
                }
            }
            None => {
                // self.build();
                false
            }
        }
    }
}
