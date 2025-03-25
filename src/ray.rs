use crate::objects::{HyperPlane, HyperSphere, Object};
use cgmath::InnerSpace;
use enum_dispatch::enum_dispatch;

#[derive(Debug, Clone, Copy)]
pub struct Ray {
    pub origin: cgmath::Vector4<f32>,
    pub direction: cgmath::Vector4<f32>,
}

#[derive(Debug, Clone, Copy)]
pub struct Hit {
    pub distance: f32,
    pub position: cgmath::Vector4<f32>,
    pub normal: cgmath::Vector4<f32>,
    pub material: u32,
}

#[enum_dispatch]
pub trait RayIntersect {
    fn intersect(&self, ray: Ray) -> Option<Hit>;
}

impl RayIntersect for HyperSphere {
    fn intersect(&self, ray: Ray) -> Option<Hit> {
        let oc = self.position - ray.origin;
        // TODO: can this be replaced with 1?
        let a = ray.direction.dot(ray.direction);
        let h = ray.direction.dot(oc);
        let c = oc.dot(oc) - self.radius * self.radius;
        let discriminant = h * h - a * c;

        if discriminant < 0.0 {
            return None;
        }

        let distance = (h - discriminant.sqrt()) / a;
        if distance <= 0.0 {
            return None;
        }

        let position = ray.origin + ray.direction * distance;
        let normal = (position - self.position) / self.radius;
        let material = self.material;
        Some(Hit {
            distance,
            position,
            normal,
            material,
        })
    }
}

impl RayIntersect for HyperPlane {
    fn intersect(&self, ray: Ray) -> Option<Hit> {
        let denom = self.normal.dot(ray.direction);
        let distance = (self.position - ray.origin).dot(self.normal) / denom;
        if distance <= 0.0 {
            return None;
        }

        let position = ray.origin + ray.direction * distance;
        let normal = self.normal * -denom.signum();
        let material = self.material;
        Some(Hit {
            distance,
            position,
            normal,
            material,
        })
    }
}
