use eframe::egui;

use crate::Program;

impl Program {
    pub fn gui(&mut self, ui: &mut eframe::egui::Ui) {
        if let Some(archive) = self.archive.as_mut() {
            egui::ComboBox::from_label("")
                .selected_text(self.observation.to_string())
                .show_ui(ui, |ui| {
                    for obs in archive.observations.iter() {
                        ui.selectable_value(&mut self.observation, obs.clone(), obs.to_string());
                    }
                });
        } else {
            egui::ComboBox::from_label("").show_ui(ui, |_| {});
        }
    }
}
