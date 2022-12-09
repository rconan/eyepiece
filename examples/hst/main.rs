use eyepiece::{Field, Hst, Observer, Photometry, Star};
use skyangle::SkyAngle;

fn main() -> anyhow::Result<()> {
    let hst = Hst::new();
    hst.show_pupil(None)?;
    let photometry: Photometry = "V".into();
    let alpha = photometry.wavelength / hst.diameter() / 16.;
    let mut field = Field::new(
        SkyAngle::Radian(alpha),
        SkyAngle::Radian(alpha * 101.),
        photometry,
        Star::default(),
        hst,
    );
    field.save("image.png")?;
    Ok(())
}
