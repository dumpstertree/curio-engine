use crate::collections::vector2::Vector2;
use crate::collections::vector2_int::Vector2Int;
use crate::collections::vector3::Vector3;
use crate::collections::vector3_int::Vector3Int;
use crate::collections::vector4::Vector4;
use serde::Serialize;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result;
use std::ops::{Add, Div, Mul, Sub};
/// A 4D Vector backed by i32
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Default)]
pub struct Vector4Int {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub w: i32,
}
// const constructors
impl Vector4Int {
    // Creates a new Vector4Int with provided x,y,z,w
    pub const fn new(x: i32, y: i32, z: i32, w: i32) -> Vector4Int {
        Vector4Int { x, y, z, w }
    }
    // Creates a new Vector4Int with 0,0,0,0
    pub const fn zero() -> Vector4Int {
        Vector4Int::new(0, 0, 0, 0)
    }
    // Creates a new Vector4Int with 1,1,1,1
    pub const fn one() -> Vector4Int {
        Vector4Int::new(1, 1, 1, 0)
    }
}
impl Vector4Int {
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

        self.x = (self.x as f32 / mag) as i32;
        self.y = (self.y as f32 / mag) as i32;
        self.z = (self.z as f32 / mag) as i32;
        self.w = (self.w as f32 / mag) as i32;
    }
    /// Normalizes all the values from 0-1 and returns a NEW instance
    pub fn normalize_and_copy(&self) -> Vector4Int {
        let mag = self.magnitude();
        if mag == 0.0 {
            panic!("Cannot normalize a zero-length vector");
        }
        Vector4Int {
            x: (self.x as f32 / mag) as i32,
            y: (self.y as f32 / mag) as i32,
            z: (self.z as f32 / mag) as i32,
            w: (self.w as f32 / mag) as i32,
        }
    }
    /// Clamps the x value of THIS instance between min and max inclusively
    pub fn clamp_x(&mut self, min: i32, max: i32) {
        self.x = self.x.clamp(min, max);
    }
    /// Clamps the y value of THIS instance between min and max inclusively
    pub fn clamp_y(&mut self, min: i32, max: i32) {
        self.y = self.y.clamp(min, max);
    }
    /// Clamps the z value of THIS instance between min and max inclusively
    pub fn clamp_z(&mut self, min: i32, max: i32) {
        self.z = self.z.clamp(min, max);
    }
    /// Clamps the z value of THIS instance between min and max inclusively
    pub fn clamp_w(&mut self, min: i32, max: i32) {
        self.w = self.w.clamp(min, max);
    }
    /// Clamps the x value of a NEW instance between min and max inclusively
    pub fn clamp_x_and_copy(&self, min: i32, max: i32) -> Vector4Int {
        Vector4Int::new(self.x.clamp(min, max), self.y, self.z, self.w)
    }
    /// Clamps the y value of a NEW instance between min and max inclusively
    pub fn clamp_y_and_copy(&self, min: i32, max: i32) -> Vector4Int {
        Vector4Int::new(self.x, self.y.clamp(min, max), self.z, self.w)
    }
    /// Clamps the z value of a NEW instance between min and max inclusively
    pub fn clamp_z_and_copy(&self, min: i32, max: i32) -> Vector4Int {
        Vector4Int::new(self.x, self.y, self.z.clamp(min, max), self.w)
    }
    /// Clamps the z value of a NEW instance between min and max inclusively
    pub fn clamp_w_and_copy(&self, min: i32, max: i32) -> Vector4Int {
        Vector4Int::new(self.x, self.y, self.z, self.w.clamp(min, max))
    }
    /// Clamps all values of THIS instance between min and max inclusively
    pub fn clamped(&mut self, min: Vector4Int, max: Vector4Int) {
        self.x = self.x.clamp(min.x, max.x);
        self.y = self.y.clamp(min.y, max.y);
        self.z = self.z.clamp(min.z, max.z);
    }
    /// Clamps all values of a NEW instance between min and max inclusively
    pub fn clamp_and_copy(&self, min: Vector4Int, max: Vector4Int) -> Vector4Int {
        Vector4Int::new(self.x.clamp(min.x, max.x), self.y.clamp(min.y, max.y), self.z.clamp(min.z, max.z), self.w.clamp(min.w, max.w))
    }
    /// returns a new instance of Vector2 discarding z and w
    pub fn to_vector2(&self) -> Vector2 {
        Vector2::new(self.x as f32, self.y as f32)
    }
    /// returns a new instance of Vector3 discarding w
    pub fn to_vector3(&self) -> Vector3 {
        Vector3::new(self.x as f32, self.y as f32, self.z as f32)
    }
    /// returns a new instance of Vector4
    pub fn to_vector4(&self) -> Vector4 {
        Vector4::new(self.x as f32, self.y as f32, self.z as f32, self.w as f32)
    }
    /// returns a new instance of Vector2Int discarding z and w
    pub fn to_vector2_int(&self) -> Vector2Int {
        Vector2Int::new(self.x, self.y)
    }
    /// returns a new instance of Vector3Int discarding w
    pub fn to_vector3_int(&self) -> Vector3Int {
        Vector3Int::new(self.x, self.y, self.z)
    }
}

// whole num mult
impl Mul<i32> for Vector4Int {
    type Output = Vector4Int;
    fn mul(self, x: i32) -> Vector4Int {
        Vector4Int {
            x: self.x * x,
            y: self.y * x,
            z: self.z * x,
            w: self.w * x,
        }
    }
}
// whole num divide
impl Div<i32> for Vector4Int {
    type Output = Vector4Int;
    fn div(self, x: i32) -> Vector4Int {
        Vector4Int {
            x: self.x / x,
            y: self.y / x,
            z: self.z / x,
            w: self.w / x,
        }
    }
}
// vector add
impl Add<Vector4Int> for Vector4Int {
    type Output = Vector4Int;
    fn add(self, x: Vector4Int) -> Vector4Int {
        Vector4Int {
            x: self.x + x.x,
            y: self.y + x.y,
            z: self.z + x.z,
            w: self.z + x.w,
        }
    }
}
// vector subtract
impl Sub<Vector4Int> for Vector4Int {
    type Output = Vector4Int;
    fn sub(self, x: Vector4Int) -> Vector4Int {
        Vector4Int {
            x: self.x - x.x,
            y: self.y - x.y,
            z: self.z - x.z,
            w: self.w - x.w,
        }
    }
}
// Display
impl Display for Vector4Int {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "Vector4Int({}, {}, {}, {})", self.x, self.y, self.z, self.w)
    }
}
