use eyepiece::{Field, Hst, Observer, PixelScale, Star};

fn main() -> anyhow::Result<()> {
    let hst = Hst::new();
    hst.show_pupil(None)?;
    let mut field = Field::new(
        PixelScale::NyquistFraction(8),
        101,
        "V",
        Star::default(),
        hst,
    );
    field.save("image.png", None)?;
    Ok(())
}
