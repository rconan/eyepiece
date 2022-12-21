use eyepiece::{Builder, Field, FieldBuilder, Gmt, Hst, Jwst, StarDistribution};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use skyangle::SkyAngle;
use std::{env, thread};

fn main() -> anyhow::Result<()> {
    env::set_var("SEED", "peekaboo42");

    let field_band = "K";
    let alpha = SkyAngle::MilliArcsec(5f64);
    println!("Resolution: {:.3}mas", alpha);
    let fov = SkyAngle::Arcsecond(1f64);
    let scale = SkyAngle::Radian(fov / 2.);
    let n_star = 1000;
    let stars = StarDistribution::Globular {
        center: None,
        scale,
        n_sample: n_star,
    };

    let field: Field<Hst> = FieldBuilder::new(Hst::new())
        .pixel_scale(alpha)
        .field_of_view(fov)
        .photometry(field_band)
        .objects(&stars)
        .build();
    println!("{field}");

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
            let mut field: Field<Hst> = FieldBuilder::new(Hst::new())
                .pixel_scale(alpha)
                .field_of_view(fov)
                .photometry(field_band)
                .objects(&stars)
                .build();
            field
                .save(format!("hst_field{field_band}.png"), Some(hst_bar))
                .unwrap();
        });
        s.spawn(|| {
            let mut field: Field<Jwst> = FieldBuilder::new(Jwst::new())
                .pixel_scale(alpha)
                .field_of_view(fov)
                .photometry(field_band)
                .objects(&stars)
                .build();
            field
                .save(format!("jwst_field{field_band}.png"), Some(jwst_bar))
                .unwrap();
        });
        s.spawn(|| {
            let mut field: Field<Gmt> = FieldBuilder::new(Gmt::new())
                .pixel_scale(alpha)
                .field_of_view(fov)
                .photometry(field_band)
                .objects(&stars)
                .build();
            field
                .save(format!("gmt_field{field_band}.png"), Some(gmt_bar))
                .unwrap();
        });
    });

    Ok(())
}
