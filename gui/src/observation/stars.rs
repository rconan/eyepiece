use std::fmt::Display;

use eyepiece::{MagnitudeDistribution, Objects, StarDistribution};
use serde::{Deserialize, Serialize};

use super::Camera;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum Distribution {
    Uniform(StarDistribution),
    Globular(StarDistribution),
}

impl Display for Distribution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Distribution::Uniform(StarDistribution::Uniform(_, n)) => write!(f, "Uniform{}", n),
            Distribution::Globular(StarDistribution::GlobularBoxed { n_sample, .. }) => {
                write!(f, "Globular{}", n_sample)
            }
            _ => write!(f, "Stars"),
        }
    }
}

impl Distribution {
    pub fn distribution(&self) -> &StarDistribution {
        match self {
            Distribution::Uniform(dist) => dist,
            Distribution::Globular(dist) => dist,
        }
    }
}

impl Default for Distribution {
    fn default() -> Self {
        Self::Uniform(StarDistribution::Uniform(
            Camera::default().field_of_view,
            1,
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum Magnitude {
    Normal(MagnitudeDistribution),
    LogNormal(MagnitudeDistribution),
}

impl Display for Magnitude {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Magnitude::Normal(_) => write!(f, "Normal"),
            Magnitude::LogNormal(_) => write!(f, "LogNormal"),
        }
    }
}

impl Magnitude {
    pub fn magnitude(&self) -> &MagnitudeDistribution {
        match self {
            Magnitude::Normal(mag) => mag,
            Magnitude::LogNormal(mag) => mag,
        }
    }
}

impl Default for Magnitude {
    fn default() -> Self {
        Self::Normal(MagnitudeDistribution::Normal(0f64, 1f64))
    }
}

#[derive(Debug, Default, PartialEq, Clone, Deserialize, Serialize)]
pub struct Stars {
    pub distribution: Distribution,
    pub magnitude: Magnitude,
    pub seed: bool,
}

impl Display for Stars {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.distribution, self.magnitude)
    }
}

impl From<&Stars> for Objects {
    fn from(stars: &Stars) -> Self {
        (
            stars.distribution.distribution(),
            stars.magnitude.magnitude(),
        )
            .into()
    }
}

impl Stars {
    pub fn update_fov(&mut self, fov: skyangle::SkyAngle<f64>) -> &mut Self {
        match &mut self.distribution {
            Distribution::Uniform(StarDistribution::Uniform(mean, _)) => {
                *mean = fov;
            }
            Distribution::Globular(StarDistribution::GlobularBoxed { width, .. }) => *width = fov,
            _ => unimplemented!(),
        }
        self
    }
    pub fn n_sample(&self) -> usize {
        match &self.distribution {
            Distribution::Uniform(StarDistribution::Uniform(_, n_sample)) => *n_sample,
            Distribution::Globular(StarDistribution::GlobularBoxed { n_sample, .. }) => *n_sample,
            _ => unimplemented!(),
        }
    }
}
