use eyepiece::{Field, Hst, Observer, Star};

fn main() -> anyhow::Result<()> {
    let hst = Hst::new();
    hst.show_pupil(None)?;
    let mut field = Field::new(8, 101, "V", Star::default(), hst);
    field.save("image.png")?;
    Ok(())
}
