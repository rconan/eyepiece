use eyepiece::{Field, Star, Telescope};
use skyangle::SkyAngle;

fn main() -> anyhow::Result<()> {
    // let tel: Telescope = Default::default();
    let tel = Telescope::new(2.4).obscuration(0.3).build();

    // StarDistribution::Lorentz {
    //     center: None,
    //     scale: (scale, scale),
    //     n_sample: 150,
    // },
    // StarDistribution::Uniform(fov, 150),
    // vec![Default::default(), star],

    let field_band = "K";
    let alpha = SkyAngle::MilliArcsec(10f64);
    // let photometry: Photometry = "K".into();
    // let alpha = SkyAngle::Radian(0.5 * photometry.wavelength / tel.diameter);
    println!("Resolution: {:.3}arcsec", alpha.into_arcsec());
    let fov = SkyAngle::Arcsecond(1f64);
    /*     let scale = SkyAngle::Radian(fov / 4.);
    let stars = StarDistribution::Globular {
        center: None,
        scale,
        n_sample: 500,
    }; */

    /*     let star = Star::new((
        SkyAngle::Radian(-1.5 * alpha.to_radians()),
        SkyAngle::Radian(3. * alpha.to_radians()),
    ))
    .magnitude(-1.); */

    let mut field = Field::new(alpha, fov, field_band, Star::default(), tel);
    // field.observer.show_pupil::<Path>(None)?;
    field.save(format!("field{field_band}.png"))?;

    Ok(())
}
