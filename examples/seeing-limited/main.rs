use std::{env, path::Path};

use eyepiece::{
    Builder, FieldBuilder, Observer, PhotometricBands, PolychromaticField, SeeingBuilder, Telescope,
};
use skyangle::SkyAngle;

fn main() -> anyhow::Result<()> {
    let mut log = env_logger::Builder::new();
    // log.filter_level(log::LevelFilter::Debug).init();
    log.filter(Some("eyepiece::field"), log::LevelFilter::Debug)
        .init();

    let path = Path::new(&env::var("CARGO_MANIFEST_DIR")?)
        .join("examples")
        .join("seeing-limited");

    let tel = Telescope::new(8.).build();
    tel.show_pupil(None)?;
    let mut field: PolychromaticField<Telescope> = FieldBuilder::new(tel)
        .pixel_scale(SkyAngle::Arcsecond(0.01))
        .field_of_view(200)
        .polychromatic(PhotometricBands::default().into_iter().collect())
        .seeing_limited(SeeingBuilder::new(16e-2).zenith_angle(SkyAngle::Degree(30.)))
        .flux(1f64)
        .build();
    // println!("{field}");
    field.save(path.join("seeing-limited_VRIKHK.png"), None)?;
    Ok(())
}
