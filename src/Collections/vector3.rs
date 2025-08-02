use core::fmt;
use std::ops::{Add, Div, Mul, Sub};

use cgmath::Point3;
use serde::Serialize;
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
impl Vector3 {
    pub const fn new(x: f32, y: f32, z: f32) -> Vector3 {
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
    pub fn magnitude(self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    pub fn dot(lhs: Vector3, rhs: Vector3) -> f32 {
        lhs.x * rhs.x + lhs.y * rhs.y + lhs.z * rhs.z
    }
    pub fn cross(lhs: Vector3, rhs: Vector3) -> Vector3 {
        Vector3::new(
            lhs.y * rhs.z - lhs.z * rhs.y,
            lhs.z * rhs.x - lhs.x * rhs.z,
            lhs.x * rhs.y - lhs.y * rhs.x,
        )
    }
    pub fn reflect(direction: Vector3, normal: Vector3) -> Vector3 {
        let factor = Vector3::dot(normal, direction) * -2.0;
        Vector3::new(
            factor * normal.x + direction.x,
            factor * normal.y + direction.y,
            factor * normal.z + direction.z,
        )
    }

    // Normalize the vector
    pub fn normalize(&mut self) {
        let mag = self.magnitude();
        if mag == 0.0 {
            panic!("Cannot normalize a zero-length vector");
        }

        self.x = self.x / mag;
        self.y = self.y / mag;
        self.z = self.z / mag;
    }
    pub fn normalized(&self) -> Vector3 {
        let mag = self.magnitude();
        if mag == 0.0 {
            panic!("Cannot normalize a zero-length vector");
        }
        Vector3 {
            x: self.x / mag,
            y: self.y / mag,
            z: self.z / mag,
        }
    }
    pub fn clamp_x(&self, min: f32, max: f32) -> Vector3 {
        Vector3::new(self.x.clamp(min, max), self.y, self.z)
    }
    pub fn clamped_x(&mut self, min: f32, max: f32) {
        self.x = self.x.clamp(min, max);
    }

    pub fn clamp(&self, min: Vector3, max: Vector3) -> Vector3 {
        Vector3::new(self.x.clamp(min.x, max.x), self.y.clamp(min.y, max.y), self.z.clamp(min.z, max.z))
    }
    pub fn clamped(&mut self, min: Vector3, max: Vector3) {
        self.x = self.x.clamp(min.x, max.x);
        self.y = self.y.clamp(min.y, max.y);
        self.z = self.z.clamp(min.z, max.z);
    }

    pub fn to_cg_math(self) -> cgmath::Vector3<f32> {
        cgmath::Vector3 {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }
    pub fn to_point3(self) -> Point3<f32> {
        Point3 {
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
