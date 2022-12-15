use std::{env, path::Path};

use eyepiece::{Field, Hst, Observer, PixelScale, Star};

fn main() -> anyhow::Result<()> {
    let mut log = env_logger::Builder::new();
    log.filter_level(log::LevelFilter::Debug).init();

    let path = Path::new(&env::var("CARGO_MANIFEST_DIR")?)
        .join("examples")
        .join("seeing-limited");

    let hst = Hst::new();
    hst.show_pupil(None)?;
    let mut field = Field::new(
        PixelScale::NyquistAt(2, "V".to_string()),
        21,
        "K",
        Star::default(),
        hst,
    );
    println!("{field}");
    field.save(path.join("image.png"), None)?;
    Ok(())
}
