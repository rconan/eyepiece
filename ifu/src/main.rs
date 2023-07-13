use clap::{Parser, Subcommand};
use eyepiece::{
    Builder, Field, FieldBuilder, FieldImage, Gmt, SeeingBuilder, SeeingLimited, Telescope,
};
use ifu::{Slit, Throughput, IFU};
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
    #[command(subcommand)]
    ifu: Option<IfuKind>,
}

#[derive(Subcommand)]
enum IfuKind {
    /// 7 hexagons IFU
    Hex {
        /// IFU hexagon flat-to-flat width [arcsec]
        #[arg(short, long, default_value_t = 0.4)]
        width_hex: f64,
    },
    /// Round IFU
    Round {
        /// IFU diameter [arcsec]
        #[arg(short, long, default_value_t = 0.84)]
        diameter: f64,
    },
    /// Slit IFU
    Slit {
        /// slit width [arcsec]
        #[arg(short, long, default_value_t = 0.4)]
        width: f64,
        /// slit length [arcsec]
        #[arg(short, long, default_value_t = 5.0)]
        length: f64,
    },
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
    intensity.save("field.png", Default::default())?;

    // IFU
    match cli.ifu {
        Some(IfuKind::Slit { width, length }) => {
            let ifu_width = SkyAngle::Arcsecond(width);
            let ifu_length = SkyAngle::Arcsecond(length);
            let ifu = Slit::new(ifu_width / px, ifu_length / px);
            ifu.throughput(&mut intensity)?;
        }
        Some(IfuKind::Round { diameter }) => {
            let ifu_width = SkyAngle::Arcsecond(diameter);
            let ifu_px = ifu_width / px;
            let ifu = Telescope::new(ifu_px).build();
            ifu.throughput(&mut intensity)?;
        }
        Some(IfuKind::Hex { width_hex }) => {
            let ifu_width = SkyAngle::Arcsecond(width_hex);
            let ifu_px = ifu_width / px;
            let ifu = IFU::new(ifu_px);
            ifu.throughput(&mut intensity)?;
        }
        None => {
            let ifu_width = SkyAngle::Arcsecond(0.4);
            let ifu_px = ifu_width / px;
            let ifu = IFU::new(ifu_px);
            ifu.throughput(&mut intensity)?;
        }
    }

    Ok(())
}
