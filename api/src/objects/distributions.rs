use rand::Rng;
use rand_distr::{Cauchy, Distribution, Normal, Uniform};
use rand_seeder::{Seeder, SipRng};
use skyangle::SkyAngle;
use std::env;
#[cfg(not(target_arch = "wasm32"))]
use std::time::SystemTime;
#[cfg(target_arch = "wasm32")]
use wasm_timer::SystemTime;

use super::{Objects, Star};

mod coordinate;
pub use coordinate::StarDistribution;
mod magnitude;
pub use magnitude::MagnitudeDistribution;
