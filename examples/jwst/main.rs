use eyepiece::{Field, Jwst, Observer, PixelScale, Star};

fn main() -> anyhow::Result<()> {
    let jwst = Jwst::new();
    jwst.show_pupil(None)?;
    let mut field = Field::new(
        PixelScale::NyquistFraction(4),
        41,
        "V",
        Star::default(),
        jwst,
    );
    println!("{field}");
    field.save("image.png", None)?;
    Ok(())
}
