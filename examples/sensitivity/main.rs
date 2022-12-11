use eyepiece::{Field, Gmt, Hst, Jwst, MagnitudeDistribution, PixelScale, Star, StarDistribution};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use skyangle::SkyAngle;
use std::{env, thread};

fn main() -> anyhow::Result<()> {
    env::set_var("SEED", "ruby");

    let field_band = "J";
    let n_star = 1000;
    let gmt_res = PixelScale::Nyquist(1);
    let gmt_fov = 1000;
    let field = Field::new(
        gmt_res.clone(),
        gmt_fov,
        field_band,
        Star::default(),
        Gmt::new(),
    );
    println!("{field}");
    let alpha = SkyAngle::Radian(field.resolution());
    let fov = SkyAngle::Radian(field.field_of_view());
    let coordinates = StarDistribution::Uniform(fov, n_star);
    let magnitudes = MagnitudeDistribution::LogNormal(31., 0.7, 0.75);

    let mbars = MultiProgress::new();
    let style = "[{eta:>4}] {bar:40.cyan/blue} {pos:>5}/{len:5}";
    let hst_bar = mbars.add(ProgressBar::new(n_star as u64));
    hst_bar.set_style(ProgressStyle::with_template(&format!("HST : {}", style)).unwrap());
    let jwst_bar = mbars.insert_after(&hst_bar, ProgressBar::new(n_star as u64));
    jwst_bar.set_style(ProgressStyle::with_template(&format!("JWST: {}", style)).unwrap());
    let gmt_bar = mbars.insert_after(&jwst_bar, ProgressBar::new(n_star as u64));
    gmt_bar.set_style(ProgressStyle::with_template(&format!("GMT : {}", style)).unwrap());

    let exposure = 60. * 15.;

    let mut hst_field = Field::new(
        alpha,
        fov,
        field_band,
        (&coordinates, &magnitudes),
        Hst::new(),
    )
    .exposure(exposure);
    println!("{hst_field}");
    let mut jwst_field = Field::new(
        alpha,
        fov,
        field_band,
        (&coordinates, &magnitudes),
        Jwst::new(),
    )
    .exposure(exposure);
    println!("{jwst_field}");
    let mut gmt_field = Field::new(
        gmt_res,
        gmt_fov,
        field_band,
        (&coordinates, &magnitudes),
        Gmt::new(),
    )
    .exposure(exposure);
    println!("{gmt_field}");

    thread::scope(|s| {
        s.spawn(|| {
            hst_field
                .save(format!("hst_field{field_band}.tiff"), Some(hst_bar))
                .unwrap();
        });
        s.spawn(|| {
            jwst_field
                .save(format!("jwst_field{field_band}.tiff"), Some(jwst_bar))
                .unwrap();
        });
        s.spawn(|| {
            gmt_field
                .save(format!("gmt_field{field_band}.tiff"), Some(gmt_bar))
                .unwrap();
        });
    });

    Ok(())
}
