use std::{env, path::Path};

use eyepiece::{FieldBuilder, Observer, SeeingBuilder, Star, Telescope};
use skyangle::SkyAngle;

fn main() -> anyhow::Result<()> {
    let mut log = env_logger::Builder::new();
    log.filter_level(log::LevelFilter::Debug).init();

    let path = Path::new(&env::var("CARGO_MANIFEST_DIR")?)
        .join("examples")
        .join("seeing-limited");

    let tel = Telescope::new(8.).build();
    tel.show_pupil(None)?;
    let mut field = FieldBuilder::new(SkyAngle::Arcsecond(0.01), 200, "J", Star::default(), tel)
        .seeing_limited(SeeingBuilder::new(16e-2).zenith_angle(SkyAngle::Degree(30.)))
        .build();
    println!("{field}");
    field.save(path.join("image.png"), None)?;
    Ok(())
}
