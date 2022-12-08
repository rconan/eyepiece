use eyepiece::{Field, Observer, Photometry, Star, Telescope};
use skyangle::{Conversion, SkyAngle};

fn main() -> anyhow::Result<()> {
    // let tel: Telescope = Default::default();
    let tel = Telescope::new(2.4).obscuration(0.3).build();

    let photometry: Photometry = "V".into();
    let alpha = 0.5 * photometry.wavelength / tel.diameter;

    let star = Star::new((SkyAngle::Radian(0.5 * alpha), SkyAngle::Radian(0.5 * alpha)));

    let field_band = "V";
    // tel.show_pupil();
    let mut field = Field::new(
        SkyAngle::Arcsecond(dbg!(alpha).to_arcsec()),
        SkyAngle::Arcsecond(alpha.to_arcsec() * 20.),
        field_band,
        star,
        tel,
    );
    field.observer.show_pupil()?;
    field.save(format!("field{field_band}.png"))?;
    Ok(())
}
