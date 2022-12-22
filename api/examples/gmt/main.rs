use std::{env, path::Path};

use eyepiece::{Builder, Field, FieldBuilder, Gmt, Observer, PixelScale};

fn main() -> anyhow::Result<()> {
    let path = Path::new(&env::var("CARGO_MANIFEST_DIR")?)
        .join("examples")
        .join("gmt");

    let gmt = Gmt::new();
    gmt.show_pupil(None)?;
    let mut field: Field<Gmt> = FieldBuilder::new(gmt)
        .pixel_scale(PixelScale::NyquistFraction(4))
        .build();
    println!("{field}");
    field.save(path.join("image.png"), Default::default())?;
    Ok(())
}
