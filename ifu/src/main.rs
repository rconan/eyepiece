use std::{f64::consts::PI, ops::Deref};

use eyepiece::{
    Builder, Field, FieldBuilder, FieldImage, Gmt, Hexagon, Observer, SeeingBuilder, SeeingLimited,
};
use skyangle::SkyAngle;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let tel = Gmt::new();
    let seeing = SeeingBuilder::new(16e-2).zenith_angle(SkyAngle::Degree(30.));
    let px = SkyAngle::MilliArcsec(1f64);
    let n_px = 6001;

    let field: Field<Gmt, SeeingLimited> = FieldBuilder::new(tel)
        .pixel_scale(px)
        .field_of_view(n_px)
        .photometry("V")
        .seeing_limited(seeing)
        .build();

    let mut intensity = FieldImage::from(field);
    let flux0 = intensity.flux();
    intensity.save("field.png", Default::default())?;

    let ifu_width = SkyAngle::Arcsecond(0.4);
    let ifu_px = ifu_width / px;
    let ifu = IFU::new(ifu_px);
    intensity.masked(&ifu); //.get(0).unwrap());
    println!("7 Hex. IFU througput: {:.3}", intensity.flux() / flux0);
    intensity.save("masked_field.png", Default::default())?;

    println!(
        "Individual Hex. throughput: {:.3?}",
        ifu.iter()
            .map(|hex| intensity.clone().masked(hex).flux() / flux0)
            .collect::<Vec<_>>()
    );

    Ok(())
}

#[derive(Debug, Clone)]
pub struct IFU(Vec<Hexagon>);
impl Deref for IFU {
    type Target = Vec<Hexagon>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl IFU {
    pub fn new(f2f: f64) -> Self {
        let (s, c) = (PI / 6.).sin_cos();
        Self(vec![
            Hexagon::new((0f64, 0f64), f2f),
            Hexagon::new((0f64, f2f), f2f),
            Hexagon::new((0f64, -f2f), f2f),
            Hexagon::new((c * f2f, s * f2f), f2f),
            Hexagon::new((c * f2f, -s * f2f), f2f),
            Hexagon::new((-c * f2f, s * f2f), f2f),
            Hexagon::new((-c * f2f, -s * f2f), f2f),
        ])
    }
}
impl Observer for IFU {
    fn diameter(&self) -> f64 {
        todo!()
    }

    fn inside_pupil(&self, x: f64, y: f64) -> bool {
        for hex in self.iter() {
            if hex.inside_pupil(x, y) {
                return true;
            }
        }
        false
    }
}
