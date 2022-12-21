use std::{env, time::Instant};

use rand::Rng;
use rand_distr::Normal;
use rand_seeder::{Seeder, SipRng};

use super::{Objects, StarDistribution};

/// Star magnitude distribution
pub enum MagnitudeDistribution {
    /// Normal distribution
    ///
    /// Distributes the star magnitudes according to a normal probability distribution specified with the mean and the standard deviation
    Normal(f64, f64),
    /// Log-normal distribution
    ///
    /// Distributes the star magnitudes according to a log-normal probability distribution specified with the offset, the mean and the standard deviation
    LogNormal(f64, f64, f64),
}
impl MagnitudeDistribution {
    pub fn get(&self, n_sample: usize) -> Vec<f64> {
        let mut rng: SipRng = if let Ok(seed) = env::var("SEED") {
            Seeder::from(seed).make_rng()
        } else {
            let now = Instant::now();
            Seeder::from(now).make_rng()
        };
        match self {
            MagnitudeDistribution::Normal(mean, std_dev) => {
                let dist = Normal::new(*mean, *std_dev).unwrap();
                (&mut rng).sample_iter(dist).take(n_sample).collect()
            }
            MagnitudeDistribution::LogNormal(m0, mu, sigma) => {
                let mean = (mu.powi(2) / (mu.powi(2) + sigma.powi(2)).sqrt()).ln();
                let sigma = (1. + (sigma / mu).powi(2)).ln().sqrt();
                let dist = Normal::new(mean, sigma).unwrap();
                (&mut rng)
                    .sample_iter(dist)
                    .take(n_sample)
                    .map(|m| m0 + m.exp())
                    .collect()
            }
        }
    }
}
impl From<(&StarDistribution, &MagnitudeDistribution)> for Objects {
    fn from(
        (coordinate_distribution, magnitude_distribution): (
            &StarDistribution,
            &MagnitudeDistribution,
        ),
    ) -> Self {
        let mut stars: Objects = coordinate_distribution.into();
        let magnitudes = magnitude_distribution.get(stars.len());
        stars
            .iter_mut()
            .zip(&magnitudes)
            .for_each(|(star, magnitude)| star.magnitude = *magnitude);
        stars
    }
}
impl From<(StarDistribution, MagnitudeDistribution)> for Objects {
    fn from(
        (coordinate_distribution, magnitude_distribution): (
            StarDistribution,
            MagnitudeDistribution,
        ),
    ) -> Self {
        (&coordinate_distribution, &magnitude_distribution).into()
    }
}
