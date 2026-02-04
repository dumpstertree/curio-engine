use crate::collections::f32;
use crate::extensions::extensions_f32::ExtensionsF32;
use crate::Vector2Int;
use crate::Vector3;
use crate::Vector3Int;
use crate::Vector4;
use crate::Vector4Int;
use core::fmt;
use fmt::Display;
use serde::Deserialize;
use serde::Serialize;
use std::fmt::Formatter;
use std::fmt::Result;
use std::hash::Hash;
use std::ops::{Add, Div, Mul, Sub};
use std::str::FromStr;

/// A 2D Vector backed by f32
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}
// hash
impl Hash for Vector2 {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.x.hash(state);
        self.y.hash(state);
    }
}
#[derive(Debug, PartialEq, Eq)]
pub struct ParseError;
impl FromStr for Vector2 {
    type Err = ParseError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let s = s.trim();

        // Must start with '(' and end with ')'
        if !s.starts_with('(') || !s.ends_with(')') {
            return Err(ParseError);
        }

        let inner = &s[1..s.len() - 1];
        let mut parts = inner.split(',');

        let x = parts
            .next()
            .and_then(|p| p.trim().parse::<f32>().ok())
            .ok_or(ParseError)?;

        let y = parts
            .next()
            .and_then(|p| p.trim().parse::<f32>().ok())
            .ok_or(ParseError)?;
        // No extra values allowed
        if parts.next().is_some() {
            return Err(ParseError);
        }

        Ok(Vector2 { x, y })
    }
}

impl Eq for Vector2 {}
// const constructors
impl Vector2 {
    /// Creates a Vector2 with the provided values
    pub const fn new(x: f32, y: f32) -> Vector2 {
        Vector2 { x, y }
    }
    /// Creates a Vector2 with the value of (0.0, 0.0)
    pub const fn zero() -> Vector2 {
        Vector2::new(0.0, 0.0)
    }
    /// Creates a Vector2 with the value of (1.0, 1.0)
    pub const fn one() -> Vector2 {
        Vector2::new(1.0, 1.0)
    }
}
// helpers
impl Vector2 {
    /// lerp from one vector to another
    pub fn lerp(a: Vector2, b: Vector2, t: f32) -> Vector2 {
        a + (b - a) * t
    }
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

        self.x = (self.x as f32 / mag) as f32;
        self.y = (self.y as f32 / mag) as f32;
    }
    /// Normalizes all the values from 0-1 and returns a NEW instance
    pub fn normalize_and_copy(&self) -> Vector2 {
        let mag = self.magnitude();
        if mag == 0.0 {
            panic!("Cannot normalize a zero-length vector");
        }
        Vector2 {
            x: (self.x as f32 / mag) as f32,
            y: (self.y as f32 / mag) as f32,
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
    /// Clamps the x value of a NEW instance between min and max inclusively
    pub fn clamp_x_and_copy(&self, min: f32, max: f32) -> Vector2 {
        Vector2::new(self.x.clamp(min, max), self.y)
    }
    /// Clamps the y value of a NEW instance between min and max inclusively
    pub fn clamp_y_and_copy(&self, min: f32, max: f32) -> Vector2 {
        Vector2::new(self.x, self.y.clamp(min, max))
    }
    /// Clamps all values of THIS instance between min and max inclusively
    pub fn clamped(&mut self, min: Vector2, max: Vector2) {
        self.x = self.x.clamp(min.x, max.x);
        self.y = self.y.clamp(min.y, max.y);
    }
    /// Clamps all values of a NEW instance between min and max inclusively
    pub fn clamp_and_copy(&self, min: Vector2, max: Vector2) -> Vector2 {
        Vector2::new(self.x.clamp(min.x, max.x), self.y.clamp(min.y, max.y))
    }

    /// returns a new instance of Vector3 converting appending new_z as f32
    pub fn to_vector3(&self, new_z: f32) -> Vector3 {
        Vector3::new(self.x as f32, self.y as f32, new_z)
    }
    /// returns a new instance of Vector4 appending new_z and new_w as f32
    pub fn to_vector4(&self, new_z: f32, new_w: f32) -> Vector4 {
        Vector4::new(self.x as f32, self.y as f32, new_z, new_w)
    }
    /// returns a new instance of Vector2Int as i32
    pub fn to_vector2_int(&self) -> Vector2Int {
        Vector2Int::new(self.x.round() as i32, self.y.round() as i32)
    }
    /// returns a new instance of Vector3Int appending new_z
    pub fn to_vector3_int(&self, new_z: i32) -> Vector3Int {
        Vector3Int::new(self.x.round() as i32, self.y.round() as i32, new_z)
    }
    /// returns a new instance of Vector3Int appending new_z and new_w
    pub fn to_vector4_int(&self, new_z: i32, new_w: i32) -> Vector4Int {
        Vector4Int::new(self.x.round() as i32, self.y.round() as i32, new_z, new_w)
    }
}
// whole num mult
impl Mul<f32> for Vector2 {
    type Output = Vector2;
    fn mul(self, x: f32) -> Vector2 {
        Vector2 { x: self.x * x, y: self.y * x }
    }
}
// whole num divide
impl Div<f32> for Vector2 {
    type Output = Vector2;
    fn div(self, x: f32) -> Vector2 {
        Vector2 { x: self.x / x, y: self.y / x }
    }
}
// vector add
impl Add<Vector2> for Vector2 {
    type Output = Vector2;
    fn add(self, x: Vector2) -> Vector2 {
        Vector2 { x: self.x + x.x, y: self.y + x.y }
    }
}
// vector subtract
impl Sub<Vector2> for Vector2 {
    type Output = Vector2;
    fn sub(self, x: Vector2) -> Vector2 {
        Vector2 { x: self.x - x.x, y: self.y - x.y }
    }
}
// display
impl Display for Vector2 {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "Vector2({}, {})", self.x, self.y)
    }
}
