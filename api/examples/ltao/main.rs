use std::{env, path::Path};

use eyepiece::{
    AdaptiveOptics, Builder, Field, FieldBuilder, PixelScale, SeeingBuilder, Star, Telescope,
};
use indicatif::ProgressBar;
use skyangle::SkyAngle;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let path = Path::new(&env::var("CARGO_MANIFEST_DIR")?)
        .join("examples")
        .join("ltao");

    let tel = Telescope::new(8.).build();

    let mut asterism: Vec<Star> = vec![Default::default()];
    let r = 7.5f64;
    for i in 0..8 {
        let o = std::f64::consts::FRAC_PI_4 * i as f64;
        let (s, c) = o.sin_cos();
        let star = Star::new((SkyAngle::Arcsecond(r * c), SkyAngle::Arcsecond(r * s)));
        asterism.push(star);
    }
    let r = 2.5f64;
    for i in 0..6 {
        let o = std::f64::consts::FRAC_PI_3 * (i as f64 + 0.5);
        let (s, c) = o.sin_cos();
        let star = Star::new((SkyAngle::Arcsecond(r * c), SkyAngle::Arcsecond(r * s)));
        asterism.push(star);
    }
    let n_star = asterism.len();

    let ltao = SeeingBuilder::new(16e-2)
        .zenith_angle(SkyAngle::Degree(30.))
        .ltao(0.5, SkyAngle::Arcsecond(1.));

    let mut ao_field: Field<Telescope, AdaptiveOptics> = FieldBuilder::new(tel)
        .pixel_scale(PixelScale::Nyquist(1))
        .field_of_view(SkyAngle::Arcsecond(20f64))
        .photometry("I")
        .objects(asterism)
        .seeing_limited(ltao)
        .lufn(f64::cbrt)
        .build();
    ao_field.save(
        path.join("ltao-image.png"),
        Some(ProgressBar::new(n_star as u64)),
    )?;

    Ok(())
}
