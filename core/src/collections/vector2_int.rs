use crate::collections::f32;
use crate::collections::vector2::Vector2;
use crate::collections::vector3::Vector3;
use crate::collections::vector3_int::Vector3Int;
use crate::collections::vector4::Vector4;
use crate::collections::vector4_int::Vector4Int;
use core::fmt;
use fmt::Display;
use serde::Serialize;
use std::fmt::Formatter;
use std::fmt::Result;
use std::ops::{Add, Div, Mul, Sub};

/// A 2D Vector backed by i32
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Default)]
pub struct Vector2Int {
    pub x: i32,
    pub y: i32,
}
// const constructors
impl Vector2Int {
    /// Creates a Vector2Int with the provided values
    pub const fn new(x: i32, y: i32) -> Vector2Int {
        Vector2Int { x, y }
    }
    /// Creates a Vector2Int with the value of (0, 0)
    pub const fn zero() -> Vector2Int {
        Vector2Int::new(0, 0)
    }
    /// Creates a Vector2Int with the value of (1, 1)
    pub const fn one() -> Vector2Int {
        Vector2Int::new(1, 1)
    }
}
// helpers
impl Vector2Int {
    // Returns the size of the vector based on x,y,z
    pub fn magnitude(self) -> f32 {
        ((self.x * self.x + self.y * self.y) as f32).sqrt()
    }
    /// Normalizes all the values from 0-1 of THIS instance
    pub fn normalize(&mut self) {
        let mag = self.magnitude();
        if mag == 0.0 {
            panic!("Cannot normalize a zero-length vector");
        }

        self.x = (self.x as f32 / mag) as i32;
        self.y = (self.y as f32 / mag) as i32;
    }
    /// Normalizes all the values from 0-1 and returns a NEW instance
    pub fn normalize_and_copy(&self) -> Vector2Int {
        let mag = self.magnitude();
        if mag == 0.0 {
            panic!("Cannot normalize a zero-length vector");
        }
        Vector2Int {
            x: (self.x as f32 / mag) as i32,
            y: (self.y as f32 / mag) as i32,
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
    /// Clamps the x value of a NEW instance between min and max inclusively
    pub fn clamp_x_and_copy(&self, min: i32, max: i32) -> Vector2Int {
        Vector2Int::new(self.x.clamp(min, max), self.y)
    }
    /// Clamps the y value of a NEW instance between min and max inclusively
    pub fn clamp_y_and_copy(&self, min: i32, max: i32) -> Vector2Int {
        Vector2Int::new(self.x, self.y.clamp(min, max))
    }
    /// Clamps all values of THIS instance between min and max inclusively
    pub fn clamped(&mut self, min: Vector2Int, max: Vector2Int) {
        self.x = self.x.clamp(min.x, max.x);
        self.y = self.y.clamp(min.y, max.y);
    }
    /// Clamps all values of a NEW instance between min and max inclusively
    pub fn clamp_and_copy(&self, min: Vector2Int, max: Vector2Int) -> Vector2Int {
        Vector2Int::new(self.x.clamp(min.x, max.x), self.y.clamp(min.y, max.y))
    }
    /// returns a new instance of Vector2 as f32
    pub fn to_vector2(&self) -> Vector2 {
        Vector2::new(self.x as f32, self.y as f32)
    }
    /// returns a new instance of Vector3 converting appending new_z as f32
    pub fn to_vector3(&self, new_z: f32) -> Vector3 {
        Vector3::new(self.x as f32, self.y as f32, new_z)
    }
    /// returns a new instance of Vector4 appending new_z and new_w as f32
    pub fn to_vector4(&self, new_z: f32, new_w: f32) -> Vector4 {
        Vector4::new(self.x as f32, self.y as f32, new_z, new_w)
    }
    /// returns a new instance of Vector3Int appending new_z
    pub fn to_vector3_int(&self, new_z: i32) -> Vector3Int {
        Vector3Int::new(self.x, self.y, new_z)
    }
    /// returns a new instance of Vector3Int appending new_z and new_w
    pub fn to_vector4_int(&self, new_z: i32, new_w: i32) -> Vector4Int {
        Vector4Int::new(self.x, self.y, new_z, new_w)
    }
}
// whole num mult
impl Mul<i32> for Vector2Int {
    type Output = Vector2Int;
    fn mul(self, x: i32) -> Vector2Int {
        Vector2Int {
            x: self.x * x,
            y: self.y * x,
        }
    }
}
// whole num divide
impl Div<i32> for Vector2Int {
    type Output = Vector2Int;
    fn div(self, x: i32) -> Vector2Int {
        Vector2Int {
            x: self.x / x,
            y: self.y / x,
        }
    }
}
// vector add
impl Add<Vector2Int> for Vector2Int {
    type Output = Vector2Int;
    fn add(self, x: Vector2Int) -> Vector2Int {
        Vector2Int {
            x: self.x + x.x,
            y: self.y + x.y,
        }
    }
}
// vector subtract
impl Sub<Vector2Int> for Vector2Int {
    type Output = Vector2Int;
    fn sub(self, x: Vector2Int) -> Vector2Int {
        Vector2Int {
            x: self.x - x.x,
            y: self.y - x.y,
        }
    }
}
// display
impl Display for Vector2Int {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "Vector2Int({}, {})", self.x, self.y)
    }
}
