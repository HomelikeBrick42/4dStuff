use encase::ShaderType;
use std::ops::{Mul, Not};

#[derive(Debug, Clone, Copy, ShaderType)]
pub struct Transform {
    pub s: f32,
    pub e01: f32,
    pub e02: f32,
    pub e03: f32,
    pub e04: f32,
    pub e12: f32,
    pub e13: f32,
    pub e14: f32,
    pub e23: f32,
    pub e24: f32,
    pub e34: f32,
    pub e0123: f32,
    pub e0124: f32,
    pub e0134: f32,
    pub e0234: f32,
    pub e1234: f32,
}

impl Transform {
    pub const IDENTITY: Self = Self {
        s: 1.0,
        e01: 0.0,
        e02: 0.0,
        e03: 0.0,
        e04: 0.0,
        e12: 0.0,
        e13: 0.0,
        e14: 0.0,
        e23: 0.0,
        e24: 0.0,
        e34: 0.0,
        e0123: 0.0,
        e0124: 0.0,
        e0134: 0.0,
        e0234: 0.0,
        e1234: 0.0,
    };

    pub fn translation(offset: cgmath::Vector4<f32>) -> Self {
        Self {
            e01: offset.x * 0.5,
            e02: offset.y * -0.5,
            e03: offset.z * 0.5,
            e04: offset.w * -0.5,
            ..Self::IDENTITY
        }
    }

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
            e01,
            e02,
            e03,
            e04,
            e12,
            e13,
            e14,
            e23,
            e24,
            e34,
            e0123,
            e0124,
            e0134,
            e0234,
            e1234,
        } = self;
        Self {
            s: s * inverse_magnitude,
            e01: e01 * inverse_magnitude,
            e02: e02 * inverse_magnitude,
            e03: e03 * inverse_magnitude,
            e04: e04 * inverse_magnitude,
            e12: e12 * inverse_magnitude,
            e13: e13 * inverse_magnitude,
            e14: e14 * inverse_magnitude,
            e23: e23 * inverse_magnitude,
            e24: e24 * inverse_magnitude,
            e34: e34 * inverse_magnitude,
            e0123: e0123 * inverse_magnitude,
            e0124: e0124 * inverse_magnitude,
            e0134: e0134 * inverse_magnitude,
            e0234: e0234 * inverse_magnitude,
            e1234: e1234 * inverse_magnitude,
        }
    }

    pub fn transform(self, point: cgmath::Vector4<f32>) -> cgmath::Vector4<f32> {
        let Self {
            s: a,
            e01: b,
            e02: c,
            e03: d,
            e04: f,
            e12: g,
            e13: h,
            e14: i,
            e23: j,
            e24: k,
            e34: l,
            e0123: m,
            e0124: n,
            e0134: o,
            e0234: p,
            e1234: q,
        } = self;
        let cgmath::Vector4 {
            x: p3,
            y: p2,
            z: p1,
            w: p0,
        } = point;
        let ap2 = a * p2;
        let gp3 = g * p3;
        let jp1 = j * p1;
        let kp0 = k * p0;
        let ap3 = a * p3;
        let gp2 = g * p2;
        let hp1 = h * p1;
        let ip0 = i * p0;
        let ap1 = a * p1;
        let lp0 = l * p0;
        let hp3 = h * p3;
        let jp2 = j * p2;
        let ap0 = a * p0;
        let lp1 = l * p1;
        let ip3 = i * p3;
        let kp2 = k * p2;
        let s0 = c + jp1 - ap2 - gp3 - kp0;
        let s1 = ap3 + b + hp1 - gp2 - ip0;
        let s2 = ap1 + d + jp2 - lp0 - hp3;
        let s3 = f + kp2 - ap0 - lp1 - ip3;
        let mut result = [
            p0 + 2.0
                * (q * (m + g * p1 + h * p2 + j * p3 - q * p0) + k * s0 + i * s1 + l * s2
                    - a * f
                    - n * g
                    - o * h
                    - p * j),
            p1 + 2.0
                * (a * d + m * g + q * (n + i * p2 + k * p3 - q * p1 - g * p0) + l * s3
                    - o * i
                    - p * k
                    - j * s0
                    - h * s1),
            p2 + 2.0
                * (m * h + n * i + q * (l * p3 + o - q * p2 - h * p0 - i * p1) + g * s1
                    - a * c
                    - l * p
                    - k * s3
                    - j * s2),
            p3 + 2.0
                * (a * b
                    + l * o
                    + m * j
                    + n * k
                    + q * (p - l * p2 - q * p3 - j * p0 - k * p1)
                    + i * s3
                    + h * s2
                    + g * s0),
        ];
        result.reverse();
        let [x, y, z, w] = result;
        cgmath::Vector4 { x, y, z, w }
    }

    pub fn transform_direction(self, normal: cgmath::Vector4<f32>) -> cgmath::Vector4<f32> {
        let Self {
            s: a,
            e01: _,
            e02: _,
            e03: _,
            e04: _,
            e12: f,
            e13: g,
            e14: h,
            e23: i,
            e24: j,
            e34: k,
            e0123: _,
            e0124: _,
            e0134: _,
            e0234: _,
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
        let mut result = [
            p0 + 2.0 * (p * (f * p1 + g * p2 + i * p3 - p * p0) + j * s0 + h * s1 + k * s2),
            p1 + 2.0 * (p * (h * p2 + j * p3 - p * p1 - f * p0) + k * s3 - i * s0 - g * s1),
            p2 + 2.0 * (p * (k * p3 - p * p2 - g * p0 - h * p1) + f * s1 - j * s3 - i * s2),
            p3 + 2.0 * (h * s3 + g * s2 + f * s0 - p * (k * p2 + p * p3 + i * p0 + j * p1)),
        ];
        result.reverse();
        let [x, y, z, w] = result;
        cgmath::Vector4 { x, y, z, w }
    }
}

impl Not for Transform {
    type Output = Self;

    fn not(self) -> Self::Output {
        let Self {
            s,
            e01,
            e02,
            e03,
            e04,
            e12,
            e13,
            e14,
            e23,
            e24,
            e34,
            e0123,
            e0124,
            e0134,
            e0234,
            e1234,
        } = self;
        Self {
            s,
            e01: -e01,
            e02: -e02,
            e03: -e03,
            e04: -e04,
            e12: -e12,
            e13: -e13,
            e14: -e14,
            e23: -e23,
            e24: -e24,
            e34: -e34,
            e0123,
            e0124,
            e0134,
            e0234,
            e1234,
        }
    }
}

impl Mul<Self> for Transform {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let Self {
            s: a1,
            e01: b1,
            e02: c1,
            e03: d1,
            e04: f1,
            e12: g1,
            e13: h1,
            e14: i1,
            e23: j1,
            e24: k1,
            e34: l1,
            e0123: m1,
            e0124: n1,
            e0134: o1,
            e0234: p1,
            e1234: q1,
        } = self;
        let Self {
            s: a2,
            e01: b2,
            e02: c2,
            e03: d2,
            e04: f2,
            e12: g2,
            e13: h2,
            e14: i2,
            e23: j2,
            e24: k2,
            e34: l2,
            e0123: m2,
            e0124: n2,
            e0134: o2,
            e0234: p2,
            e1234: q2,
        } = rhs;
        Self {
            s: -g1 * g2 + -h1 * h2 + -i1 * i2 + -j1 * j2 + -k1 * k2 + -l1 * l2 + a1 * a2 + q1 * q2,
            e01: -c1 * g2
                + -d1 * h2
                + -f1 * i2
                + -j1 * m2
                + -j2 * m1
                + -k1 * n2
                + -k2 * n1
                + -l1 * o2
                + -l2 * o1
                + -p2 * q1
                + a1 * b2
                + a2 * b1
                + c2 * g1
                + d2 * h1
                + f2 * i1
                + p1 * q2,
            e02: -b2 * g1
                + -d1 * j2
                + -f1 * k2
                + -l1 * p2
                + -l2 * p1
                + -o1 * q2
                + a1 * c2
                + a2 * c1
                + b1 * g2
                + d2 * j1
                + f2 * k1
                + h1 * m2
                + h2 * m1
                + i1 * n2
                + i2 * n1
                + o2 * q1,
            e03: -b2 * h1
                + -c2 * j1
                + -f1 * l2
                + -g1 * m2
                + -g2 * m1
                + -n2 * q1
                + a1 * d2
                + a2 * d1
                + b1 * h2
                + c1 * j2
                + f2 * l1
                + i1 * o2
                + i2 * o1
                + k1 * p2
                + k2 * p1
                + n1 * q2,
            e04: -b2 * i1
                + -c2 * k1
                + -d2 * l1
                + -g1 * n2
                + -g2 * n1
                + -h1 * o2
                + -h2 * o1
                + -j1 * p2
                + -j2 * p1
                + -m1 * q2
                + a1 * f2
                + a2 * f1
                + b1 * i2
                + c1 * k2
                + d1 * l2
                + m2 * q1,
            e12: -h1 * j2 + -i1 * k2 + -l1 * q2 + -l2 * q1 + a1 * g2 + a2 * g1 + h2 * j1 + i2 * k1,
            e13: -g2 * j1 + -i1 * l2 + a1 * h2 + a2 * h1 + g1 * j2 + i2 * l1 + k1 * q2 + k2 * q1,
            e14: -g2 * k1 + -h2 * l1 + -j1 * q2 + -j2 * q1 + a1 * i2 + a2 * i1 + g1 * k2 + h1 * l2,
            e23: -g1 * h2 + -i1 * q2 + -i2 * q1 + -k1 * l2 + a1 * j2 + a2 * j1 + g2 * h1 + k2 * l1,
            e24: -g1 * i2 + -j2 * l1 + a1 * k2 + a2 * k1 + g2 * i1 + h1 * q2 + h2 * q1 + j1 * l2,
            e34: -g1 * q2 + -g2 * q1 + -h1 * i2 + -j1 * k2 + a1 * l2 + a2 * l1 + h2 * i1 + j2 * k1,
            e0123: -c1 * h2
                + -c2 * h1
                + -f1 * q2
                + -i2 * p1
                + -k1 * o2
                + -l2 * n1
                + a1 * m2
                + a2 * m1
                + b1 * j2
                + b2 * j1
                + d1 * g2
                + d2 * g1
                + f2 * q1
                + i1 * p2
                + k2 * o1
                + l1 * n2,
            e0124: -c1 * i2
                + -c2 * i1
                + -d2 * q1
                + -h1 * p2
                + -j2 * o1
                + -l1 * m2
                + a1 * n2
                + a2 * n1
                + b1 * k2
                + b2 * k1
                + d1 * q2
                + f1 * g2
                + f2 * g1
                + h2 * p1
                + j1 * o2
                + l2 * m1,
            e0134: -c1 * q2
                + -d1 * i2
                + -d2 * i1
                + -g2 * p1
                + -j1 * n2
                + -k2 * m1
                + a1 * o2
                + a2 * o1
                + b1 * l2
                + b2 * l1
                + c2 * q1
                + f1 * h2
                + f2 * h1
                + g1 * p2
                + j2 * n1
                + k1 * m2,
            e0234: -b2 * q1
                + -d1 * k2
                + -d2 * k1
                + -g1 * o2
                + -h2 * n1
                + -i1 * m2
                + a1 * p2
                + a2 * p1
                + b1 * q2
                + c1 * l2
                + c2 * l1
                + f1 * j2
                + f2 * j1
                + g2 * o1
                + h1 * n2
                + i2 * m1,
            e1234: -h1 * k2 + -h2 * k1 + a1 * q2 + a2 * q1 + g1 * l2 + g2 * l1 + i1 * j2 + i2 * j1,
        }
    }
}
