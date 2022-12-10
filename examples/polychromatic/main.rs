use eyepiece::{Field, FieldOfView, Hst, PhotometricBands, PixelScale, Star};

fn main() -> anyhow::Result<()> {
    let hst = Hst::new();
    for (i, band) in PhotometricBands::default().into_iter().enumerate() {
        let mut field = Field::new(
            PixelScale::NyquistFractionAt(2, "V".to_string()),
            FieldOfView::PixelScaleAt(61, "K".to_string()),
            band,
            Star::default(),
            hst,
        );
        field.save(format!("image{i}{band}.png"), None)?;
    }
    Ok(())
}
