#![deny(rust_2018_idioms)]

mod app;
pub mod transform;

pub use app::App;
use eframe::egui;
use serde::{Deserialize, Serialize};
use transform::Transform;

#[derive(Serialize, Deserialize)]
enum Camera {
    Normal {
        base_transform: Transform,
        vertical_look: f32,
    },
    Volume {
        transform: Transform,
    },
}

impl Camera {
    pub fn get_transform(&self) -> Transform {
        match *self {
            Self::Normal {
                base_transform,
                vertical_look,
            } => base_transform * Transform::rotation_xz(vertical_look),
            Self::Volume { transform } => {
                transform * Transform::rotation_zw(std::f32::consts::FRAC_PI_2)
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

        let transform = self.get_transform();
        let mut position = transform.transform(cgmath::vec4(0.0, 0.0, 0.0, 0.0));
        let old_position = position;
        ui.horizontal(|ui| {
            ui.label("Position: ");
            if position.draw_ui(ui) {
                let difference = position - old_position;
                let transform = match self {
                    Self::Normal {
                        base_transform: transform,
                        vertical_look: _,
                    }
                    | Self::Volume { transform } => transform,
                };
                *transform = Transform::translation(difference) * *transform;
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

        let volume_enabled = match self {
            Camera::Normal {
                base_transform: _,
                vertical_look,
            } => *vertical_look == 0.0,
            Camera::Volume { transform: _ } => true,
        };

        if ui.button("Reset Rotation").clicked() {
            let transform = match self {
                Camera::Normal {
                    base_transform,
                    vertical_look,
                } => {
                    *vertical_look = 0.0;
                    base_transform
                }
                Camera::Volume { transform } => transform,
            };
            *transform = Transform::translation(position);
            changed = true;
        }

        ui.horizontal(|ui| {
            ui.label("Volume View: ");

            let mut volume = matches!(self, Self::Volume { .. });
            ui.add_enabled_ui(volume_enabled, |ui| {
                if ui.checkbox(&mut volume, "").clicked() {
                    match *self {
                        Camera::Normal {
                            base_transform,
                            vertical_look: _,
                        } if volume => {
                            *self = Camera::Volume {
                                transform: base_transform,
                            };
                        }

                        Camera::Volume { transform } if !volume => {
                            *self = Camera::Normal {
                                base_transform: transform,
                                vertical_look: 0.0,
                            };
                        }

                        _ => {}
                    };
                    changed = true;
                }
            });
        });

        if let Self::Normal {
            base_transform: _,
            vertical_look,
        } = self
        {
            ui.horizontal(|ui| {
                ui.label("Vertical Angle: ");
                changed |= ui.drag_angle(vertical_look).changed();
            });
            if !volume_enabled {
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
