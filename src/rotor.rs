use encase::ShaderType;
use serde::{Deserialize, Serialize};
use std::ops::{Mul, Not};

#[derive(Debug, Clone, Copy, ShaderType, Serialize, Deserialize)]
pub struct Rotor {
    pub s: f32,
    pub e12: f32,
    pub e13: f32,
    pub e14: f32,
    pub e23: f32,
    pub e24: f32,
    pub e34: f32,
    pub e1234: f32,
}

impl Rotor {
    pub const IDENTITY: Self = Self {
        s: 1.0,
        e12: 0.0,
        e13: 0.0,
        e14: 0.0,
        e23: 0.0,
        e24: 0.0,
        e34: 0.0,
        e1234: 0.0,
    };

    pub fn rotation_xy(angle: f32) -> Self {
        let (sin, cos) = (angle * 0.5).sin_cos();
        Self {
            s: cos,
            e12: sin,
            ..Self::IDENTITY
        }
    }

    pub fn rotation_xz(angle: f32) -> Self {
        let (sin, cos) = (angle * 0.5).sin_cos();
        Self {
            s: cos,
            e13: -sin,
            ..Self::IDENTITY
        }
    }

    pub fn rotation_xw(angle: f32) -> Self {
        let (sin, cos) = (angle * 0.5).sin_cos();
        Self {
            s: cos,
            e14: sin,
            ..Self::IDENTITY
        }
    }

    pub fn rotation_yz(angle: f32) -> Self {
        let (sin, cos) = (angle * 0.5).sin_cos();
        Self {
            s: cos,
            e23: sin,
            ..Self::IDENTITY
        }
    }

    pub fn rotation_yw(angle: f32) -> Self {
        let (sin, cos) = (angle * 0.5).sin_cos();
        Self {
            s: cos,
            e24: -sin,
            ..Self::IDENTITY
        }
    }

    pub fn rotation_zw(angle: f32) -> Self {
        let (sin, cos) = (angle * 0.5).sin_cos();
        Self {
            s: cos,
            e34: sin,
            ..Self::IDENTITY
        }
    }

    pub fn magnitude_squared(self) -> f32 {
        (!self * self).s
    }

    pub fn magnitude(self) -> f32 {
        self.magnitude_squared().sqrt()
    }

    pub fn normalized(self) -> Self {
        let inverse_magnitude = self.magnitude().recip();
        let Self {
            s,
            e12,
            e13,
            e14,
            e23,
            e24,
            e34,
            e1234,
        } = self;
        Self {
            s: s * inverse_magnitude,
            e12: e12 * inverse_magnitude,
            e13: e13 * inverse_magnitude,
            e14: e14 * inverse_magnitude,
            e23: e23 * inverse_magnitude,
            e24: e24 * inverse_magnitude,
            e34: e34 * inverse_magnitude,
            e1234: e1234 * inverse_magnitude,
        }
    }

    pub fn rotate(self, normal: cgmath::Vector4<f32>) -> cgmath::Vector4<f32> {
        let Self {
            s: a,
            e12: f,
            e13: g,
            e14: h,
            e23: i,
            e24: j,
            e34: k,
            e1234: p,
        } = self;
        let cgmath::Vector4 {
            x: p3,
            y: p2,
            z: p1,
            w: p0,
        } = normal;
        let ap2 = a * p2;
        let fp3 = f * p3;
        let ip1 = i * p1;
        let jp0 = j * p0;
        let ap3 = a * p3;
        let fp2 = f * p2;
        let gp1 = g * p1;
        let hp0 = h * p0;
        let ap1 = a * p1;
        let kp0 = k * p0;
        let gp3 = g * p3;
        let ip2 = i * p2;
        let ap0 = a * p0;
        let kp1 = k * p1;
        let hp3 = h * p3;
        let jp2 = j * p2;
        let s0 = ip1 - ap2 - fp3 - jp0;
        let s1 = ap3 + gp1 - fp2 - hp0;
        let s2 = ap1 + ip2 - kp0 - gp3;
        let s3 = jp2 - ap0 - kp1 - hp3;
        let [w, z, y, x] = [
            p0 + 2.0 * (p * (f * p1 + g * p2 + i * p3 - p * p0) + j * s0 + h * s1 + k * s2),
            p1 + 2.0 * (p * (h * p2 + j * p3 - p * p1 - f * p0) + k * s3 - i * s0 - g * s1),
            p2 + 2.0 * (p * (k * p3 - p * p2 - g * p0 - h * p1) + f * s1 - j * s3 - i * s2),
            p3 + 2.0 * (h * s3 + g * s2 + f * s0 - p * (k * p2 + p * p3 + i * p0 + j * p1)),
        ];
        cgmath::Vector4 { x, y, z, w }
    }
}

impl Not for Rotor {
    type Output = Self;

    fn not(self) -> Self::Output {
        let Self {
            s,
            e12,
            e13,
            e14,
            e23,
            e24,
            e34,
            e1234,
        } = self;
        Self {
            s,
            e12: -e12,
            e13: -e13,
            e14: -e14,
            e23: -e23,
            e24: -e24,
            e34: -e34,
            e1234,
        }
    }
}

impl Mul<Self> for Rotor {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let Self {
            s: a1,
            e12: g1,
            e13: h1,
            e14: i1,
            e23: j1,
            e24: k1,
            e34: l1,
            e1234: q1,
        } = self;
        let Self {
            s: a2,
            e12: g2,
            e13: h2,
            e14: i2,
            e23: j2,
            e24: k2,
            e34: l2,
            e1234: q2,
        } = rhs;
        Self {
            s: -g1 * g2 + -h1 * h2 + -i1 * i2 + -j1 * j2 + -k1 * k2 + -l1 * l2 + a1 * a2 + q1 * q2,
            e12: -h1 * j2 + -i1 * k2 + -l1 * q2 + -l2 * q1 + a1 * g2 + a2 * g1 + h2 * j1 + i2 * k1,
            e13: -g2 * j1 + -i1 * l2 + a1 * h2 + a2 * h1 + g1 * j2 + i2 * l1 + k1 * q2 + k2 * q1,
            e14: -g2 * k1 + -h2 * l1 + -j1 * q2 + -j2 * q1 + a1 * i2 + a2 * i1 + g1 * k2 + h1 * l2,
            e23: -g1 * h2 + -i1 * q2 + -i2 * q1 + -k1 * l2 + a1 * j2 + a2 * j1 + g2 * h1 + k2 * l1,
            e24: -g1 * i2 + -j2 * l1 + a1 * k2 + a2 * k1 + g2 * i1 + h1 * q2 + h2 * q1 + j1 * l2,
            e34: -g1 * q2 + -g2 * q1 + -h1 * i2 + -j1 * k2 + a1 * l2 + a2 * l1 + h2 * i1 + j2 * k1,
            e1234: -h1 * k2 + -h2 * k1 + a1 * q2 + a2 * q1 + g1 * l2 + g2 * l1 + i1 * j2 + i2 * j1,
        }
    }
}
