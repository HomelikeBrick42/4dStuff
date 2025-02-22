#![deny(rust_2018_idioms)]

mod app;

pub use app::App;
use eframe::egui;

struct Camera {
    pub position: cgmath::Vector4<f32>,
}

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
