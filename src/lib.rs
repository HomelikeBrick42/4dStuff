#![deny(rust_2018_idioms)]

mod app;
pub mod transform;

pub use app::App;
use eframe::egui;
use serde::{Deserialize, Serialize};
use transform::Transform;

#[derive(Serialize, Deserialize)]
struct Camera {
    pub base_transform: Transform,
    pub extra_transform: Transform,
}

impl Camera {
    pub fn get_transform(&self) -> Transform {
        self.base_transform * self.extra_transform
    }
}

#[derive(Serialize, Deserialize)]
struct HyperSphere {
    pub name: String,
    pub ui_id: usize,
    pub position: cgmath::Vector4<f32>,
    pub radius: f32,
    pub color: cgmath::Vector3<f32>,
}

trait DrawUi {
    fn draw_ui(&mut self, ui: &mut egui::Ui) -> bool;
}

impl DrawUi for cgmath::Vector4<f32> {
    fn draw_ui(&mut self, ui: &mut egui::Ui) -> bool {
        let mut changed = false;
        ui.horizontal(|ui| {
            changed |= ui
                .add(egui::DragValue::new(&mut self.x).prefix("x: ").speed(0.1))
                .changed();
            changed |= ui
                .add(egui::DragValue::new(&mut self.y).prefix("y: ").speed(0.1))
                .changed();
            changed |= ui
                .add(egui::DragValue::new(&mut self.z).prefix("z: ").speed(0.1))
                .changed();
            changed |= ui
                .add(egui::DragValue::new(&mut self.w).prefix("w: ").speed(0.1))
                .changed();
        });
        changed
    }
}

impl DrawUi for Camera {
    fn draw_ui(&mut self, ui: &mut egui::Ui) -> bool {
        let mut changed = false;

        let transform = self.get_transform();
        let mut position = transform.transform(cgmath::vec4(0.0, 0.0, 0.0, 0.0));
        let old_position = position;
        ui.horizontal(|ui| {
            ui.label("Position: ");
            if position.draw_ui(ui) {
                let difference = position - old_position;
                self.base_transform = Transform::translation(difference) * self.base_transform;
                changed = true;
            }
        });

        ui.add_enabled_ui(false, |ui| {
            let mut forward = transform.transform_direction(cgmath::vec4(1.0, 0.0, 0.0, 0.0));
            let mut right = transform.transform_direction(cgmath::vec4(0.0, 1.0, 0.0, 0.0));
            let mut up = transform.transform_direction(cgmath::vec4(0.0, 0.0, 1.0, 0.0));
            let mut ana = transform.transform_direction(cgmath::vec4(0.0, 0.0, 0.0, 1.0));
            ui.horizontal(|ui| {
                ui.label("Forward: ");
                forward.draw_ui(ui);
            });
            ui.horizontal(|ui| {
                ui.label("Right: ");
                right.draw_ui(ui);
            });
            ui.horizontal(|ui| {
                ui.label("Up: ");
                up.draw_ui(ui);
            });
            ui.horizontal(|ui| {
                ui.label("Ana: ");
                ana.draw_ui(ui);
            });
        });

        if ui.button("Reset Rotation").clicked() {
            self.base_transform = Transform::translation(position);
            self.extra_transform = Transform::IDENTITY;
            changed = true;
        }

        changed
    }
}

impl DrawUi for HyperSphere {
    fn draw_ui(&mut self, ui: &mut egui::Ui) -> bool {
        let mut changed = false;
        ui.horizontal(|ui| {
            ui.label("Name: ");
            changed |= ui.text_edit_singleline(&mut self.name).changed();
        });
        ui.horizontal(|ui| {
            ui.label("Position: ");
            changed |= self.position.draw_ui(ui);
        });
        ui.horizontal(|ui| {
            ui.label("Radius: ");
            changed |= ui
                .add(egui::DragValue::new(&mut self.radius).speed(0.1))
                .changed();
        });
        ui.horizontal(|ui| {
            ui.label("Color: ");
            changed |= ui.color_edit_button_rgb(self.color.as_mut()).changed();
        });
        changed
    }
}
