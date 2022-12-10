use eyepiece::{Field, Gmt, Observer, PixelScale, Star};

fn main() -> anyhow::Result<()> {
    let gmt = Gmt::new();
    gmt.show_pupil(None)?;
    let mut field = Field::new(
        PixelScale::NyquistFraction(4),
        101,
        "V",
        Star::default(),
        gmt,
    );
    field.save("image.png", None)?;
    Ok(())
}
