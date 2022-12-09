use eyepiece::{Field, Jwst, Observer, Star};

fn main() -> anyhow::Result<()> {
    let jwst = Jwst::new();
    jwst.show_pupil(None)?;
    let mut field = Field::new(4, 41, "V", Star::default(), jwst);
    field.save("image.png")?;
    Ok(())
}
