use clap::{Parser, ValueEnum};
use eyepiece::{
    AdaptiveOptics, Builder, Field, FieldBuilder, Gmt, Hst, Jwst, MagnitudeDistribution, Objects,
    Observer, PixelScale, SaveOptions, SeeingBuilder, SeeingLimited, Star, StarDistribution,
    Telescope,
};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use skyangle::SkyAngle;
use std::{env, fs::File, thread};

// Natural Seeing
fn natural_seeing(cluster: &Objects, seeing_builder: &SeeingBuilder) -> anyhow::Result<()> {
    let style = "[{eta:>4}] {bar:40.cyan/blue} {pos:>5}/{len:5}";
    let bar = ProgressBar::new(cluster.len() as u64);
    bar.set_style(ProgressStyle::with_template(&format!("{}", style)).unwrap());

    // env::set_var("N_THREAD", "16");

    // natural seeing V & J bands 0.65arcsec / (1x1 arcmin & 20x20 arcsec)
    let tel = Telescope::new(8.).build();
    for (tag, fov) in vec!["wide", "narrow"]
        .into_iter()
        .zip(vec![SkyAngle::Arcminute(1f64), SkyAngle::Arcsecond(20.)].into_iter())
    {
        for band in ["V", "J"] {
            let mut seeing: Field<Telescope, SeeingLimited> = FieldBuilder::new(tel)
                .pixel_scale(SkyAngle::Arcsecond(0.3))
                .field_of_view(fov)
                .photometry(band)
                .seeing_limited(seeing_builder.clone())
                .objects(cluster)
                .exposure(3600.)
                .photon_noise()
                .build();
            println!("{seeing}");
            seeing.save(
                format!("{tag}_seeing_{band}.png"),
                SaveOptions::new().progress(bar.clone()),
            )?;
            bar.reset();
        }
    }
    Ok(())
}
// GLAO
fn glao(cluster: &Objects, seeing_builder: &SeeingBuilder) -> anyhow::Result<()> {
    let style = "[{eta:>4}] {bar:40.cyan/blue} {pos:>5}/{len:5}";
    let bar = ProgressBar::new(cluster.len() as u64);
    bar.set_style(ProgressStyle::with_template(&format!("{}", style)).unwrap());

    // env::set_var("N_THREAD", "16");

    // natural seeing V & J bands 0.65arcsec / (1x1 arcmin & 20x20 arcsec)
    let tel = Telescope::new(8.).build();
    for (tag, fov) in vec!["wide", "narrow"]
        .into_iter()
        .zip(vec![SkyAngle::Arcminute(1f64), SkyAngle::Arcsecond(20.)].into_iter())
    {
        for band in ["V", "J"] {
            let mut seeing: Field<Telescope, SeeingLimited> = FieldBuilder::new(tel)
                .pixel_scale(SkyAngle::Arcsecond(0.3))
                .field_of_view(fov)
                .photometry(band)
                .seeing_limited(seeing_builder.clone().glao(0.2))
                .objects(cluster)
                .exposure(3600.)
                .photon_noise()
                .build();
            println!("{seeing}");
            seeing.save(
                format!("{tag}_glao_{band}.png"),
                SaveOptions::new().progress(bar.clone()),
            )?;
            bar.reset();
        }
    }
    Ok(())
}

#[derive(Parser)]
struct Cli {
    #[arg(value_enum)]
    mode: Mode,
}
#[derive(ValueEnum, Clone)]
enum Mode {
    NaturalSeeing,
    GLAO,
    HAR,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // random generator seed
    env::set_var("SEED", "rebecca1");

    let n_sample = 5000;
    let coordinates = StarDistribution::GlobularBoxed {
        center: None,
        scale: SkyAngle::Arcsecond(30f64),
        n_sample,
        width: SkyAngle::Arcminute(1f64),
    };
    let magnitudes = MagnitudeDistribution::LogNormal(28., 0.7, 0.75);

    let cluster: Objects = (coordinates, magnitudes).into();
    serde_pickle::to_writer(
        &mut File::create("cluster.pkl")?,
        &cluster,
        Default::default(),
    )?;

    let gmt = Gmt::new();
    gmt.show_pupil(Some("gmt_pupil.png"))?;
    let gmt_exposure = 900.;

    let jwst = Jwst::new();
    jwst.show_pupil(Some("jwst_pupil.png"))?;
    let jwst_exposure = 45. * 60.;

    let hst = Hst::new();
    hst.show_pupil(Some("hst_pupil.png"))?;
    let hst_exposure = 4. * 3600.;

    let seeing_builder = SeeingBuilder::new(16e-2).zenith_angle(SkyAngle::Degree(30.));

    match cli.mode {
        Mode::NaturalSeeing => natural_seeing(&cluster, &seeing_builder)?,
        Mode::GLAO => glao(&cluster, &seeing_builder)?,
        Mode::HAR => {
            let style = "{msg:>10}: [{eta:>4}] {bar:40.cyan/blue} {pos:>5}/{len:5}";
            let bar = ProgressBar::new(cluster.len() as u64);
            bar.set_style(ProgressStyle::with_template(&format!("{}", style)).unwrap());
            // HST
            let mut hst_field: Field<Hst> = FieldBuilder::new(hst)
                .pixel_scale(PixelScale::Nyquist(1))
                .field_of_view(SkyAngle::Arcsecond(20.))
                .photometry("J")
                .objects(&cluster)
                .exposure(hst_exposure)
                .photon_noise()
                .build();
            println!("{hst_field}");
            // JWST
            let mut jwst_field: Field<Jwst> = FieldBuilder::new(jwst)
                .pixel_scale(PixelScale::Nyquist(1))
                .field_of_view(SkyAngle::Arcsecond(20.))
                .photometry("J")
                .objects(&cluster)
                .exposure(jwst_exposure)
                .photon_noise()
                .build();
            println!("{jwst_field}");
            // GMT NGAO
            let guide_star = Star::new((SkyAngle::Arcsecond(7.5), SkyAngle::Arcsecond(7.5)));
            let mut gmt_ngao_field: Field<Gmt, AdaptiveOptics> = FieldBuilder::new(gmt.clone())
                .pixel_scale(PixelScale::Nyquist(1))
                .field_of_view(SkyAngle::Arcsecond(20.))
                .photometry("J")
                .objects(&cluster)
                .seeing_limited(seeing_builder.clone().ngao(0.8, Some(guide_star)))
                .exposure(gmt_exposure)
                .photon_noise()
                .build();
            println!("{gmt_ngao_field}");
            // GMT LTAO
            let mut gmt_ltao_field: Field<Gmt, AdaptiveOptics> = FieldBuilder::new(gmt)
                .pixel_scale(PixelScale::Nyquist(1))
                .field_of_view(SkyAngle::Arcsecond(20.))
                .photometry("J")
                .objects(&cluster)
                .seeing_limited(seeing_builder.clone().ltao(0.5, SkyAngle::Arcsecond(30.)))
                .exposure(gmt_exposure)
                .photon_noise()
                .build();
            println!("{gmt_ltao_field}");
            // IMAGES
            bar.set_message("HST");
            hst_field.save(
                format!("hst_J.pkl"),
                SaveOptions::new().progress(bar.clone()),
            )?;
            bar.reset();
            bar.set_message("JWST");
            jwst_field.save(
                format!("jwst_J.pkl"),
                SaveOptions::new().progress(bar.clone()),
            )?;
            bar.finish_and_clear();

            let mbar = MultiProgress::new();
            let style = "{msg:>10}: [{eta:>4}] {bar:40.cyan/blue} {pos:>5}/{len:5}";

            thread::scope(|s| {
                s.spawn(|| {
                    let bar = mbar.add(ProgressBar::new(cluster.len() as u64));
                    bar.set_style(ProgressStyle::with_template(&format!("{}", style)).unwrap());
                    bar.set_message("GMT NGAO");
                    gmt_ngao_field
                        .save(format!("gmt_ngao_J.pkl"), SaveOptions::new().progress(bar))
                        .unwrap();
                });
                s.spawn(|| {
                    let bar = mbar.add(ProgressBar::new(cluster.len() as u64));
                    bar.set_style(ProgressStyle::with_template(&format!("{}", style)).unwrap());
                    bar.set_message("GMT LTAO");
                    gmt_ltao_field
                        .save(format!("gmt_ltao_J.pkl"), SaveOptions::new().progress(bar))
                        .unwrap();
                });
            });
        }
    }

    Ok(())
}
