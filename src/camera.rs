use crate::rotor::Rotor;
use cgmath::Zero;
use winit::{event::ElementState, keyboard::KeyCode};

#[derive(Debug)]
pub struct Camera {
    pub position: cgmath::Vector4<f32>,

    pub forward_movement: f32,
    pub up_movement: f32,
    pub right_movement: f32,
    pub ana_movement: f32,

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

    pub fn get_rotation_without_xy(&self) -> Rotor {
        self.base_rotation
            * Rotor::rotation_yw(core::f32::consts::FRAC_PI_2 * self.volume_mode_percentage)
    }

    pub fn get_rotation(&self) -> Rotor {
        self.get_rotation_without_xy()
            * Rotor::rotation_xy(self.xy_rotation * (1.0 - self.volume_mode_percentage))
    }

    pub fn update(&mut self, ts: f32) {
        if self.volume_mode {
            self.volume_mode_percentage += ts;
        } else {
            self.volume_mode_percentage -= ts;
        }
        self.volume_mode_percentage = self.volume_mode_percentage.clamp(0.0, 1.0);

        if self.volume_mode_percentage >= 1.0 {
            self.xy_rotation = 0.0;
        }

        let rotation = self.get_rotation_without_xy();
        let forward = rotation.rotate(Camera::FORWARD);
        let up = rotation.rotate(Camera::UP);
        let right = rotation.rotate(Camera::RIGHT);
        let ana = rotation.rotate(Camera::ANA);

        self.position += forward * (self.forward_movement * ts);
        self.position += up * (self.up_movement * ts);
        self.position += right * (self.right_movement * ts);
        self.position += ana * (self.ana_movement * ts);

        // not really sure if this needs to be done here, but doing it somewhere is probably good
        self.base_rotation = self.base_rotation.normalized();
    }

    pub fn key(&mut self, key: KeyCode, state: ElementState) {
        let speed = 2.0;
        let movement = match key {
            KeyCode::KeyW => Some((&mut self.forward_movement, speed)),
            KeyCode::KeyS => Some((&mut self.forward_movement, -speed)),
            KeyCode::KeyQ => Some((&mut self.up_movement, -speed)),
            KeyCode::KeyE => Some((&mut self.up_movement, speed)),
            KeyCode::KeyA => Some((&mut self.right_movement, -speed)),
            KeyCode::KeyD => Some((&mut self.right_movement, speed)),
            KeyCode::KeyR => Some((&mut self.ana_movement, speed)),
            KeyCode::KeyF => Some((&mut self.ana_movement, -speed)),
            _ => None,
        };

        if let Some((movement, amount)) = movement {
            match state {
                ElementState::Pressed => *movement = amount,
                ElementState::Released => *movement = 0.0,
            }
        }
    }

    pub fn reset_keys(&mut self) {
        self.forward_movement = 0.0;
        self.up_movement = 0.0;
        self.right_movement = 0.0;
        self.ana_movement = 0.0;
    }

    pub fn mouse_moved(&mut self, delta: cgmath::Vector2<f32>) {
        let sensitivity = 0.01;

        if self.volume_mode {
            self.base_rotation = self.base_rotation * Rotor::rotation_xz(delta.x * sensitivity);
            self.base_rotation = self.base_rotation * Rotor::rotation_xw(delta.y * sensitivity);
        } else {
            self.xy_rotation -= delta.y * sensitivity;
            self.xy_rotation = self
                .xy_rotation
                .clamp(-core::f32::consts::FRAC_PI_2, core::f32::consts::FRAC_PI_2);

            self.base_rotation = self.base_rotation * Rotor::rotation_xz(delta.x * sensitivity);
        }
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: cgmath::Vector4::zero(),

            forward_movement: 0.0,
            up_movement: 0.0,
            right_movement: 0.0,
            ana_movement: 0.0,

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
