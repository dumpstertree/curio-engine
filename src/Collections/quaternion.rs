use core::fmt;
use std::{f32::consts::PI, ops::Mul};

use crate::Collections::vector3::Vector3;

#[derive(Clone, Copy)]
pub struct Quaternion {
    x: f32,
    y: f32,
    z: f32,
    w: f32,
}

impl Quaternion {
    fn rotate_vector(&self, v: Vector3) -> Vector3 {
        let q = *self;

        // Quaternion multiplication: q * v * q_conjugate
        let q_vec = Vector3 { x: q.x, y: q.y, z: q.z };
        let uv = Vector3::cross(q_vec, v);
        let uuv = Vector3::cross(q_vec, uv);

        let uv = uv * 2.0 * q.w;
        let uuv = uuv * 2.0;

        v + uv + uuv
    }
    pub fn look_rotation(forward: Vector3, up: Vector3) -> Quaternion {
        let f = forward.normalized();
        let u = up.normalized();
        let r = Vector3::cross(u, f).normalized();
        let u = Vector3::cross(f, r); // ensure orthogonal up

        // Construct quaternion from basis vectors directly
        // using direction vectors' rotation alignment method
        let trace = r.x + u.y + f.z;

        if trace > 0.0 {
            let s = (trace + 1.0).sqrt() * 2.0;
            let inv_s = 1.0 / s;
            Quaternion::new((u.z - f.y) * inv_s, (f.x - r.z) * inv_s, (r.y - u.x) * inv_s, 0.25 * s)
        } else if r.x > u.y && r.x > f.z {
            let s = (1.0 + r.x - u.y - f.z).sqrt() * 2.0;
            let inv_s = 1.0 / s;
            Quaternion::new(0.25 * s, (r.y + u.x) * inv_s, (f.x + r.z) * inv_s, (u.z - f.y) * inv_s)
        } else if u.y > f.z {
            let s = (1.0 + u.y - r.x - f.z).sqrt() * 2.0;
            let inv_s = 1.0 / s;
            Quaternion::new((r.y + u.x) * inv_s, 0.25 * s, (u.z + f.y) * inv_s, (f.x - r.z) * inv_s)
        } else {
            let s = (1.0 + f.z - r.x - u.y).sqrt() * 2.0;
            let inv_s = 1.0 / s;
            Quaternion::new((f.x + r.z) * inv_s, (u.z + f.y) * inv_s, 0.25 * s, (r.y - u.x) * inv_s)
        }
    }
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Quaternion {
        Quaternion { x: x, y: y, z: z, w: w }
    }
    pub fn identity() -> Quaternion {
        Quaternion::new(0.0, 0.0, 0.0, 1.0)
    }
    pub fn zero() -> Quaternion {
        Quaternion::new(0.0, 0.0, 0.0, 10.0)
    }
    pub fn to_euler(&self) -> Vector3 {
        let sinr_cosp = 2.0 * (self.w * self.x + self.y * self.z);
        let cosr_cosp = 1.0 - 2.0 * (self.x * self.x + self.y * self.y);
        let roll = sinr_cosp.atan2(cosr_cosp);

        let sinp = 2.0 * (self.w * self.y - self.z * self.x);
        let pitch = if sinp.abs() >= 1.0 {
            // use 90 degrees if out of range
            sinp.signum() * PI / 2.0
        } else {
            sinp.asin()
        };

        let siny_cosp = 2.0 * (self.w * self.z + self.x * self.y);
        let cosy_cosp = 1.0 - 2.0 * (self.y * self.y + self.z * self.z);
        let yaw = siny_cosp.atan2(cosy_cosp);

        Vector3::new(pitch.to_degrees(), yaw.to_degrees(), roll.to_degrees())
    }
    pub fn from_euler(euler: Vector3) -> Quaternion {
        let cr: f32 = f32::cos(euler.x * 0.5);
        let sr: f32 = f32::sin(euler.x * 0.5);
        let cp: f32 = f32::cos(euler.y * 0.5);
        let sp: f32 = f32::sin(euler.y * 0.5);
        let cy: f32 = f32::cos(euler.z * 0.5);
        let sy: f32 = f32::sin(euler.z * 0.5);

        Quaternion::new(
            cr * cp * cy + sr * sp * sy,
            sr * cp * cy - cr * sp * sy,
            cr * sp * cy + sr * cp * sy,
            cr * cp * sy - sr * sp * cy,
        )
    }
    pub fn from_angle_axis(axis: Vector3, angle: f32) -> Quaternion {
        if angle == 0.0 {
            return Quaternion::identity();
        }
        Quaternion {
            x: axis.x * f32::sin(angle / 2.0),
            y: axis.y * f32::sin(angle / 2.0),
            z: axis.z * f32::sin(angle / 2.0),
            w: f32::cos(angle / 2.0),
        }
    }
    pub fn to_cg_math(&self) -> cgmath::Quaternion<f32> {
        cgmath::Quaternion::new(self.w, self.x, self.y, self.z)
    }
}

// whole num mult
impl Mul<Quaternion> for Quaternion {
    type Output = Quaternion;
    fn mul(self, other: Quaternion) -> Quaternion {
        Quaternion {
            w: self.w * other.w - self.x * other.x - self.y * other.y - self.z * other.z,
            x: self.w * other.x + self.x * other.w + self.y * other.z - self.z * other.y,
            y: self.w * other.y - self.x * other.z + self.y * other.w + self.z * other.x,
            z: self.w * other.z + self.x * other.y - self.y * other.x + self.z * other.w,
        }
    }
}
impl Mul<Vector3> for Quaternion {
    type Output = Vector3;
    fn mul(self, v: Vector3) -> Vector3 {
        let q: Quaternion = self;

        // Quaternion multiplication: q * v * q_conjugate
        let q_vec = Vector3 { x: q.x, y: q.y, z: q.z };
        let uv = Vector3::cross(q_vec, v);
        let uuv = Vector3::cross(q_vec, uv);

        let uv = uv * 2.0 * q.w;
        let uuv = uuv * 2.0;

        v + uv + uuv
    }
}

impl fmt::Display for Quaternion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Quaternion({}, {}, {}, {})", self.x, self.y, self.z, self.w)
    }
}
