use eframe::egui::{self, Layout};

use crate::Program;

impl Program {
    pub fn gui(&mut self, ui: &mut eframe::egui::Ui) {
        ui.horizontal(|ui| {
            if let Some(archive) = self.archive.as_mut() {
                egui::ComboBox::from_label("")
                    .selected_text(self.observation.to_string())
                    .show_ui(ui, |ui| {
                        for obs in archive.observations.iter() {
                            ui.selectable_value(
                                &mut self.observation,
                                obs.clone(),
                                obs.to_string(),
                            );
                        }
                    });
            } else {
                egui::ComboBox::from_label("").show_ui(ui, |_| {});
            }
            ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Delete field!").clicked() {
                    if let Some(archive) = self.archive.take() {
                        let observations: Vec<_> = archive
                            .observations
                            .into_iter()
                            .filter_map(|obs| (obs != self.observation).then_some(obs))
                            .collect();
                        if observations.is_empty() {
                            self.observation = Default::default();
                        } else {
                            self.observation = observations[0].clone();
                            self.archive = Some(observations.into());
                        }
                    }
                    // ctx.request_repaint();
                }
            });
        });
    }
}
