use eyepiece::{Field, Gmt, Hst, Jwst, Observer, StarDistribution};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
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
    let scale = SkyAngle::Radian(fov / 2.);
    let n_star = 500;
    let stars = StarDistribution::Globular {
        center: None,
        scale,
        n_sample: n_star,
    };
    // let scale = SkyAngle::Radian(fov / 10.);
    // let stars = StarDistribution::Lorentz {
    //     center: None,
    //     scale: (scale, scale),
    //     n_sample: 150,
    // };

    let mbars = MultiProgress::new();
    let style = "[{eta:>4}] {bar:40.cyan/blue} {pos:>5}/{len:5}";
    let hst_bar = mbars.add(ProgressBar::new(n_star as u64));
    hst_bar.set_style(ProgressStyle::with_template(&format!("HST : {}", style)).unwrap());
    let jwst_bar = mbars.insert_after(&hst_bar, ProgressBar::new(n_star as u64));
    jwst_bar.set_style(ProgressStyle::with_template(&format!("JWST: {}", style)).unwrap());
    let gmt_bar = mbars.insert_after(&jwst_bar, ProgressBar::new(n_star as u64));
    gmt_bar.set_style(ProgressStyle::with_template(&format!("GMT : {}", style)).unwrap());

    thread::scope(|s| {
        s.spawn(|| {
            let mut field = Field::new(alpha, fov, field_band, &stars, Hst::new());
            field
                .observer
                .show_pupil(Some(Path::new(&format!("hubble_pupil.png")).into()))
                .unwrap();
            field
                .save(format!("hubble_field{field_band}.tiff"), Some(hst_bar))
                .unwrap();
        });
        s.spawn(|| {
            let mut field = Field::new(alpha, fov, field_band, &stars, Jwst::new());
            field
                .observer
                .show_pupil(Some(Path::new(&format!("jwst_pupil.png")).into()))
                .unwrap();
            field
                .save(format!("jwst_field{field_band}.tiff"), Some(jwst_bar))
                .unwrap();
        });
        s.spawn(|| {
            let mut field = Field::new(alpha, fov, field_band, &stars, Gmt::new());
            field
                .observer
                .show_pupil(Some(Path::new(&format!("gmt_pupil.png")).into()))
                .unwrap();
            field
                .save(format!("gmt_field{field_band}.tiff"), Some(gmt_bar))
                .unwrap();
        });
    });

    Ok(())
}
