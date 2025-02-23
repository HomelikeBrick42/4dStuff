#![deny(rust_2018_idioms)]

mod app;
pub mod transform;

pub use app::App;
use eframe::egui;
use serde::{Deserialize, Serialize};
use transform::Transform;

#[derive(Serialize, Deserialize)]
struct Camera {
    pub position: cgmath::Vector4<f32>,
    pub xy_rotation: f32,
    pub xz_rotation: f32,
    pub yz_rotation: f32,
    pub xw_rotation: f32,
    pub yw_rotation: f32,
    pub zw_rotation: f32,
}

impl Camera {
    fn get_transform(&self) -> Transform {
        Transform::translation(self.position)
            * Transform::rotation_xy(self.xy_rotation)
            * Transform::rotation_xz(self.xz_rotation)
            * Transform::rotation_yz(self.yz_rotation)
            * Transform::rotation_xw(self.xw_rotation)
            * Transform::rotation_yw(self.yw_rotation)
            * Transform::rotation_zw(self.zw_rotation)
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
        ui.horizontal(|ui| {
            ui.label("XY Rotation: ");
            changed |= ui.drag_angle(&mut self.xy_rotation).changed();
        });
        ui.horizontal(|ui| {
            ui.label("XZ Rotation: ");
            changed |= ui.drag_angle(&mut self.xz_rotation).changed();
        });
        ui.horizontal(|ui| {
            ui.label("YZ Rotation: ");
            changed |= ui.drag_angle(&mut self.yz_rotation).changed();
        });
        ui.horizontal(|ui| {
            ui.label("XW Rotation: ");
            changed |= ui.drag_angle(&mut self.xw_rotation).changed();
        });
        ui.horizontal(|ui| {
            ui.label("YW Rotation: ");
            changed |= ui.drag_angle(&mut self.yw_rotation).changed();
        });
        ui.horizontal(|ui| {
            ui.label("ZW Rotation: ");
            changed |= ui.drag_angle(&mut self.zw_rotation).changed();
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
