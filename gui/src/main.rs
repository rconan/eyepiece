use eyepiece_gui::Program;
use std::default::Default;
use std::env;

fn main() -> eframe::Result {
    env_logger::init();
    env::set_var("SEED", "peekaboo42");

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Eyepiece",
        native_options,
        Box::new(|_cc| Ok(Box::new(Program::default()))),
    )
}
