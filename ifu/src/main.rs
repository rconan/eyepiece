use std::{f64::consts::PI, ops::Deref};

use clap::Parser;
use eyepiece::{
    Builder, Field, FieldBuilder, FieldImage, Gmt, Hexagon, Observer, SeeingBuilder, SeeingLimited,
};
use skyangle::SkyAngle;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Fried parameter [cm]
    #[arg(long, default_value_t = 16.0)]
    r0: f64,
    /// zenith angle [deg]
    #[arg(short, long, default_value_t = 30.0)]
    zenith_angle: f64,
    /// Photometric band, one of V,R,I,J,H,K
    #[arg(short, long, default_value_t = String::from("V"))]
    band: String,
    /// IFU hexagon flat-to-flat width [arcsec]
    #[arg(short, long, default_value_t = 0.4)]
    width_hex: f64,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // the telescope
    let tel = Gmt::new();
    // the seeing
    let seeing = SeeingBuilder::new(cli.r0 * 1e-2).zenith_angle(SkyAngle::Degree(cli.zenith_angle));
    // image pixel scale (1mas)
    let px = SkyAngle::MilliArcsec(1f64);
    // image size (6arsec x 6 arcsec)
    let n_px = 6001;

    // on-sky field definition
    let field: Field<Gmt, SeeingLimited> = FieldBuilder::new(tel)
        .pixel_scale(px)
        .field_of_view(n_px)
        .photometry(cli.band)
        .seeing_limited(seeing)
        .build();

    // field image
    let mut intensity = FieldImage::from(field);
    // total intensity
    let flux0 = intensity.flux();
    intensity.save("field.png", Default::default())?;

    // 7 hexagons IFU
    let ifu_width = SkyAngle::Arcsecond(cli.width_hex);
    let ifu_px = ifu_width / px;
    let ifu = IFU::new(ifu_px);
    // IFU throughput
    intensity.masked(&ifu);
    println!("7 Hex. IFU througput: {:.3}", intensity.flux() / flux0);
    intensity.save("masked_field.png", Default::default())?;

    // IFU hexagons throughput
    println!(
        "Individual Hex. throughput: {:.3?}",
        ifu.iter()
            .map(|hex| intensity.clone().masked(hex).flux() / flux0)
            .collect::<Vec<_>>()
    );

    Ok(())
}

/// Manifest IFU model
///
/// The IFU consist of 7 hexagons
#[derive(Debug, Clone)]
pub struct IFU(Vec<Hexagon>);
impl Deref for IFU {
    type Target = Vec<Hexagon>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl IFU {
    /// Creates a new IFU based on the size (flat-to-flat) of each hexagone
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
