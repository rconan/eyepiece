use eframe::egui;

use crate::{
    observation::{Based, ObservingMode, Telescope},
    Observation,
};

use super::Gui;

impl Gui for Based {
    fn gui(obs: &mut Observation, ui: &mut eframe::egui::Ui) {
        ui.horizontal(|ui| {
            ui.selectable_value(&mut obs.telescope, Based::Space(Telescope::HST), "HST");
            ui.selectable_value(&mut obs.telescope, Based::Space(Telescope::JWST), "JWST");
            ui.selectable_value(&mut obs.telescope, Based::Ground(Telescope::GMT), "GMT");
        });
        ui.horizontal(|ui| {
            // ui.selectable_value(
            //     &mut obs.telescope,
            //     Based::Ground(Telescope::Telescope(eyepiece::Telescope::new(8f64).build())),
            //     "Circular",
            // );
            if let Based::Ground(Telescope::Telescope(eyepiece::Telescope { diameter, .. })) =
                &mut obs.telescope
            {
                ui.add(egui::DragValue::new(diameter).speed(1.0).range(1.0..=50.0));
                ui.label("m diameter");
            }
        });
        if let Based::Space(_) = &obs.telescope {
            obs.mode = ObservingMode::DiffractionLimited
        }
    }
}
