use eyepiece::{
    Builder, Field, FieldBuilder, Saturation, SaveOptions, SeeingBuilder, SeeingLimited, Telescope,
};
use indicatif::{ProgressBar, ProgressStyle};
use mast_eyepiece::Mast;
use skyangle::SkyAngle;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let fov_arcmin = 3.;
    let mast = Mast::new("J");
    let objects = mast
        .query_region("NGC 6405", SkyAngle::Arcminute(fov_arcmin))
        .await
        .unwrap();
    println!("{objects}");
    let n_star = objects.len();

    let tel = Telescope::new(8.).build();
    let mut field: Field<Telescope, SeeingLimited> = FieldBuilder::new(tel)
        .pixel_scale(SkyAngle::Arcsecond(0.1))
        .field_of_view(SkyAngle::Arcminute(fov_arcmin))
        .photometry("J")
        .objects(objects)
        .seeing_limited(SeeingBuilder::new(16e-2).zenith_angle(SkyAngle::Degree(30.)))
        .photon_noise()
        .build();

    let style = "[{eta:>4}] {bar:40.cyan/blue} {pos:>5}/{len:5}";
    let bar = ProgressBar::new(n_star as u64);
    bar.set_style(ProgressStyle::with_template(&format!("{}", style)).unwrap());
    field.save(
        "image.png",
        SaveOptions::new().saturation(Saturation::LogSigma(3f64)), // .progress(bar),
    )?;
    Ok(())
}
