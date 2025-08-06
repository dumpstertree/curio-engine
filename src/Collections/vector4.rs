use core::fmt;
use std::ops::{Add, Div, Mul, Sub};


use crate::Collections::vector3::Vector3;
#[derive(Clone, Copy, PartialEq)]
pub struct Vector4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}
impl Vector4 {
    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Vector4 {
        Vector4 { x, y, z, w }
    }
    pub fn zero() -> Vector4 {
        Vector4::new(0.0, 0.0, 0.0, 0.0)
    }
    pub fn one() -> Vector4 {
        Vector4::new(1.0, 1.0, 1.0, 0.0)
    }
    pub fn new_from_vec3(xyz: Vector3, w: f32) -> Vector4 {
        Vector4::new(xyz.x, xyz.y, xyz.z, w)
    }
}

// whole num mult
impl Mul<f32> for Vector4 {
    type Output = Vector4;
    fn mul(self, x: f32) -> Vector4 {
        Vector4 {
            x: self.x * x,
            y: self.y * x,
            z: self.z * x,
            w: self.w * x,
        }
    }
}
// whole num divide
impl Div<f32> for Vector4 {
    type Output = Vector4;
    fn div(self, x: f32) -> Vector4 {
        Vector4 {
            x: self.x / x,
            y: self.y / x,
            z: self.z / x,
            w: self.w / x,
        }
    }
}
// vector add
impl Add<Vector4> for Vector4 {
    type Output = Vector4;
    fn add(self, x: Vector4) -> Vector4 {
        Vector4 {
            x: self.x + x.x,
            y: self.y + x.y,
            z: self.z + x.z,
            w: self.z + x.w,
        }
    }
}
// vector subtract
impl Sub<Vector4> for Vector4 {
    type Output = Vector4;
    fn sub(self, x: Vector4) -> Vector4 {
        Vector4 {
            x: self.x - x.x,
            y: self.y - x.y,
            z: self.z - x.z,
            w: self.w - x.w,
        }
    }
}

impl fmt::Display for Vector4 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Vector4({}, {}, {}, {})", self.x, self.y, self.z, self.w)
    }
}
