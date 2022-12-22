use std::{env, path::Path};

use eyepiece::{
    Builder, FieldBuilder, FieldOfView, Hst, PhotometricBands, PixelScale, PolychromaticField,
};

fn main() -> anyhow::Result<()> {
    let hst = Hst::new();
    let mut field: PolychromaticField<Hst> = FieldBuilder::new(hst)
        .pixel_scale(PixelScale::NyquistFractionAt(2, "V".to_string()))
        .field_of_view(FieldOfView::PixelScaleAt(61, "K".to_string()))
        .polychromatic(PhotometricBands::default().into_iter().collect())
        .flux(1f64)
        .build();
    let path = Path::new(&env::var("CARGO_MANIFEST_DIR")?)
        .join("examples")
        .join("polychromatic");
    field.save(path.join("image.png"), Default::default())?;
    Ok(())
}
