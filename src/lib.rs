#![deny(elided_lifetimes_in_paths)]

mod app;
pub mod gpu_buffer;
pub mod rotor;

pub use app::App;
use eframe::egui;
use rotor::Rotor;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Camera {
    pub position: cgmath::Vector4<f32>,
    pub base_rotation: Rotor,
    pub vertical_angle: f32,

    pub volume_view_enabled: bool,
    /// a value between 0 and 1, 0 is no volume view, 1 is full volume view
    pub volume_view_percentage: f32,
}

impl Camera {
    pub const FORWARD: cgmath::Vector4<f32> = cgmath::vec4(1.0, 0.0, 0.0, 0.0);
    pub const RIGHT: cgmath::Vector4<f32> = cgmath::vec4(0.0, 0.0, 1.0, 0.0);
    pub const UP: cgmath::Vector4<f32> = cgmath::vec4(0.0, 1.0, 0.0, 0.0);
    pub const ANA: cgmath::Vector4<f32> = cgmath::vec4(0.0, 0.0, 0.0, 1.0);

    pub fn get_rotation(&self) -> Rotor {
        self.base_rotation
            * Rotor::rotation_xy(self.vertical_angle * (1.0 - self.volume_view_percentage))
            * Rotor::rotation_yw(std::f32::consts::FRAC_PI_2 * self.volume_view_percentage)
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

        ui.horizontal(|ui| {
            ui.label("Position: ");
            changed |= self.position.draw_ui(ui);
        });

        ui.add_enabled_ui(false, |ui| {
            let rotation = self.get_rotation();
            let mut forward = rotation.rotate(Self::FORWARD);
            let mut right = rotation.rotate(Self::RIGHT);
            let mut up = rotation.rotate(Self::UP);
            let mut ana = rotation.rotate(Self::ANA);

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
            self.base_rotation = Rotor::IDENTITY;
            self.vertical_angle = 0.0;
            changed = true;
        }

        ui.horizontal(|ui| {
            ui.label("Volume View: ");
            ui.checkbox(&mut self.volume_view_enabled, "");
        });

        ui.horizontal(|ui| {
            ui.label("Vertical Angle: ");
            changed |= ui.drag_angle(&mut self.vertical_angle).changed();
            ui.label("(not used in volume view)");
        });

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
