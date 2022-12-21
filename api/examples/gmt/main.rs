use eyepiece::{Builder, Field, FieldBuilder, Gmt, Observer, PixelScale};

fn main() -> anyhow::Result<()> {
    let gmt = Gmt::new();
    gmt.show_pupil(None)?;
    let mut field: Field<Gmt> = FieldBuilder::new(gmt)
        .pixel_scale(PixelScale::NyquistFraction(4))
        .build();
    println!("{field}");
    field.save("image.png", None)?;
    Ok(())
}
