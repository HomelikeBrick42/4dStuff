pub mod fixed_size_buffer;
pub mod state;

pub const FORWARD: cgmath::Vector4<f32> = cgmath::Vector4::new(1.0, 0.0, 0.0, 0.0);
pub const UP: cgmath::Vector4<f32> = cgmath::Vector4::new(0.0, 1.0, 0.0, 0.0);
pub const RIGHT: cgmath::Vector4<f32> = cgmath::Vector4::new(0.0, 0.0, 1.0, 0.0);
pub const ANA: cgmath::Vector4<f32> = cgmath::Vector4::new(0.0, 0.0, 0.0, 1.0);

pub struct Camera {
    pub position: cgmath::Vector4<f32>,
    pub sun_direction: cgmath::Vector4<f32>,
    pub sun_color: cgmath::Vector3<f32>,
    pub sun_light_color: cgmath::Vector3<f32>,
    pub ambient_light_color: cgmath::Vector3<f32>,
    pub up_sky_color: cgmath::Vector3<f32>,
    pub down_sky_color: cgmath::Vector3<f32>,
}
