use std::{env, time::Instant};

use rand_distr::{Cauchy, Distribution, Uniform};
use rand_seeder::{Seeder, SipRng};
use skyangle::SkyAngle;

use super::{Objects, Star};

/// Spatial distribution of stars
///
/// The seed of the random generator can be set with the `SEED` environment variable
pub enum StarDistribution {
    /// Uniform distribution
    ///
    /// Distributes the stars uniformly in the range `[-h/2,h/2]` independently along the x and y directions with `n` sample total
    /// ```
    /// use skyangle::SkyAngle;
    /// use eyepiece::{Objects, StarDistribution};
    /// let h = SkyAngle::Arcminute(1f64);
    /// let n = 150;
    /// let stars: Objects = StarDistribution::Uniform(h, n).into();
    /// ```
    Uniform(SkyAngle<f64>, usize),
    /// Cartesian Lorentz distribution
    ///
    /// Distributes the stars according to a Lorentz probability distribution with the origin at `center`
    /// and independently along the x and y directions with `n` sample total
    /// ```
    /// use skyangle::SkyAngle;
    /// use eyepiece::{Objects, StarDistribution};
    /// let scale = SkyAngle::Arcminute(0.25f64);
    /// let n = 150;
    /// let stars: Objects = StarDistribution::Lorentz {
    ///     center: None,
    ///     scale: (scale, scale),
    ///     n_sample: n,
    /// }.into();
    /// ```
    Lorentz {
        center: Option<(SkyAngle<f64>, SkyAngle<f64>)>,
        scale: (SkyAngle<f64>, SkyAngle<f64>),
        n_sample: usize,
    },
    /// Polar Lorentz distribution
    ///
    /// Distributes the stars according to a Lorentz probability distribution with the origin at `center`
    /// along the radial direction with `n` sample total
    /// ```
    /// use skyangle::SkyAngle;
    /// use eyepiece::{Objects, StarDistribution};
    /// let scale = SkyAngle::Arcminute(0.25f64);
    /// let n = 150;
    /// let stars: Objects = StarDistribution::Globular {
    ///     center: None,
    ///     scale: scale,
    ///     n_sample: n,
    /// }.into();
    /// ```
    Globular {
        center: Option<(SkyAngle<f64>, SkyAngle<f64>)>,
        scale: SkyAngle<f64>,
        n_sample: usize,
    },
}
impl From<&StarDistribution> for Objects {
    fn from(star_dist: &StarDistribution) -> Self {
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
                    (0..*n_sample)
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
                    (0..*n_sample)
                        .map(|_| {
                            Star::new((
                                SkyAngle::Radian(lorentz_x.sample(&mut rng)),
                                SkyAngle::Radian(lorentz_y.sample(&mut rng)),
                            ))
                        })
                        .collect(),
                )
            }
            StarDistribution::Globular {
                center,
                scale,
                n_sample,
            } => {
                let (cx, cy) =
                    center.map_or((0f64, 0f64), |(x, y)| (x.to_radians(), y.to_radians()));
                let radius = Cauchy::new(0f64, scale.to_radians()).unwrap();
                let azimuth = Uniform::new(0f64, 2. * std::f64::consts::PI);
                let mut stars = vec![];
                for _ in 0..*n_sample {
                    let r = radius.sample(&mut rng);
                    let o = azimuth.sample(&mut rng);
                    let (so, co) = o.sin_cos();
                    let (x, y) = (cx + r * co, cy + r * so);
                    stars.push(Star::new((SkyAngle::Radian(x), SkyAngle::Radian(y))));
                }
                stars.into()
            }
        }
    }
}
impl From<StarDistribution> for Objects {
    fn from(star_dist: StarDistribution) -> Self {
        (&star_dist).into()
    }
}
