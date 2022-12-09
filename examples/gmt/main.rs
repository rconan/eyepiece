use eyepiece::{Field, Gmt, Observer, Star};

fn main() -> anyhow::Result<()> {
    let gmt = Gmt::new();
    gmt.show_pupil(None)?;
    let mut field = Field::new(4, 101, "V", Star::default(), gmt);
    field.save("image.png")?;
    Ok(())
}
