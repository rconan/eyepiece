use eframe::egui;
use eyepiece::{MagnitudeDistribution, StarDistribution};
use skyangle::SkyAngle;

use crate::{
    observation::{
        stars::{Distribution, Magnitude},
        Camera, Stars,
    },
    Observation,
};

use super::Gui;

impl Gui for Stars {
    fn gui(obs: &mut Observation, ui: &mut eframe::egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Distribution: ");
            ui.selectable_value(
                &mut obs.stars.distribution,
                Distribution::Uniform(StarDistribution::Uniform(
                    Camera::default().field_of_view,
                    1,
                )),
                "Uniform",
            );
            ui.selectable_value(
                &mut obs.stars.distribution,
                Distribution::Globular(StarDistribution::GlobularBoxed {
                    center: None,
                    scale: SkyAngle::Arcsecond(1f64),
                    n_sample: 1,
                    width: Camera::default().field_of_view,
                }),
                "Globular",
            );
        });
        match &mut obs.stars.distribution {
            Distribution::Uniform(StarDistribution::Uniform(_, n_sample)) => {
                ui.horizontal(|ui| {
                    ui.label("Star #");
                    ui.add(egui::DragValue::new(n_sample).speed(1).range(0..=1000));
                });
            }
            Distribution::Globular(StarDistribution::GlobularBoxed {
                scale, n_sample, ..
            }) => {
                ui.horizontal(|ui| {
                    ui.label("Star #");
                    ui.add(egui::DragValue::new(n_sample).speed(1).range(0..=100));
                    ui.label("Radius");
                    ui.add(
                        egui::DragValue::new(scale.as_mut())
                            .speed(0.1)
                            .range(0.1..=60.0),
                    );
                    ui.label(r#"""#);
                });
            }
            _ => unimplemented!(),
        }

        ui.horizontal(|ui| {
            ui.label("Magnitude: ");
            ui.selectable_value(
                &mut obs.stars.magnitude,
                Magnitude::Normal(MagnitudeDistribution::Normal(14f64, 1f64)),
                "Normal",
            );
            ui.selectable_value(
                &mut obs.stars.magnitude,
                Magnitude::LogNormal(MagnitudeDistribution::LogNormal(0f64, 14f64, 1f64)),
                "Log-Normal",
            );
        });

        match &mut obs.stars.magnitude {
            Magnitude::Normal(MagnitudeDistribution::Normal(mean, std)) => {
                ui.horizontal(|ui| {
                    ui.label("Mean");
                    ui.add(egui::DragValue::new(mean).speed(0.1));
                    ui.label("Std");
                    ui.add(egui::DragValue::new(std).speed(0.1).range(0..=10));
                });
            }
            Magnitude::LogNormal(MagnitudeDistribution::LogNormal(offset, mean, std)) => {
                ui.horizontal(|ui| {
                    ui.label("Offset");
                    ui.add(egui::DragValue::new(offset).speed(0.1));
                    ui.label("Mean");
                    ui.add(egui::DragValue::new(mean).speed(0.1));
                    ui.label("Scale");
                    ui.add(egui::DragValue::new(std).speed(0.1));
                });
            }
            _ => unimplemented!(),
        }
        ui.checkbox(&mut obs.stars.seed, "Recompute random distributions");
    }
}
