use crate::Observation;

#[derive(Debug, Default)]
pub struct Archive {
    pub observations: Vec<Observation>,
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
}

impl Default for Program {
    fn default() -> Self {
        Self {
            observation: Default::default(),
            archive: None,
            state: State::Idle,
        }
    }
}
