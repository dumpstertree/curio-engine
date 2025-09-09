use crate::collections::f32;
use crate::collections::vector2::Vector2;
use crate::collections::vector2_int::Vector2Int;
use crate::collections::vector3_int::Vector3Int;
use crate::collections::vector4::Vector4;
use crate::collections::vector4_int::Vector4Int;
use core::fmt;
use fmt::Display;
use serde::Deserialize;
use serde::Serialize;
use std::fmt::Formatter;
use std::fmt::Result;
use std::ops::{Add, Div, Mul, Sub};

/// A 3D Vector backed by f32
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
// const constructors
impl Vector3 {
    /// Creates a Vector3 with the provided values
    pub const fn new(x: f32, y: f32, z: f32) -> Vector3 {
        Vector3 { x, y, z }
    }
    /// Creates a Vector3 with the value of (0, 0, 0)
    pub const fn zero() -> Vector3 {
        Vector3::new(0.0, 0.0, 0.0)
    }
    /// Creates a Vector3 with the value of (1, 1, 1)
    pub const fn one() -> Vector3 {
        Vector3::new(1.0, 1.0, 1.0)
    }
    /// Creates a Vector3 with the value of (0, 0, 1)
    pub const fn forward() -> Vector3 {
        Vector3::new(0.0, 0.0, 1.0)
    }
    /// Creates a Vector3 with the value of (0, 0, -1)
    pub const fn back() -> Vector3 {
        Vector3::new(0.0, 0.0, -1.0)
    }
    /// Creates a Vector3 with the value of (1, 0, )
    pub const fn left() -> Vector3 {
        Vector3::new(1.0, 0.0, 0.0)
    }
    /// Creates a Vector3 with the value of (-1, 0, 0)
    pub const fn right() -> Vector3 {
        Vector3::new(-1.0, 0.0, 0.0)
    }
    /// Creates a Vector3 with the value of (0, 1, 0)
    pub const fn up() -> Vector3 {
        Vector3::new(0.0, 1.0, 0.0)
    }
    /// Creates a Vector3 with the value of (0, -1, 0)
    pub const fn down() -> Vector3 {
        Vector3::new(0.0, -1.0, 0.0)
    }
}
// helpers
impl Vector3 {
    // Returns the size of the vector based on x,y,z
    pub fn magnitude(self) -> f32 {
        ((self.x * self.x + self.y * self.y + self.z * self.z) as f32).sqrt()
    }
    // Returns the dot product of the lhs and rhs with a bounds of -1 to 1
    pub fn dot(lhs: Vector3, rhs: Vector3) -> f32 {
        lhs.x * rhs.x + lhs.y * rhs.y + lhs.z * rhs.z
    }
    /// Returns a vector perpendicular to rhs and lhs
    pub fn cross(lhs: Vector3, rhs: Vector3) -> Vector3 {
        Vector3::new(lhs.y * rhs.z - lhs.z * rhs.y, lhs.z * rhs.x - lhs.x * rhs.z, lhs.x * rhs.y - lhs.y * rhs.x)
    }
    ///
    pub fn reflect(direction: Vector3, normal: Vector3) -> Vector3 {
        let factor = Vector3::dot(normal, direction) * -2.0;
        Vector3::new(factor * normal.x + direction.x, factor * normal.y + direction.y, factor * normal.z + direction.z)
    }
    /// Normalizes all the values from 0-1 of THIS instance
    pub fn normalize(&mut self) {
        let mag = self.magnitude();
        if mag == 0.0 {
            panic!("Cannot normalize a zero-length vector");
        }

        self.x = self.x / mag;
        self.y = self.y / mag;
        self.z = self.z / mag;
    }
    /// Normalizes all the values from 0-1 and returns a NEW instance
    pub fn normalize_and_copy(&self) -> Vector3 {
        let mag = self.magnitude();
        if mag == 0.0 {
            panic!("Cannot normalize a zero-length vector");
        }
        Vector3 {
            x: self.x as f32 / mag,
            y: self.y as f32 / mag,
            z: self.z as f32 / mag,
        }
    }
    /// lerp from one vector to another
    pub fn lerp(a: Vector3, b: Vector3, t: f32) -> Vector3 {
        a + (b - a) * t
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
    /// Clamps the x value of a NEW instance between min and max inclusively
    pub fn clamp_x_and_copy(&self, min: f32, max: f32) -> Vector3 {
        Vector3::new(self.x.clamp(min, max), self.y, self.z)
    }
    /// Clamps the y value of a NEW instance between min and max inclusively
    pub fn clamp_y_and_copy(&self, min: f32, max: f32) -> Vector3 {
        Vector3::new(self.x, self.y.clamp(min, max), self.z)
    }
    /// Clamps the z value of a NEW instance between min and max inclusively
    pub fn clamp_z_and_copy(&self, min: f32, max: f32) -> Vector3 {
        Vector3::new(self.x, self.y, self.z.clamp(min, max))
    }
    /// Clamps all values of THIS instance between min and max inclusively
    pub fn clamped(&mut self, min: Vector3, max: Vector3) {
        self.x = self.x.clamp(min.x, max.x);
        self.y = self.y.clamp(min.y, max.y);
        self.z = self.z.clamp(min.z, max.z);
    }
    /// Clamps all values of a NEW instance between min and max inclusively
    pub fn clamp_and_copy(&self, min: Vector3, max: Vector3) -> Vector3 {
        Vector3::new(self.x.clamp(min.x, max.x), self.y.clamp(min.y, max.y), self.z.clamp(min.z, max.z))
    }
    /// returns a new instance of Vector2 discarding z
    pub fn to_vector2(&self) -> Vector2 {
        Vector2::new(self.x, self.y)
    }
    /// returns a new instance of Vector4 appending w as new_w
    pub fn to_vector4(&self, new_w: f32) -> Vector4 {
        Vector4::new(self.x, self.y, self.z, new_w)
    }
    /// returns a new instance of Vector2Int discarding z
    pub fn to_vector2_int(&self) -> Vector2Int {
        Vector2Int::new(self.x.round() as i32, self.y.round() as i32)
    }
    /// returns a new instance of Vector3 converting x,y,z to f32
    pub fn to_vector3_int(&self) -> Vector3Int {
        Vector3Int::new(self.x.round() as i32, self.y.round() as i32, self.z.round() as i32)
    }
    /// returns a new instance of Vector4Int appending w as new_w
    pub fn to_vector4_int(&self, new_w: i32) -> Vector4Int {
        Vector4Int::new(self.x.round() as i32, self.y.round() as i32, self.z.round() as i32, new_w)
    }
}
// whole num mult
impl Mul<f32> for Vector3 {
    type Output = Vector3;
    fn mul(self, x: f32) -> Vector3 {
        Vector3 { x: self.x * x, y: self.y * x, z: self.z * x }
    }
}
// whole num divide
impl Div<f32> for Vector3 {
    type Output = Vector3;
    fn div(self, x: f32) -> Vector3 {
        Vector3 { x: self.x / x, y: self.y / x, z: self.z / x }
    }
}
// vector add
impl Add<Vector3> for Vector3 {
    type Output = Vector3;
    fn add(self, x: Vector3) -> Vector3 {
        Vector3 { x: self.x + x.x, y: self.y + x.y, z: self.z + x.z }
    }
}
// vector subtract
impl Sub<Vector3> for Vector3 {
    type Output = Vector3;
    fn sub(self, x: Vector3) -> Vector3 {
        Vector3 { x: self.x - x.x, y: self.y - x.y, z: self.z - x.z }
    }
}
// display
impl Display for Vector3 {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "Vector3({}, {}, {})", self.x, self.y, self.z)
    }
}
