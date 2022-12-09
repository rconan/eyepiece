use eyepiece::{Field, Gmt, Hst, Jwst, Observer, StarDistribution};
use skyangle::SkyAngle;
use std::path::Path;
use std::thread;
fn main() -> anyhow::Result<()> {
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
    let alpha = SkyAngle::MilliArcsec(5f64);
    println!("Resolution: {:.3}mas", alpha);
    let fov = SkyAngle::Arcsecond(1f64);
    // let scale = SkyAngle::Radian(fov / 2.);
    // let stars = StarDistribution::Globular {
    //     center: None,
    //     scale,
    //     n_sample: 150,
    // };
    let scale = SkyAngle::Radian(fov / 10.);
    let stars = StarDistribution::Lorentz {
        center: None,
        scale: (scale, scale),
        n_sample: 150,
    };

    thread::scope(|s| {
        s.spawn(|| {
            let mut field = Field::new(alpha, fov, field_band, &stars, Hst::new());
            field
                .observer
                .show_pupil(Some(Path::new(&format!("hubble_pupil.png")).into()))
                .unwrap();
            field
                .save(format!("hubble_field{field_band}.tiff"))
                .unwrap();
        });
        s.spawn(|| {
            let mut field = Field::new(alpha, fov, field_band, &stars, Jwst::new());
            field
                .observer
                .show_pupil(Some(Path::new(&format!("jwst_pupil.png")).into()))
                .unwrap();
            field.save(format!("jwst_field{field_band}.tiff")).unwrap();
        });
        s.spawn(|| {
            let mut field = Field::new(alpha, fov, field_band, &stars, Gmt::new());
            field
                .observer
                .show_pupil(Some(Path::new(&format!("gmt_pupil.png")).into()))
                .unwrap();
            field.save(format!("gmt_field{field_band}.tiff")).unwrap();
        });
    });

    Ok(())
}
