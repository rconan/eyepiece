use eyepiece::{Field, Observer, Photometry, Star, StarDistribution, Telescope};
use skyangle::{Conversion, SkyAngle};
fn main() -> anyhow::Result<()> {
    // let tel: Telescope = Default::default();
    let hubble = Telescope::new(2.4).obscuration(0.3).build();
    let gmt = Telescope::new(6.5).obscuration(0.74).build();
    let gmt = Telescope::new(25.5).obscuration(3.6).build();

    let photometry: Photometry = "K".into();
    let alpha = 0.5 * photometry.wavelength / gmt.diameter;
    println!("Resolution: {:.3}arcsec", alpha.to_arcsec());

    let star =
        Star::new((SkyAngle::Radian(-1.5 * alpha), SkyAngle::Radian(3. * alpha))).magnitude(-1.);

    let field_band = "K";
    let fov = SkyAngle::Arcsecond(alpha.to_arcsec() * 101.);
    let scale = SkyAngle::Radian(fov / 10.);
    // hubble.show_pupil();
    let mut field = Field::new(
        SkyAngle::Arcsecond(alpha.to_arcsec()),
        fov,
        field_band,
        StarDistribution::Lorentz {
            center: None,
            scale: (scale, scale),
            n_sample: 150,
        },
        // StarDistribution::Uniform(fov, 150),
        // vec![Default::default(), star],
        gmt,
    );
    field.observer.show_pupil()?;
    field.save(format!("gmt_field{field_band}.tiff"))?;
    Ok(())
}
