#![deny(elided_lifetimes_in_paths)]

mod app;
mod camera;
pub mod gpu_buffer;
mod hyper_sphere;
pub mod rotor;

pub use app::App;
use eframe::egui;

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
