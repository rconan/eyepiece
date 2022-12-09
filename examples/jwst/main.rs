use eyepiece::{Field, Jwst, Observer, Photometry, Star};
use skyangle::SkyAngle;

fn main() -> anyhow::Result<()> {
    let jwst = Jwst::new();
    jwst.show_pupil(None)?;
    let photometry: Photometry = "V".into();
    let alpha = photometry.wavelength / jwst.diameter() / 16.;
    let mut field = Field::new(
        SkyAngle::Radian(alpha),
        SkyAngle::Radian(alpha * 101.),
        photometry,
        Star::default(),
        jwst,
    );
    field.save("image.png")?;
    Ok(())
}
