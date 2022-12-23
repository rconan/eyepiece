use eyepiece::{
    Builder, Field, FieldBuilder, Gmt, Saturation, SaveOptions, SeeingBuilder, SeeingLimited,
};
use indicatif::{ProgressBar, ProgressStyle};
use mast_eyepiece::Mast;
use skyangle::SkyAngle;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let object_id = "NGC3532";
    let fov_arcmin = 3.;
    let px_scale_arcsec = 0.1;
    let band = "J";

    let mast = Mast::new(band);
    let objects = mast
        .query_region(object_id, SkyAngle::Arcminute(fov_arcmin))
        .await
        .unwrap();
    println!("{objects}");
    let n_star = objects.len();

    let stars: eyepiece::Objects = objects.into();
    let guide_star = stars.brightest();

    let tel = Gmt::new();
    let mut field: Field<Gmt, SeeingLimited> = FieldBuilder::new(tel)
        .pixel_scale(SkyAngle::Arcsecond(px_scale_arcsec))
        .field_of_view(SkyAngle::Arcminute(fov_arcmin))
        .photometry(band)
        .objects(stars)
        .seeing_limited(
            SeeingBuilder::new(16e-2)
                .zenith_angle(SkyAngle::Degree(30.))
                .ngao(0.8, Some(guide_star)),
        )
        .photon_noise()
        .build();

    let style = "[{eta:>4}] {bar:40.cyan/blue} {pos:>5}/{len:5}";
    let bar = ProgressBar::new(n_star as u64);
    bar.set_style(ProgressStyle::with_template(&format!("{}", style)).unwrap());
    field.save(
        format!(
            "{}_{:.0}fov_{:.0}px.png",
            object_id,
            fov_arcmin * 60.,
            px_scale_arcsec * 1000.
        ),
        SaveOptions::new()
            .saturation(Saturation::LogSigma(3f64))
            .progress(bar),
    )?;
    Ok(())
}
