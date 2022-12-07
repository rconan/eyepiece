use eyepiece::{Field, Observer, Telescope};
use skyangle::SkyAngle;

fn main() {
    let tel: Telescope = Default::default();
    let tel = Telescope::new(2.4).obscuration(0.3).build();
    // tel.show_pupil();
    let field = Field::new(
        SkyAngle::Arcsecond(0.1),
        SkyAngle::Arcminute(1.),
        "V",
        Default::default(),
        tel,
    );
    field.observer.show_pupil();
}
