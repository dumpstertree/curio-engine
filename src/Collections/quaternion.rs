use core::fmt;
use std::ops::Mul;

use crate::Collections::vector3::Vector3;

#[derive(Clone, Copy)]
pub struct Quaternion {
    x: f32,
    y: f32,
    z: f32,
    w: f32,
}

impl Quaternion {
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Quaternion {
        Quaternion { x: x, y: y, z: z, w: w }
    }
    pub fn identity() -> Quaternion {
        Quaternion::new(0.0, 0.0, 0.0, 1.0)
    }
    pub fn zero() -> Quaternion {
        Quaternion::new(0.0, 0.0, 0.0, 10.0)
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

impl fmt::Display for Quaternion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Quaternion({}, {}, {}, {})", self.x, self.y, self.z, self.w)
    }
}
