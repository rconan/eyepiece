use clap::Parser;
use eyepiece::{
    AdaptiveOptics, Builder, Field, FieldBuilder, Gmt, PixelScale, Saturation, SaveOptions,
    SeeingBuilder, SeeingLimited,
};
use indicatif::{ProgressBar, ProgressStyle};
use mast_eyepiece::Mast;
use skyangle::SkyAngle;

#[derive(Parser)]
struct Cli {
    #[arg(short, long)]
    target: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let cli = Cli::parse();

    let object_id = cli.target;
    let fov_arcmin = 20. / 60.;
    // let px_scale_arcsec = 0.1;
    let band = "J";

    let mast = Mast::new(band);
    let objects = mast
        .query_region(&object_id, SkyAngle::Arcminute(fov_arcmin / 2.))
        .await
        .unwrap();
    println!("{objects}");
    let n_star = objects.len();

    let stars: eyepiece::Objects = objects.into();
    println!("{stars}");
    let guide_star = stars.brightest();
    println!("Guide star: {:}", guide_star);

    let tel = Gmt::new();
    let mut field: Field<Gmt, AdaptiveOptics> = FieldBuilder::new(tel)
        .pixel_scale(PixelScale::Nyquist(2))
        .field_of_view(SkyAngle::Arcminute(fov_arcmin))
        .photometry(band)
        .objects(stars)
        .seeing_limited(
            SeeingBuilder::new(16e-2)
                .zenith_angle(SkyAngle::Degree(30.))
                // .glao(0.2)
                .ngao(0.5, Some(guide_star)),
        )
        // .photon_noise()
        .exposure(900.)
        .build();
    println!("{field}");

    let style = "[{eta:>4}] {bar:40.cyan/blue} {pos:>5}/{len:5}";
    let bar = ProgressBar::new(n_star as u64);
    bar.set_style(ProgressStyle::with_template(&format!("{}", style)).unwrap());
    field.save(
        format!(
            "{}_{}_{:.0}fov_{:}.tiff",
            object_id.replace(' ', ""),
            band,
            fov_arcmin * 60.,
            "nyquist_ngao"
        ),
        SaveOptions::new()
            .lufn(f64::cbrt)
            // .saturation(Saturation::LogSigma(3f64))
            .progress(bar),
    )?;
    Ok(())
}
