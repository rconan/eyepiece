use eyepiece::{Builder, Field, FieldBuilder, Jwst, Observer, PixelScale};

fn main() -> anyhow::Result<()> {
    let jwst = Jwst::new();
    jwst.show_pupil(None)?;
    let mut field: Field<Jwst> = FieldBuilder::new(jwst)
        .pixel_scale(PixelScale::NyquistFraction(4))
        .field_of_view(41)
        .build();
    println!("{field}");
    field.save("image.png", Default::default())?;
    Ok(())
}
