use serde::{Deserialize, Serialize};
use std::{
    env,
    fmt::Display,
    fs::File,
    io::{self, BufReader, BufWriter},
    path::Path,
    thread::{self, JoinHandle},
};

use epaint::ColorImage;
use eyepiece::{FieldBuilder, FieldImage, Gmt, Hst, Jwst, Observer};

mod camera;
pub use camera::Camera;
mod observing_mode;
use nanorand::{Rng, WyRand};
pub use observing_mode::ObservingMode;
mod telescope;
pub use telescope::{Based, Telescope};
pub mod stars;
pub use stars::Stars;

#[derive(Debug, thiserror::Error)]
pub enum ObservationError {
    #[error("io error")]
    IO(#[from] io::Error),
    #[error("bincode error")]
    Bincode(#[from] bincode::Error),
}

type Result<T> = std::result::Result<T, ObservationError>;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Observation {
    pub telescope: Based,
    pub mode: ObservingMode,
    pub stars: Stars,
    pub camera: Camera,
    pub image: Option<ColorImage>,
}

impl Display for Observation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}_{}_{}_{}",
            self.telescope, self.mode, self.camera, self.stars
        )
    }
}

pub trait Builder {
    fn camera(self, camera: &Camera) -> Self;
    fn observation(self, observation: &ObservingMode) -> FieldImage;
}

impl<T: Observer> Builder for FieldBuilder<T> {
    #[inline]
    fn camera(self, camera: &Camera) -> Self {
        camera.build(self)
    }
    #[inline]
    fn observation(self, observation: &ObservingMode) -> FieldImage {
        observation.build(self)
    }
}

impl Observation {
    pub fn build(&mut self) -> JoinHandle<FieldImage> {
        self.stars.update_fov(self.camera.field_of_view);
        let this = self.clone();
        thread::spawn(move || {
            if this.stars.seed {
                let mut rng = WyRand::new();
                env::set_var("SEED", format!("{}", rng.generate::<u64>()));
            }
            match this.telescope {
                Based::Ground(Telescope::GMT) => FieldBuilder::new(Gmt::new())
                    .camera(&this.camera)
                    .objects(&this.stars)
                    .observation(&this.mode),
                Based::Ground(Telescope::Telescope(telescope)) => FieldBuilder::new(telescope)
                    .camera(&this.camera)
                    .objects(&this.stars)
                    .observation(&this.mode),
                Based::Space(Telescope::JWST) => FieldBuilder::new(Jwst::new())
                    .camera(&this.camera)
                    .objects(&this.stars)
                    .observation(&this.mode),
                Based::Space(Telescope::HST) => FieldBuilder::new(Hst::new())
                    .camera(&this.camera)
                    .objects(&this.stars)
                    .observation(&this.mode),
                _ => unimplemented!(),
            }
            // image.pixels()
            // let pixels = image.pixels();
            // self.image = Some(ColorImage::from_rgb(image.resolution(), &pixels));
        })
    }
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let file = File::create(path.as_ref().with_extension("eye"))?;
        let mut buffer = BufWriter::new(file);
        bincode::serialize_into(&mut buffer, self)?;
        Ok(())
    }
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::open(path.as_ref().with_extension("eye"))?;
        let mut buffer = BufReader::new(file);
        Ok(bincode::deserialize_from(&mut buffer)?)
    }
}

/*
impl Config {
    pub fn make_field(&mut self) {
        log::info!("Make field!");
        let Self {
            tel,
            band,
            pixel_scale,
            fov,
            stars,
            ..
        } = self;

        // let field_band = "K";
        let alpha = SkyAngle::MilliArcsec(*pixel_scale);
        // println!("Resolution: {:.3}mas", alpha);
        let fov = SkyAngle::Arcsecond(*fov);

        let mut intensity = match tel {
            &mut TelescopeKind::Telescope {
                diameter,
                obscuration,
            } => {
                let mut field: Field<Telescope> = FieldBuilder::new(Telescope {
                    diameter,
                    obscuration: Some(obscuration),
                })
                .pixel_scale(alpha)
                .field_of_view(fov)
                .photometry(*band)
                .objects(dbg!(stars))
                .build();
                field.intensity(None)
            }
            TelescopeKind::HST => {
                let mut field: Field<Hst> = FieldBuilder::new(Hst::new())
                    .pixel_scale(alpha)
                    .field_of_view(fov)
                    .photometry(*band)
                    .objects(stars)
                    .build();
                field.intensity(None)
            }
            TelescopeKind::JWST => {
                let mut field: Field<Jwst> = FieldBuilder::new(Jwst::new())
                    .pixel_scale(alpha)
                    .field_of_view(fov)
                    .photometry(*band)
                    .objects(stars)
                    .build();
                field.intensity(None)
            }
            TelescopeKind::GMT => {
                let mut field: Field<Gmt> = FieldBuilder::new(Gmt::new())
                    .pixel_scale(alpha)
                    .field_of_view(fov)
                    .photometry(*band)
                    .objects(stars)
                    .build();
                field.intensity(None)
            }
        };

        // let field_band = "K";
        // let alpha = SkyAngle::MilliArcsec(10f64);
        // println!("Resolution: {:.3}mas", alpha);
        // let fov = SkyAngle::Arcsecond(1f64);
        // let scale = SkyAngle::Radian(fov / 4.);
        // let n_star = 2;
        // let stars = StarDistribution::Globular {
        //     center: None,
        //     scale,
        //     n_sample: n_star,
        // };
        //
        // let mut field: Field<Telescope> = FieldBuilder::new(Telescope {
        //     diameter: 8.,
        //     obscuration: Some(0.),
        // })
        // .pixel_scale(alpha)
        // .field_of_view(fov)
        // .photometry(*band)
        // .objects(&stars)
        // .build();

        // let mut intensity = field.intensity(None);
        let threshold = *intensity
            .iter()
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();
        intensity.iter_mut().for_each(|i| *i /= threshold);
        dbg!(intensity.len());
        let flux = intensity.iter().sum::<f64>();
        dbg!(flux);

        let lut = colorous::CUBEHELIX;
        let n_px = (intensity.len() as f64).sqrt() as usize;
        let pixels: Vec<_> = intensity
            .iter()
            .flat_map(|i| lut.eval_continuous(*i).into_array().to_vec())
            .collect();
        self.image = Some(ColorImage::from_rgb([n_px, n_px], &pixels));
        // let mut img = RgbImage::new(n_px as u32, n_px as u32);
        // img.pixels_mut().zip(&intensity).for_each(|(p, i)| {
        //     *p = Rgb(lut.eval_continuous(*i).into_array());
        // });
    }
}
 */
