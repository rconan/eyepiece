use eyepiece::{Field, Observer, StarDistribution, Telescope};
use skyangle::SkyAngle;
fn main() -> anyhow::Result<()> {
    // let tel: Telescope = Default::default();
    let hubble = Telescope::new(2.4).obscuration(0.3).build();
    let jwst = Telescope::new(6.5).obscuration(0.74).build();
    let gmt = Telescope::new(25.5).obscuration(3.6).build();

    // let star =
    //     Star::new((SkyAngle::Radian(-1.5 * alpha), SkyAngle::Radian(3. * alpha))).magnitude(-1.);
    // StarDistribution::Lorentz {
    //     center: None,
    //     scale: (scale, scale),
    //     n_sample: 150,
    // },
    // StarDistribution::Uniform(fov, 150),
    // vec![Default::default(), star],

    let field_band = "K";
    let alpha = SkyAngle::MilliArcsec(10f64);
    println!("Resolution: {:.3}mas", alpha);
    let fov = SkyAngle::Arcsecond(1f64);
    let scale = SkyAngle::Radian(fov / 2.);
    let stars = StarDistribution::Globular {
        center: None,
        scale,
        n_sample: 150,
    };

    for (tel, tag) in [hubble, jwst, gmt]
        .into_iter()
        .zip(["hubble", "jwst", "gmt"])
    {
        let mut field = Field::new(alpha, fov, field_band, &stars, tel);
        field
            .observer
            .show_pupil(Some(format!("{tag}_pupil.png")))?;
        field.save(format!("{tag}_field{field_band}.tiff"))?;
    }
    Ok(())
}
