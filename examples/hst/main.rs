use std::{env, path::Path};

use eyepiece::{Builder, Field, FieldBuilder, Hst, Observer, PixelScale};

fn main() -> anyhow::Result<()> {
    let mut log = env_logger::Builder::new();
    log.filter_level(log::LevelFilter::Debug).init();

    let path = Path::new(&env::var("CARGO_MANIFEST_DIR")?)
        .join("examples")
        .join("seeing-limited");

    let hst = Hst::new();
    hst.show_pupil(None)?;
    let mut field: Field<Hst> = FieldBuilder::new(hst)
        .pixel_scale(PixelScale::NyquistAt(2, "V".to_string()))
        .field_of_view(21)
        .photometry("K")
        .build();
    println!("{field}");
    field.save(path.join("image.png"), None)?;
    Ok(())
}
