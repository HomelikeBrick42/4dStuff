use crate::camera::Camera;
use encase::ShaderType;

#[derive(ShaderType)]
pub struct GpuCamera {
    pub position: cgmath::Vector4<f32>,
    pub forward: cgmath::Vector4<f32>,
    pub up: cgmath::Vector4<f32>,
    pub right: cgmath::Vector4<f32>,
    pub sun_direction: cgmath::Vector4<f32>,
    pub sun_color: cgmath::Vector3<f32>,
    pub sun_light_color: cgmath::Vector3<f32>,
    pub ambient_light_color: cgmath::Vector3<f32>,
    pub up_sky_color: cgmath::Vector3<f32>,
    pub down_sky_color: cgmath::Vector3<f32>,
}

impl GpuCamera {
    pub fn from_camera(camera: &Camera) -> Self {
        let Camera {
            position,

            forward_movement: _,
            up_movement: _,
            right_movement: _,
            ana_movement: _,

            base_rotation: _,
            volume_mode: _,
            volume_mode_percentage: _,
            xy_rotation: _,

            sun_direction,
            sun_color,
            sun_light_color,
            ambient_light_color,
            up_sky_color,
            down_sky_color,
        } = *camera;
        let rotation = camera.get_rotation();
        Self {
            position,
            forward: rotation.rotate(Camera::FORWARD),
            up: rotation.rotate(Camera::UP),
            right: rotation.rotate(Camera::RIGHT),
            sun_direction,
            sun_color,
            sun_light_color,
            ambient_light_color,
            up_sky_color,
            down_sky_color,
        }
    }
}
