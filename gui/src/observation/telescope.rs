use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Based {
    Space(Telescope),
    Ground(Telescope),
}

impl Display for Based {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Based::Space(telescope) => telescope.fmt(f),
            Based::Ground(telescope) => telescope.fmt(f),
        }
    }
}

impl Default for Based {
    fn default() -> Self {
        Based::Ground(Telescope::default())
    }
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub enum Telescope {
    #[default]
    GMT,
    Telescope(eyepiece::Telescope),
    JWST,
    HST,
}

impl Display for Telescope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Telescope::GMT => write!(f, "GMT"),
            Telescope::Telescope(_) => write!(f, "Telescope"),
            Telescope::JWST => write!(f, "JWST"),
            Telescope::HST => write!(f, "HST"),
        }
    }
}
