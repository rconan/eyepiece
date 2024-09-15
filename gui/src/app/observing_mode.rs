use eframe::egui;
use eyepiece::SeeingBuilder;
use skyangle::SkyAngle;

use crate::{
    observation::{Based, ObservingMode},
    Observation,
};

use super::Gui;

impl Gui for ObservingMode {
    fn gui(obs: &mut Observation, ui: &mut eframe::egui::Ui) {
        ui.selectable_value(
            &mut obs.mode,
            ObservingMode::DiffractionLimited,
            "Diffraction Limited",
        );
        if let Based::Ground(_) = obs.telescope {
            ui.selectable_value(
                &mut obs.mode,
                ObservingMode::SeeingLimited(
                    SeeingBuilder::new(0.15)
                        .zenith_angle(SkyAngle::Degree(0.))
                        .outer_scale(30.),
                ),
                "Seeing Limited",
            );
            ui.selectable_value(
                &mut obs.mode,
                ObservingMode::AdaptiveOptics {
                    seeing: SeeingBuilder::new(0.15)
                        .zenith_angle(SkyAngle::Degree(0.))
                        .outer_scale(30.),
                    strehl_ratio: 50f64,
                },
                "Adaptive Optics",
            );
        }
        if let Based::Ground(_) = obs.telescope {
            if let ObservingMode::SeeingLimited(SeeingBuilder {
                fried_parameter,
                outer_scale,
                ..
            })
            | ObservingMode::AdaptiveOptics {
                seeing:
                    SeeingBuilder {
                        fried_parameter,
                        outer_scale,
                        ..
                    },
                ..
            } = &mut obs.mode
            {
                ui.horizontal(|ui| {
                    ui.label("Fried parameter");
                    ui.add(
                        egui::DragValue::new(fried_parameter)
                            .speed(0.01)
                            .range(0.05..=0.5),
                    );
                    ui.label("m");
                });
                ui.horizontal(|ui| {
                    ui.label("outer scale");
                    ui.add(
                        egui::DragValue::new(outer_scale)
                            .speed(5.0)
                            .range(5.0..=100.0),
                    );
                    ui.label("m");
                });
                if let ObservingMode::AdaptiveOptics { strehl_ratio, .. } = &mut obs.mode {
                    ui.horizontal(|ui| {
                        ui.label("Strehl ratio");
                        ui.add(
                            egui::DragValue::new(strehl_ratio)
                                .speed(5.0)
                                .range(50.0..=90.0),
                        );
                        ui.label("[%]");
                    });
                }
            }
        }
    }
}
