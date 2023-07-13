use eyepiece::{FieldImage, Telescope};

mod manifest;
pub use manifest::IFU;
mod slit;
pub use slit::Slit;

/// IFU throughput definition
pub trait Throughput {
    fn throughput(&self, field_image: &mut FieldImage) -> image::ImageResult<()>;
}

impl Throughput for IFU {
    fn throughput(&self, field_image: &mut FieldImage) -> image::ImageResult<()> {
        let flux0 = field_image.flux();
        // IFU throughput
        field_image.masked(self);
        println!("7 Hex. IFU throughput: {:.3}", field_image.flux() / flux0);
        field_image.save("hex_ifu_field.png", Default::default())?;

        // IFU hexagons throughput
        println!(
            "Individual Hex. throughput: {:.3?}",
            self.iter()
                .map(|hex| field_image.clone().masked(hex).flux() / flux0)
                .collect::<Vec<_>>()
        );
        Ok(())
    }
}

impl Throughput for Telescope {
    fn throughput(&self, field_image: &mut FieldImage) -> image::ImageResult<()> {
        let flux0 = field_image.flux();
        // IFU throughput
        field_image.masked(self);
        println!("Round IFU throughput: {:.3}", field_image.flux() / flux0);
        field_image.save("round_ifu_field.png", Default::default())?;

        Ok(())
    }
}

impl Throughput for Slit {
    fn throughput(&self, field_image: &mut FieldImage) -> image::ImageResult<()> {
        let flux0 = field_image.flux();
        // IFU throughput
        field_image.masked(self);
        println!("Slit IFU throughput: {:.3}", field_image.flux() / flux0);
        field_image.save("slit_ifu_field.png", Default::default())?;

        Ok(())
    }
}
