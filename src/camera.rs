use crate::rotor::Rotor;
use cgmath::Zero;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Camera {
    pub position: cgmath::Vector4<f32>,

    pub base_rotation: Rotor,
    pub volume_mode: bool,
    pub volume_mode_percentage: f32,
    pub xy_rotation: f32,

    pub sun_direction: cgmath::Vector4<f32>,
    pub sun_color: cgmath::Vector3<f32>,
    pub sun_light_color: cgmath::Vector3<f32>,
    pub ambient_light_color: cgmath::Vector3<f32>,
    pub up_sky_color: cgmath::Vector3<f32>,
    pub down_sky_color: cgmath::Vector3<f32>,
}

impl Camera {
    pub const FORWARD: cgmath::Vector4<f32> = cgmath::Vector4::new(1.0, 0.0, 0.0, 0.0);
    pub const UP: cgmath::Vector4<f32> = cgmath::Vector4::new(0.0, 1.0, 0.0, 0.0);
    pub const RIGHT: cgmath::Vector4<f32> = cgmath::Vector4::new(0.0, 0.0, 1.0, 0.0);
    pub const ANA: cgmath::Vector4<f32> = cgmath::Vector4::new(0.0, 0.0, 0.0, 1.0);

    pub fn get_rotation(&self) -> Rotor {
        self.base_rotation
            * Rotor::rotation_yw(core::f32::consts::FRAC_PI_2 * self.volume_mode_percentage)
            * Rotor::rotation_xy(self.xy_rotation)
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: cgmath::Vector4::zero(),

            base_rotation: Rotor::IDENTITY,
            volume_mode: false,
            volume_mode_percentage: 0.0,
            xy_rotation: 0.0,

            sun_direction: cgmath::vec4(-0.2, 1.0, 0.1, 0.0),
            sun_color: cgmath::vec3(0.9, 0.8, 0.7),
            sun_light_color: cgmath::vec3(1.0, 1.0, 1.0),
            ambient_light_color: cgmath::vec3(0.1, 0.1, 0.1),
            up_sky_color: cgmath::vec3(0.5, 0.5, 0.9),
            down_sky_color: cgmath::vec3(0.2, 0.2, 0.2),
        }
    }
}
