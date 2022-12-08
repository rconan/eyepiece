use rand::distributions::{Distribution, Uniform};
use rand_distr::Cauchy;
use rand_seeder::{Seeder, SipRng};
use skyangle::SkyAngle;
use std::{env, ops::Deref, time::Instant};

type SkyCoordinates = (SkyAngle<f64>, SkyAngle<f64>);

#[derive(Debug)]
pub struct Star {
    pub coordinates: SkyCoordinates,
    pub magnitude: f64,
}
impl Default for Star {
    fn default() -> Self {
        Self {
            coordinates: (SkyAngle::Arcsecond(0f64), SkyAngle::Arcsecond(0f64)),
            magnitude: Default::default(),
        }
    }
}
impl Star {
    pub fn new(coordinates: SkyCoordinates) -> Self {
        Self {
            coordinates,
            ..Default::default()
        }
    }
    pub fn magnitude(mut self, magnitude: f64) -> Self {
        self.magnitude = magnitude;
        self
    }
}
pub struct Objects(Vec<Star>);
impl Deref for Objects {
    type Target = Vec<Star>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl Default for Objects {
    fn default() -> Self {
        Self(Default::default())
    }
}
impl From<Star> for Objects {
    fn from(star: Star) -> Self {
        Self(vec![star])
    }
}
impl From<Vec<Star>> for Objects {
    fn from(stars: Vec<Star>) -> Self {
        Self(stars)
    }
}
pub enum StarDistribution {
    Uniform(SkyAngle<f64>, usize),
    Lorentz {
        center: Option<(SkyAngle<f64>, SkyAngle<f64>)>,
        scale: (SkyAngle<f64>, SkyAngle<f64>),
        n_sample: usize,
    },
}
impl From<StarDistribution> for Objects {
    fn from(star_dist: StarDistribution) -> Self {
        let mut rng: SipRng = if let Ok(seed) = env::var("SEED") {
            Seeder::from(seed).make_rng()
        } else {
            let now = Instant::now();
            Seeder::from(now).make_rng()
        };
        // let mut rng = rand::thread_rng();
        // let mut rng: SipRng = Seeder::from("stripy zebra").make_rng();
        match star_dist {
            StarDistribution::Uniform(fov, n_sample) => {
                let h = 0.5 * fov.to_radians();
                let dist = Uniform::new_inclusive(-h, h);
                Self(
                    (0..n_sample)
                        .map(|_| {
                            Star::new((
                                SkyAngle::Radian(dist.sample(&mut rng)),
                                SkyAngle::Radian(dist.sample(&mut rng)),
                            ))
                        })
                        .collect(),
                )
            }
            StarDistribution::Lorentz {
                center,
                scale,
                n_sample,
            } => {
                let (cx, cy) = center.unwrap_or((SkyAngle::Radian(0f64), SkyAngle::Radian(0f64)));
                let (sx, sy) = scale;
                let lorentz_x = Cauchy::new(cx.to_radians(), sx.to_radians()).unwrap();
                let lorentz_y = Cauchy::new(cy.to_radians(), sy.to_radians()).unwrap();
                Self(
                    (0..n_sample)
                        .map(|_| {
                            Star::new((
                                SkyAngle::Radian(lorentz_x.sample(&mut rng)),
                                SkyAngle::Radian(lorentz_y.sample(&mut rng)),
                            ))
                        })
                        .collect(),
                )
            }
        }
    }
}
