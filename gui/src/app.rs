use eframe::egui::{self, load::SizedTexture, TextureOptions};

use crate::{
    observation::{Based, Camera, Stars},
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
            // ObservingMode::show(self, ui, "OBSERVING MODE");
            Camera::show(self, ui, "CAMERA");
            Stars::show(self, ui, "STARS");
            ui.horizontal(|ui| {
                if ui.button("Compute field!").clicked() {
                    self.build();
                    #[cfg(target_arch = "wasm32")]
                    web_sys::console::log_1(&format!("{:?}", self.state).into());
                }
                // #[cfg(target_arch = "wasm32")]
                // web_sys::console::log_1(&"building field ...".into());
                if !self.is_built() {
                    if let State::Building = self.state {
                        #[cfg(target_arch = "wasm32")]
                        web_sys::console::log_1(&"computing...".into());
                        ui.spinner();
                        ui.label("computing!");
                        ctx.request_repaint();
                    }
                }
            });
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            self.gui(ui);
            match self.state {
                State::Observing => {
                    let available_size = ui.available_size();
                    if let Some(image) = self.observation.image.clone() {
                        let texture =
                            ctx.load_texture("field image", image, TextureOptions::NEAREST);
                        ui.add(
                            egui::Image::new(SizedTexture::from(&texture))
                                .fit_to_exact_size(available_size),
                        );
                    }
                }
                _ => (),
            }
        });
    }
}
