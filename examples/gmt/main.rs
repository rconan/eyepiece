use eyepiece::{Field, Gmt, Observer, Photometry, Star};
use skyangle::SkyAngle;

fn main() -> anyhow::Result<()> {
    let gmt = Gmt::new();
    gmt.show_pupil(None)?;
    let photometry: Photometry = "V".into();
    let alpha = photometry.wavelength / gmt.diameter() / 4.;
    let mut field = Field::new(
        SkyAngle::Radian(alpha),
        SkyAngle::Radian(alpha * 101.),
        photometry,
        Star::default(),
        gmt,
    );
    field.save("image.png")?;
    Ok(())
}
