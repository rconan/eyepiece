use eyepiece::{Field, Hst, Observer, PixelScale, Star};

fn main() -> anyhow::Result<()> {
    let hst = Hst::new();
    hst.show_pupil(None)?;
    let mut field = Field::new(
        PixelScale::NyquistAt(2, "V".to_string()),
        21,
        "K",
        Star::default(),
        hst,
    );
    println!("{field}");
    field.save("image.png", None)?;
    Ok(())
}
