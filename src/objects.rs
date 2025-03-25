use enum_dispatch::enum_dispatch;

#[derive(Debug)]
pub struct HyperSphere {
    pub position: cgmath::Vector4<f32>,
    pub radius: f32,
    pub material: u32,
}

#[derive(Debug)]
pub struct HyperPlane {
    pub position: cgmath::Vector4<f32>,
    pub normal: cgmath::Vector4<f32>,
    pub material: u32,
}

#[derive(Debug)]
#[enum_dispatch(RayIntersect)]
pub enum Object {
    HyperSphere(HyperSphere),
    HyperPlane(HyperPlane),
}

impl Object {
    pub fn position(&self) -> cgmath::Vector4<f32> {
        match self {
            Object::HyperSphere(hyper_sphere) => hyper_sphere.position,
            Object::HyperPlane(hyper_plane) => hyper_plane.position,
        }
    }

    pub fn move_position(&mut self, offset: cgmath::Vector4<f32>) {
        match self {
            Object::HyperSphere(hyper_sphere) => hyper_sphere.position += offset,
            Object::HyperPlane(hyper_plane) => hyper_plane.position += offset,
        }
    }
}
