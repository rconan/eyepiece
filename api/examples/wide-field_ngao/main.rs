use std::{env, path::Path};

use eyepiece::{
    AdaptiveOptics, Builder, Field, FieldBuilder, PixelScale, SeeingBuilder, Star, Telescope,
};
use skyangle::SkyAngle;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let path = Path::new(&env::var("CARGO_MANIFEST_DIR")?)
        .join("examples")
        .join("wide-field_ngao");

    let tel = Telescope::new(8.).build();

    let mut asterism: Vec<Star> = vec![Default::default()];
    let r = 5f64;
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

    let ngao = SeeingBuilder::new(16e-2)
        .zenith_angle(SkyAngle::Degree(30.))
        .ngao(0.8, Some(asterism[1]));

    let mut ao_field: Field<Telescope, AdaptiveOptics> = FieldBuilder::new(tel)
        .pixel_scale(PixelScale::Nyquist(1))
        .field_of_view(SkyAngle::Arcsecond(12f64))
        .photometry("J")
        .objects(asterism)
        .seeing_limited(ngao)
        .lufn(f64::cbrt)
        .build();
    ao_field.save(path.join("wide-field_ngao-image.png"), None)?;

    Ok(())
}
