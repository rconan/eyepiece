use std::{env, path::Path};

use eyepiece::{Builder, Field, FieldBuilder, Jwst, Observer, PixelScale};

fn main() -> anyhow::Result<()> {
    let path = Path::new(&env::var("CARGO_MANIFEST_DIR")?)
        .join("examples")
        .join("jwst");

    let jwst = Jwst::new();
    jwst.show_pupil(Option::<&Path>::None)?;
    let mut field: Field<Jwst> = FieldBuilder::new(jwst)
        .pixel_scale(PixelScale::NyquistFraction(4))
        .field_of_view(41)
        .build();
    println!("{field}");
    field.save(path.join("image.png"), Default::default())?;
    Ok(())
}
