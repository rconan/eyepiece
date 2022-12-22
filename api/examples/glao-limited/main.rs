use std::{env, path::Path};

use eyepiece::{Builder, FieldBuilder, SeeingBuilder, SeeingLimitedField, Telescope};
use skyangle::SkyAngle;

fn main() -> anyhow::Result<()> {
    let path = Path::new(&env::var("CARGO_MANIFEST_DIR")?)
        .join("examples")
        .join("glao-limited");

    let tel = Telescope::new(8.).build();

    let seeing: Vec<_> = (0..5)
        .map(|i| {
            SeeingBuilder::new(16e-2)
                .zenith_angle(SkyAngle::Degree(30.))
                .glao(0.1 * i as f64)
        })
        .collect();

    let mut field: SeeingLimitedField<Telescope> = (
        FieldBuilder::new(tel)
            .pixel_scale(SkyAngle::Arcsecond(0.01))
            .field_of_view(200),
        seeing,
    )
        .build();
    field.save(path.join("glao-limited.png"), Default::default())?;
    Ok(())
}
