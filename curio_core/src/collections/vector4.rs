use crate::collections::vector2::Vector2;
use crate::collections::vector2_int::Vector2Int;
use crate::collections::vector3::Vector3;
use crate::collections::vector3_int::Vector3Int;
use crate::collections::vector4_int::Vector4Int;
use serde::Serialize;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result;
use std::ops::{Add, Div, Mul, Sub};

/// A 4D Vector backed by f32
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Default)]
pub struct Vector4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}
// const constructors
impl Vector4 {
    // Creates a new Vector4 with provided x,y,z,w
    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Vector4 {
        Vector4 { x, y, z, w }
    }
    // Creates a new Vector4 with 0,0,0,0
    pub const fn zero() -> Vector4 {
        Vector4::new(0.0, 0.0, 0.0, 0.0)
    }
    // Creates a new Vector4 with 1,1,1,1
    pub const fn one() -> Vector4 {
        Vector4::new(1.0, 1.0, 1.0, 0.0)
    }
}
impl Vector4 {
    // Returns the size of the vector based on x,y,z,w
    pub fn magnitude(self) -> f32 {
        ((self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w) as f32).sqrt()
    }
    /// Normalizes all the values from 0-1 of THIS instance
    pub fn normalize(&mut self) {
        let mag = self.magnitude();
        if mag == 0.0 {
            panic!("Cannot normalize a zero-length vector");
        }

        self.x = (self.x as f32 / mag) as f32;
        self.y = (self.y as f32 / mag) as f32;
        self.z = (self.z as f32 / mag) as f32;
        self.w = (self.w as f32 / mag) as f32;
    }
    /// Normalizes all the values from 0-1 and returns a NEW instance
    pub fn normalize_and_copy(&self) -> Vector4 {
        let mag = self.magnitude();
        if mag == 0.0 {
            panic!("Cannot normalize a zero-length vector");
        }
        Vector4 {
            x: (self.x as f32 / mag) as f32,
            y: (self.y as f32 / mag) as f32,
            z: (self.z as f32 / mag) as f32,
            w: (self.w as f32 / mag) as f32,
        }
    }
    /// Clamps the x value of THIS instance between min and max inclusively
    pub fn clamp_x(&mut self, min: f32, max: f32) {
        self.x = self.x.clamp(min, max);
    }
    /// Clamps the y value of THIS instance between min and max inclusively
    pub fn clamp_y(&mut self, min: f32, max: f32) {
        self.y = self.y.clamp(min, max);
    }
    /// Clamps the z value of THIS instance between min and max inclusively
    pub fn clamp_z(&mut self, min: f32, max: f32) {
        self.z = self.z.clamp(min, max);
    }
    /// Clamps the z value of THIS instance between min and max inclusively
    pub fn clamp_w(&mut self, min: f32, max: f32) {
        self.w = self.w.clamp(min, max);
    }
    /// Clamps the x value of a NEW instance between min and max inclusively
    pub fn clamp_x_and_copy(&self, min: f32, max: f32) -> Vector4 {
        Vector4::new(self.x.clamp(min, max), self.y, self.z, self.w)
    }
    /// Clamps the y value of a NEW instance between min and max inclusively
    pub fn clamp_y_and_copy(&self, min: f32, max: f32) -> Vector4 {
        Vector4::new(self.x, self.y.clamp(min, max), self.z, self.w)
    }
    /// Clamps the z value of a NEW instance between min and max inclusively
    pub fn clamp_z_and_copy(&self, min: f32, max: f32) -> Vector4 {
        Vector4::new(self.x, self.y, self.z.clamp(min, max), self.w)
    }
    /// Clamps the z value of a NEW instance between min and max inclusively
    pub fn clamp_w_and_copy(&self, min: f32, max: f32) -> Vector4 {
        Vector4::new(self.x, self.y, self.z, self.w.clamp(min, max))
    }
    /// Clamps all values of THIS instance between min and max inclusively
    pub fn clamped(&mut self, min: Vector4, max: Vector4) {
        self.x = self.x.clamp(min.x, max.x);
        self.y = self.y.clamp(min.y, max.y);
        self.z = self.z.clamp(min.z, max.z);
    }
    /// Clamps all values of a NEW instance between min and max inclusively
    pub fn clamp_and_copy(&self, min: Vector4, max: Vector4) -> Vector4 {
        Vector4::new(self.x.clamp(min.x, max.x), self.y.clamp(min.y, max.y), self.z.clamp(min.z, max.z), self.w.clamp(min.w, max.w))
    }
    /// returns a new instance of Vector2 discarding z and w
    pub fn to_vector2(&self) -> Vector2 {
        Vector2::new(self.x, self.y)
    }
    /// returns a new instance of Vector3 discarding w
    pub fn to_vector3(&self) -> Vector3 {
        Vector3::new(self.x, self.y, self.z)
    }
    /// returns a new instance of Vector2Int discarding z and w
    pub fn to_vector2_int(&self) -> Vector2Int {
        Vector2Int::new(self.x.round() as i32, self.y.round() as i32)
    }
    /// returns a new instance of Vector3Int discarding w
    pub fn to_vector3_int(&self) -> Vector3Int {
        Vector3Int::new(self.x.round() as i32, self.y.round() as i32, self.z.round() as i32)
    }
    /// returns a new instance of Vector4Int
    pub fn to_vector4_int(&self) -> Vector4Int {
        Vector4Int::new(self.x as i32, self.y as i32, self.z as i32, self.w as i32)
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
// Display
impl Display for Vector4 {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "Vector4({}, {}, {}, {})", self.x, self.y, self.z, self.w)
    }
}
