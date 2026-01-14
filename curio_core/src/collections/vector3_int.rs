use crate::collections::f32;
use crate::collections::vector2::Vector2;
use crate::collections::vector2_int::Vector2Int;
use crate::collections::vector3::Vector3;
use crate::collections::vector4::Vector4;
use crate::collections::vector4_int::Vector4Int;
use core::fmt;
use fmt::Display;
use serde::Serialize;
use std::fmt::Formatter;
use std::fmt::Result;
use std::ops::{Add, Div, Mul, Sub};

/// A 3D Vector backed by i32
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Default)]
pub struct Vector3Int {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}
// const constructors
impl Vector3Int {
    /// Creates a Vector3Int with the provided values
    pub const fn new(x: i32, y: i32, z: i32) -> Vector3Int {
        Vector3Int { x, y, z }
    }
    /// Creates a Vector3Int with the value of (0, 0, 0)
    pub const fn zero() -> Vector3Int {
        Vector3Int::new(0, 0, 0)
    }
    /// Creates a Vector3Int with the value of (1, 1, 1)
    pub const fn one() -> Vector3Int {
        Vector3Int::new(1, 1, 1)
    }
    /// Creates a Vector3Int with the value of (0, 0, 1)
    pub const fn forward() -> Vector3Int {
        Vector3Int::new(0, 0, 1)
    }
    /// Creates a Vector3Int with the value of (0, 0, -1)
    pub const fn back() -> Vector3Int {
        Vector3Int::new(0, 0, -1)
    }
    /// Creates a Vector3Int with the value of (1, 0, )
    pub const fn left() -> Vector3Int {
        Vector3Int::new(1, 0, 0)
    }
    /// Creates a Vector3Int with the value of (-1, 0, 0)
    pub const fn right() -> Vector3Int {
        Vector3Int::new(-1, 0, 0)
    }
    /// Creates a Vector3Int with the value of (0, 1, 0)
    pub const fn up() -> Vector3Int {
        Vector3Int::new(0, 1, 0)
    }
    /// Creates a Vector3Int with the value of (0, -1, 0)
    pub const fn down() -> Vector3Int {
        Vector3Int::new(0, -1, 0)
    }
}
// helpers
impl Vector3Int {
    // Returns the size of the vector based on x,y,z
    pub fn magnitude(self) -> f32 {
        ((self.x * self.x + self.y * self.y + self.z * self.z) as f32).sqrt()
    }
    // Returns the dot product of the lhs and rhs with a bounds of -1 to 1
    pub fn dot(lhs: Vector3Int, rhs: Vector3Int) -> i32 {
        lhs.x * rhs.x + lhs.y * rhs.y + lhs.z * rhs.z
    }
    /// Returns a vector perpendicular to rhs and lhs
    pub fn cross(lhs: Vector3Int, rhs: Vector3Int) -> Vector3Int {
        Vector3Int::new(lhs.y * rhs.z - lhs.z * rhs.y, lhs.z * rhs.x - lhs.x * rhs.z, lhs.x * rhs.y - lhs.y * rhs.x)
    }
    ///
    pub fn reflect(direction: Vector3Int, normal: Vector3Int) -> Vector3Int {
        let factor = Vector3Int::dot(normal, direction) * -2;
        Vector3Int::new(factor * normal.x + direction.x, factor * normal.y + direction.y, factor * normal.z + direction.z)
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
    }
    /// Normalizes all the values from 0-1 and returns a NEW instance
    pub fn normalize_and_copy(&self) -> Vector3Int {
        let mag = self.magnitude();
        if mag == 0.0 {
            panic!("Cannot normalize a zero-length vector");
        }
        Vector3Int {
            x: (self.x as f32 / mag) as i32,
            y: (self.y as f32 / mag) as i32,
            z: (self.z as f32 / mag) as i32,
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
    /// Clamps the x value of a NEW instance between min and max inclusively
    pub fn clamp_x_and_copy(&self, min: i32, max: i32) -> Vector3Int {
        Vector3Int::new(self.x.clamp(min, max), self.y, self.z)
    }
    /// Clamps the y value of a NEW instance between min and max inclusively
    pub fn clamp_y_and_copy(&self, min: i32, max: i32) -> Vector3Int {
        Vector3Int::new(self.x, self.y.clamp(min, max), self.z)
    }
    /// Clamps the z value of a NEW instance between min and max inclusively
    pub fn clamp_z_and_copy(&self, min: i32, max: i32) -> Vector3Int {
        Vector3Int::new(self.x, self.y, self.z.clamp(min, max))
    }
    /// Clamps all values of THIS instance between min and max inclusively
    pub fn clamped(&mut self, min: Vector3Int, max: Vector3Int) {
        self.x = self.x.clamp(min.x, max.x);
        self.y = self.y.clamp(min.y, max.y);
        self.z = self.z.clamp(min.z, max.z);
    }
    /// Clamps all values of a NEW instance between min and max inclusively
    pub fn clamp_and_copy(&self, min: Vector3Int, max: Vector3Int) -> Vector3Int {
        Vector3Int::new(self.x.clamp(min.x, max.x), self.y.clamp(min.y, max.y), self.z.clamp(min.z, max.z))
    }
    /// returns a new instance of Vector2 discarding z
    pub fn to_vector2(&self) -> Vector2 {
        Vector2::new(self.x as f32, self.y as f32)
    }
    /// returns a new instance of Vector3 converting x,y,z to f32
    pub fn to_vector3(&self) -> Vector3 {
        Vector3::new(self.x as f32, self.y as f32, self.z as f32)
    }
    /// returns a new instance of Vector4 appending w as 0.0
    pub fn to_vector4(&self) -> Vector4 {
        Vector4::new(self.x as f32, self.y as f32, self.z as f32, 0.0)
    }
    /// returns a new instance of Vector2Int discarding z
    pub fn to_vector2_int(&self) -> Vector2Int {
        Vector2Int::new(self.x, self.y)
    }
    /// returns a new instance of Vector4Int appending w as 0
    pub fn to_vector4_int(&self) -> Vector4Int {
        Vector4Int::new(self.x, self.y, self.z, 0)
    }
}
// whole num mult
impl Mul<i32> for Vector3Int {
    type Output = Vector3Int;
    fn mul(self, x: i32) -> Vector3Int {
        Vector3Int { x: self.x * x, y: self.y * x, z: self.z * x }
    }
}
// whole num divide
impl Div<i32> for Vector3Int {
    type Output = Vector3Int;
    fn div(self, x: i32) -> Vector3Int {
        Vector3Int { x: self.x / x, y: self.y / x, z: self.z / x }
    }
}
// vector add
impl Add<Vector3Int> for Vector3Int {
    type Output = Vector3Int;
    fn add(self, x: Vector3Int) -> Vector3Int {
        Vector3Int { x: self.x + x.x, y: self.y + x.y, z: self.z + x.z }
    }
}
// vector subtract
impl Sub<Vector3Int> for Vector3Int {
    type Output = Vector3Int;
    fn sub(self, x: Vector3Int) -> Vector3Int {
        Vector3Int { x: self.x - x.x, y: self.y - x.y, z: self.z - x.z }
    }
}
// display
impl Display for Vector3Int {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "Vector3Int({}, {}, {})", self.x, self.y, self.z)
    }
}
