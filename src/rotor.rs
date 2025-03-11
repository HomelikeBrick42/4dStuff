use encase::ShaderType;
use std::ops::{Mul, Not};

#[derive(Debug, Clone, Copy, ShaderType)]
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
            e12: b,
            e13: c,
            e14: d,
            e23: e,
            e24: f,
            e34: g,
            e1234: h,
        } = self;
        let cgmath::Vector4 {
            x: p3,
            y: p2,
            z: p1,
            w: p0,
        } = normal;
        let ap2 = a * p2;
        let bp3 = b * p3;
        let ep1 = e * p1;
        let fp0 = f * p0;
        let ap3 = a * p3;
        let bp2 = b * p2;
        let cp1 = c * p1;
        let dp0 = d * p0;
        let ap1 = a * p1;
        let gp0 = g * p0;
        let cp3 = c * p3;
        let ep2 = e * p2;
        let ap0 = a * p0;
        let gp1 = g * p1;
        let dp3 = d * p3;
        let fp2 = f * p2;
        let s0 = ep1 - ap2 - bp3 - fp0;
        let s1 = ap3 + cp1 - bp2 - dp0;
        let s2 = ap1 + ep2 - gp0 - cp3;
        let s3 = fp2 - ap0 - gp1 - dp3;
        let [w, z, y, x] = [
            p0 + 2.0 * (h * (b * p1 + c * p2 + e * p3 - h * p0) + f * s0 + d * s1 + g * s2),
            p1 + 2.0 * (h * (d * p2 + f * p3 - h * p1 - b * p0) + g * s3 - e * s0 - c * s1),
            p2 + 2.0 * (h * (g * p3 - h * p2 - c * p0 - d * p1) + b * s1 - f * s3 - e * s2),
            p3 + 2.0 * (d * s3 + c * s2 + b * s0 - h * (g * p2 + h * p3 + e * p0 + f * p1)),
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
            e12: b1,
            e13: c1,
            e14: d1,
            e23: e1,
            e24: f1,
            e34: g1,
            e1234: h1,
        } = self;
        let Self {
            s: a2,
            e12: b2,
            e13: c2,
            e14: d2,
            e23: e2,
            e24: f2,
            e34: g2,
            e1234: h2,
        } = rhs;
        Self {
            s: -b1 * b2 + -c1 * c2 + -d1 * d2 + -e1 * e2 + -f1 * f2 + -g1 * g2 + a1 * a2 + h1 * h2,
            e12: -c1 * e2 + -d1 * f2 + -g1 * h2 + -g2 * h1 + a1 * b2 + a2 * b1 + c2 * e1 + d2 * f1,
            e13: -b2 * e1 + -d1 * g2 + a1 * c2 + a2 * c1 + b1 * e2 + d2 * g1 + f1 * h2 + f2 * h1,
            e14: -b2 * f1 + -c2 * g1 + -e1 * h2 + -e2 * h1 + a1 * d2 + a2 * d1 + b1 * f2 + c1 * g2,
            e23: -b1 * c2 + -d1 * h2 + -d2 * h1 + -f1 * g2 + a1 * e2 + a2 * e1 + b2 * c1 + f2 * g1,
            e24: -b1 * d2 + -e2 * g1 + a1 * f2 + a2 * f1 + b2 * d1 + c1 * h2 + c2 * h1 + e1 * g2,
            e34: -b1 * h2 + -b2 * h1 + -c1 * d2 + -e1 * f2 + a1 * g2 + a2 * g1 + c2 * d1 + e2 * f1,
            e1234: -c1 * f2 + -c2 * f1 + a1 * h2 + a2 * h1 + b1 * g2 + b2 * g1 + d1 * e2 + d2 * e1,
        }
    }
}
