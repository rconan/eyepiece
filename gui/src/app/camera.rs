use eframe::egui;
use eyepiece::PhotometricBands;
use skyangle::SkyAngle;

use crate::{observation::Camera, CameraSkyPixel, Observation};

use super::Gui;

impl Gui for Camera {
    fn gui(obs: &mut Observation, ui: &mut eframe::egui::Ui) {
        egui::ComboBox::from_label("Select band")
            .selected_text(format!("{}", obs.camera.spectral_filter))
            .show_ui(ui, |ui| {
                PhotometricBands::default().into_iter().for_each(|b| {
                    ui.selectable_value(&mut obs.camera.spectral_filter, b.into(), b);
                });
            });
        ui.horizontal(|ui| {
            ui.label("Pixel scale:");
            ui.add(
                egui::DragValue::new(&mut obs.camera.pixel_scale)
                    .speed(1.0)
                    .range(1.0..=1000.0),
            );
            ui.label("mas");
        });
        ui.horizontal(|ui| {
            ui.label("Field of view:");
            ui.add(
                egui::DragValue::new(obs.camera.field_of_view.as_mut())
                    .speed(0.1)
                    .range(0.1..=60.),
            );
            ui.label(r#"""#);
        });
        ui.horizontal(|ui| {
            ui.label("Exposure:");
            ui.add(
                egui::DragValue::new(&mut obs.camera.exposure)
                    .speed(0.1)
                    .range(1. ..=3660.),
            );
            ui.label("s");
        });
    }
}

impl eframe::emath::Numeric for CameraSkyPixel {
    const INTEGRAL: bool = false;

    const MIN: Self = CameraSkyPixel(SkyAngle::Radian(0f64));

    const MAX: Self = CameraSkyPixel(SkyAngle::Radian(f64::MAX));

    fn to_f64(self) -> f64 {
        self.0.into_value()
    }

    fn from_f64(num: f64) -> Self {
        CameraSkyPixel(SkyAngle::MilliArcsec(num))
    }
}
