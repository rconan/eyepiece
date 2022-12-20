use std::{env, path::Path};

use eyepiece::{Builder, FieldBuilder, PixelScale, SeeingBuilder, SeeingLimitedField, Telescope};
use skyangle::SkyAngle;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let path = Path::new(&env::var("CARGO_MANIFEST_DIR")?)
        .join("examples")
        .join("ngao");

    let tel = Telescope::new(8.).build();

    let aos: Vec<_> = (0..5)
        .map(|i| {
            SeeingBuilder::new(16e-2)
                .zenith_angle(SkyAngle::Degree(30.))
                .ngao(0.5 + 0.1 * i as f64, None)
        })
        .collect();

    let mut ao_fields: SeeingLimitedField<Telescope> = (
        FieldBuilder::new(tel)
            .pixel_scale(PixelScale::NyquistFraction(2))
            .field_of_view(SkyAngle::Arcsecond(1.5f64))
            .photometry("V")
            .lufn(f64::cbrt),
        aos,
    )
        .build();
    ao_fields.save(path.join("ngaos-image.png"), None)?;

    Ok(())
}
