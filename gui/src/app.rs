use eframe::egui::{self, load::SizedTexture, ProgressBar, TextureOptions};

use crate::{
    observation::{Based, Camera, ObservingMode, Stars},
    program::State,
    Observation, Program,
};

mod camera;
mod observing_mode;
mod program;
mod stars;
mod telescope;

pub trait Gui {
    fn gui(_obs: &mut Observation, _ui: &mut egui::Ui);
    fn show(prog: &mut Program, ui: &mut egui::Ui, label: &str) {
        ui.group(|ui| {
            ui.label(label);
            <Self as Gui>::gui(&mut prog.observation, ui);
        });
    }
}

impl eframe::App for Program {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::left("Controls").show(ctx, |ui| {
            Based::show(self, ui, "TELESCOPE");
            ObservingMode::show(self, ui, "OBSERVING MODE");
            Camera::show(self, ui, "CAMERA");
            Stars::show(self, ui, "STARS");
            ui.add(ProgressBar::new(0.));
            if ui.button("Compute field!").clicked() {
                self.state = State::Building;
                self.observation.build();
                self.archive
                    .get_or_insert(Default::default())
                    .observations
                    .push(self.observation.clone());
                self.state = State::Observing;
                ctx.request_repaint();
            }
            ui.group(|ui| {
                ui.label("ARCHIVE");
                self.gui(ui)
            });
        });
        egui::CentralPanel::default().show(ctx, |ui| match self.state {
            State::Observing => {
                let available_size = ui.available_size();
                if let Some(image) = self.observation.image.clone() {
                    let texture = ctx.load_texture("field image", image, TextureOptions::NEAREST);
                    ui.add(
                        egui::Image::new(SizedTexture::from(&texture))
                            .fit_to_exact_size(available_size),
                    );
                }
            }
            _ => (),
        });
    }
}
