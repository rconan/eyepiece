use rand::Rng;
use rand_distr::{Cauchy, Distribution, Normal, Uniform};
use rand_seeder::{Seeder, SipRng};
use skyangle::SkyAngle;
use std::{env, time::Instant};

use super::{Objects, Star};

mod coordinate;
pub use coordinate::StarDistribution;
mod magnitude;
pub use magnitude::MagnitudeDistribution;
