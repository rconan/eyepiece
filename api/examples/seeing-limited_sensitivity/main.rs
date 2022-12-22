use std::{env, path::Path};

use eyepiece::{
    Builder, FieldBuilder, Gmt, PhotometricBands, PolychromaticField, SeeingBuilder, Star,
};
use skyangle::SkyAngle;

fn main() -> anyhow::Result<()> {
    let mut log = env_logger::builder();
    log.filter(Some("eyepiece::seeing"), log::LevelFilter::Debug)
        .init();

    let path = Path::new(&env::var("CARGO_MANIFEST_DIR")?)
        .join("examples")
        .join("seeing-limited_sensitivity");

    let mut field: PolychromaticField<Gmt> = FieldBuilder::new(Gmt::new())
        .pixel_scale(SkyAngle::Arcsecond(0.01))
        .field_of_view(200)
        .objects(Star::default().magnitude(30.))
        .exposure(5. * 3600.)
        .photon_noise()
        .polychromatic(PhotometricBands::default().into_iter().skip(2).collect())
        .seeing_limited(SeeingBuilder::new(16e-2).zenith_angle(SkyAngle::Degree(30.)))
        .build();
    field.save(path.join("seeing-limited_IJHK.png"), Default::default())?;
    Ok(())
}
