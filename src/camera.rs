use crate::{DrawUi, rotor::Rotor};
use eframe::egui;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Camera {
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

    /// returns whether the camera was updated
    pub fn update(&mut self, ts: f32, ctx: &egui::Context) -> bool {
        let mut changed = false;

        if !ctx.wants_keyboard_input() {
            ctx.input(|i| {
                let movement_amount = 4.0 * ts;
                let rotation_amount = std::f32::consts::FRAC_PI_2 * ts;

                let rotation = self.get_rotation();
                let forward = rotation.rotate(Camera::FORWARD * movement_amount);
                let right = rotation.rotate(Camera::RIGHT * movement_amount);
                let up = rotation.rotate(Camera::UP * movement_amount);
                let ana = rotation.rotate(Camera::ANA * movement_amount);

                if i.key_down(egui::Key::W) {
                    self.position += forward;
                    changed = true;
                }
                if i.key_down(egui::Key::S) {
                    self.position -= forward;
                    changed = true;
                }
                if i.key_down(egui::Key::A) {
                    self.position -= right;
                    changed = true;
                }
                if i.key_down(egui::Key::D) {
                    self.position += right;
                    changed = true;
                }
                if i.key_down(egui::Key::Q) {
                    self.position -= up;
                    changed = true;
                }
                if i.key_down(egui::Key::E) {
                    self.position += up;
                    changed = true;
                }
                if i.key_down(egui::Key::R) {
                    self.position += ana;
                    changed = true;
                }
                if i.key_down(egui::Key::F) {
                    self.position -= ana;
                    changed = true;
                }

                #[expect(clippy::collapsible_else_if)]
                if !self.volume_view_enabled {
                    if i.modifiers.shift {
                        if i.key_down(egui::Key::ArrowUp) {
                            self.base_rotation =
                                self.base_rotation * Rotor::rotation_xw(rotation_amount);
                            changed = true;
                        }
                        if i.key_down(egui::Key::ArrowDown) {
                            self.base_rotation =
                                self.base_rotation * Rotor::rotation_xw(-rotation_amount);
                            changed = true;
                        }
                        if i.key_down(egui::Key::ArrowLeft) {
                            self.base_rotation =
                                self.base_rotation * Rotor::rotation_zw(-rotation_amount);
                            changed = true;
                        }
                        if i.key_down(egui::Key::ArrowRight) {
                            self.base_rotation =
                                self.base_rotation * Rotor::rotation_zw(rotation_amount);
                            changed = true;
                        }
                    } else {
                        if i.key_down(egui::Key::ArrowUp) {
                            self.vertical_angle += rotation_amount;
                            changed = true;
                        }
                        if i.key_down(egui::Key::ArrowDown) {
                            self.vertical_angle -= rotation_amount;
                            changed = true;
                        }
                        if i.key_down(egui::Key::ArrowLeft) {
                            self.base_rotation =
                                self.base_rotation * Rotor::rotation_xz(-rotation_amount);
                            changed = true;
                        }
                        if i.key_down(egui::Key::ArrowRight) {
                            self.base_rotation =
                                self.base_rotation * Rotor::rotation_xz(rotation_amount);
                            changed = true;
                        }
                    }
                } else {
                    if i.modifiers.shift {
                        if i.key_down(egui::Key::ArrowLeft) {
                            self.base_rotation =
                                self.base_rotation * Rotor::rotation_zw(rotation_amount);
                            changed = true;
                        }
                        if i.key_down(egui::Key::ArrowRight) {
                            self.base_rotation =
                                self.base_rotation * Rotor::rotation_zw(-rotation_amount);
                            changed = true;
                        }
                    } else {
                        if i.key_down(egui::Key::ArrowUp) {
                            self.base_rotation =
                                self.base_rotation * Rotor::rotation_xw(rotation_amount);
                            changed = true;
                        }
                        if i.key_down(egui::Key::ArrowDown) {
                            self.base_rotation =
                                self.base_rotation * Rotor::rotation_xw(-rotation_amount);
                            changed = true;
                        }
                        if i.key_down(egui::Key::ArrowLeft) {
                            self.base_rotation =
                                self.base_rotation * Rotor::rotation_xz(-rotation_amount);
                            changed = true;
                        }
                        if i.key_down(egui::Key::ArrowRight) {
                            self.base_rotation =
                                self.base_rotation * Rotor::rotation_xz(rotation_amount);
                            changed = true;
                        }
                    }
                }

                if i.key_pressed(egui::Key::V) {
                    self.volume_view_enabled = !self.volume_view_enabled;
                }
            });
        }

        if changed {
            self.base_rotation = self.base_rotation.normalized();
        }

        let volume_view_duration = 0.5;
        if self.volume_view_enabled && self.volume_view_percentage < 1.0 {
            self.volume_view_percentage =
                (self.volume_view_percentage + ts / volume_view_duration).min(1.0);
            if self.volume_view_percentage == 1.0 {
                self.vertical_angle = 0.0;
            }
            changed = true;
        } else if !self.volume_view_enabled && self.volume_view_percentage > 0.0 {
            self.volume_view_percentage =
                (self.volume_view_percentage - ts / volume_view_duration).max(0.0);
            changed = true;
        }

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
