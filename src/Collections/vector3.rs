use core::fmt;
use std::ops::{Add, Div, Mul, Sub};
#[derive(Clone, PartialEq)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
impl Vector3 {
    pub fn new(x: f32, y: f32, z: f32) -> Vector3 {
        Vector3 { x, y, z }
    }
    pub fn zero() -> Vector3 {
        Vector3::new(0.0, 0.0, 0.0)
    }
    pub fn one() -> Vector3 {
        Vector3::new(1.0, 1.0, 1.0)
    }
    pub fn forward() -> Vector3 {
        Vector3::new(0.0, 0.0, 1.0)
    }
    pub fn back() -> Vector3 {
        Vector3::new(0.0, 0.0, -1.0)
    }
    pub fn left() -> Vector3 {
        Vector3::new(1.0, 0.0, 0.0)
    }
    pub fn right() -> Vector3 {
        Vector3::new(-1.0, 0.0, 0.0)
    }
    pub fn up() -> Vector3 {
        Vector3::new(0.0, 1.0, 0.0)
    }
    pub fn down() -> Vector3 {
        Vector3::new(0.0, -1.0, 0.0)
    }
    pub fn to_cg_math(self) -> cgmath::Vector3<f32> {
        cgmath::Vector3 {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }
}

// whole num mult
impl Mul<f32> for Vector3 {
    type Output = Vector3;
    fn mul(self, x: f32) -> Vector3 {
        Vector3 {
            x: self.x * x,
            y: self.y * x,
            z: self.z * x,
        }
    }
}
// whole num divide
impl Div<f32> for Vector3 {
    type Output = Vector3;
    fn div(self, x: f32) -> Vector3 {
        Vector3 {
            x: self.x / x,
            y: self.y / x,
            z: self.z / x,
        }
    }
}
// vector add
impl Add<Vector3> for Vector3 {
    type Output = Vector3;
    fn add(self, x: Vector3) -> Vector3 {
        Vector3 {
            x: self.x + x.x,
            y: self.y + x.y,
            z: self.z + x.z,
        }
    }
}
// vector subtract
impl Sub<Vector3> for Vector3 {
    type Output = Vector3;
    fn sub(self, x: Vector3) -> Vector3 {
        Vector3 {
            x: self.x - x.x,
            y: self.y - x.y,
            z: self.z - x.z,
        }
    }
}

impl fmt::Display for Vector3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Vector3({}, {}, {})", self.x, self.y, self.z)
    }
}
