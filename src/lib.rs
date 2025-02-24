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
    pub mode: CameraMode,
}

#[derive(Serialize, Deserialize)]
enum CameraMode {
    Normal { vertical_angle: f32 },
    Volume,
}

impl Camera {
    pub fn get_rotation(&self) -> Rotor {
        match self.mode {
            CameraMode::Normal { vertical_angle } => {
                self.base_rotation * Rotor::rotation_xz(vertical_angle)
            }
            CameraMode::Volume => {
                self.base_rotation * Rotor::rotation_zw(std::f32::consts::FRAC_PI_2)
            }
        }
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
            let mut forward = rotation.rotate(cgmath::vec4(1.0, 0.0, 0.0, 0.0));
            let mut right = rotation.rotate(cgmath::vec4(0.0, 1.0, 0.0, 0.0));
            let mut up = rotation.rotate(cgmath::vec4(0.0, 0.0, 1.0, 0.0));
            let mut ana = rotation.rotate(cgmath::vec4(0.0, 0.0, 0.0, 1.0));

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

        let volume_change_enabled = match self.mode {
            CameraMode::Normal { vertical_angle } => vertical_angle == 0.0,
            CameraMode::Volume => true,
        };

        if ui.button("Reset Rotation").clicked() {
            match &mut self.mode {
                CameraMode::Normal { vertical_angle } => *vertical_angle = 0.0,
                CameraMode::Volume => {}
            }
            self.base_rotation = Rotor::IDENTITY;
            changed = true;
        }

        ui.horizontal(|ui| {
            ui.label("Volume View: ");

            let mut volume = matches!(self.mode, CameraMode::Volume);
            ui.add_enabled_ui(volume_change_enabled, |ui| {
                if ui.checkbox(&mut volume, "").clicked() {
                    self.mode = if volume {
                        CameraMode::Volume
                    } else {
                        CameraMode::Normal {
                            vertical_angle: 0.0,
                        }
                    };
                    changed = true;
                }
            });
        });

        if let CameraMode::Normal { vertical_angle } = &mut self.mode {
            ui.horizontal(|ui| {
                ui.label("Vertical Angle: ");
                changed |= ui.drag_angle(vertical_angle).changed();
            });
            if !volume_change_enabled {
                ui.label("Volume View cannot be enabled if the vertical angle is not exactly 0");
            }
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
