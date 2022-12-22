use eyepiece::{
    Builder, Field, FieldBuilder, Gmt, Hst, Jwst, MagnitudeDistribution, PixelScale, SaveOptions,
    Star, StarDistribution,
};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use skyangle::SkyAngle;
use std::{env, thread};

fn main() -> anyhow::Result<()> {
    env::set_var("SEED", "ruby");

    let field_band = "J";
    let n_star = 1000;
    let gmt_res = PixelScale::Nyquist(1);
    let gmt_fov = 1000;
    let field: Field<Gmt> = FieldBuilder::new(Gmt::new())
        .pixel_scale(gmt_res.clone())
        .field_of_view(gmt_fov)
        .photometry(field_band)
        .objects(Star::default())
        .build();
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

    let mut hst_field: Field<Hst> = FieldBuilder::new(Hst::new())
        .pixel_scale(alpha)
        .field_of_view(fov)
        .photometry(field_band)
        .objects((&coordinates, &magnitudes))
        .exposure(exposure)
        .photon_noise()
        .build();
    println!("{hst_field}");
    let mut jwst_field: Field<Jwst> = FieldBuilder::new(Jwst::new())
        .pixel_scale(alpha)
        .field_of_view(fov)
        .photometry(field_band)
        .objects((&coordinates, &magnitudes))
        .exposure(exposure)
        .photon_noise()
        .build();
    println!("{jwst_field}");
    let mut gmt_field: Field<Gmt> = FieldBuilder::new(Gmt::new())
        .pixel_scale(gmt_res)
        .field_of_view(gmt_fov)
        .photometry(field_band)
        .objects((&coordinates, &magnitudes))
        .exposure(exposure)
        .photon_noise()
        .build();
    println!("{gmt_field}");

    thread::scope(|s| {
        s.spawn(|| {
            hst_field
                .save(
                    format!("hst_field{field_band}.tiff"),
                    SaveOptions::new().progress(hst_bar),
                )
                .unwrap();
        });
        s.spawn(|| {
            jwst_field
                .save(
                    format!("jwst_field{field_band}.tiff"),
                    SaveOptions::new().progress(jwst_bar),
                )
                .unwrap();
        });
        s.spawn(|| {
            gmt_field
                .save(
                    format!("gmt_field{field_band}.tiff"),
                    SaveOptions::new().progress(gmt_bar),
                )
                .unwrap();
        });
    });

    Ok(())
}
