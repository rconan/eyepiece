use eyepiece::{Hexagon, Observer};

fn main() -> anyhow::Result<()> {
    let hex = Hexagon::new((1f64, 0.5f64), 1.32);
    hex.show_pupil(Option::<&str>::None)?;
    Ok(())
}
