use crate::extensions::extensions_f32::ExtensionsF32;
use crate::Vector3;
use cgmath::num_traits::real::Real;
use core::fmt;
use fmt::Display;
use serde::Deserialize;
use serde::Serialize;
use std::f32::consts::PI;
use std::fmt::Formatter;
use std::fmt::Result;
use std::hash::Hash;
use std::ops::Mul;

/// A representation of 3D rotation
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default, bytemuck::Zeroable, bytemuck::Pod)]
pub struct Quaternion {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}
impl Eq for Quaternion {}
impl Hash for Quaternion {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.w.hash(state);
        self.x.hash(state);
        self.y.hash(state);
        self.z.hash(state);
    }
}

impl Quaternion {
    /// Returns a Quaternion with provided values
    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Quaternion {
        Quaternion { x: x, y: y, z: z, w: w }
    }
    /// Returns a Quaternion with default values of 0.0, 0.0, 0.0, 1.0
    pub const fn identity() -> Quaternion {
        Quaternion::new(0.0, 0.0, 0.0, 1.0)
    }
    /// Returns a Quaternion with default values of 0.0, 0.0, 0.0, 0.0
    pub const fn zero() -> Quaternion {
        Quaternion::new(0.0, 0.0, 0.0, 10.0)
    }
}
// static
impl Quaternion {
    /// Returns an instance of a Quaternion representing the euler angles provided
    pub fn from_euler(euler: Vector3) -> Quaternion {
        // Convert degrees to radians
        let (roll, pitch, yaw) = (euler.x.to_radians(), euler.y.to_radians(), euler.z.to_radians());

        // Half angles
        let (hr, hp, hy) = (roll * 0.5, pitch * 0.5, yaw * 0.5);

        // Calculate sin and cos for each half angle
        let (sr, cr) = (hr.sin(), hr.cos());
        let (sp, cp) = (hp.sin(), hp.cos());
        let (sy, cy) = (hy.sin(), hy.cos());

        // Apply quaternion formula (XYZ order → roll, pitch, yaw)
        Quaternion {
            w: cr * cp * cy + sr * sp * sy,
            x: sr * cp * cy - cr * sp * sy,
            y: cr * sp * cy + sr * cp * sy,
            z: cr * cp * sy - sr * sp * cy,
        }
    }
    pub fn inverse(&self) -> Self {
        let norm_sq = self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w;

        if norm_sq == 0.0 {
            // Return identity as a safe fallback
            return Self::identity();
        }

        Self {
            x: -self.x / norm_sq,
            y: -self.y / norm_sq,
            z: -self.z / norm_sq,
            w: self.w / norm_sq,
        }
    }

    /// Returns an instance of a Quaternion representing the angle in degrees around provided axis
    pub fn from_angle_axis(axis: Vector3, angle: f32) -> Quaternion {
        if angle == 0.0 {
            return Quaternion::identity();
        }
        Quaternion {
            x: axis.x * f32::sin(angle / 2.0),
            y: axis.y * f32::sin(angle / 2.0),
            z: axis.z * f32::sin(angle / 2.0),
            w: f32::cos(angle / 2.0),
        }
    }
    /// Returns an instance of a Quaternion representing the orientation of forward, up and the cross product
    pub fn from_look_rotation(forward: Vector3, up: Vector3) -> Quaternion {
        let f = forward.normalize_and_copy();
        let u = up.normalize_and_copy();
        let r = Vector3::cross(u, f).normalize_and_copy();
        let u = Vector3::cross(f, r); // ensure orthogonal up

        // Construct quaternion from basis vectors directly
        // using direction vectors' rotation alignment method
        let trace = r.x + u.y + f.z;

        if trace > 0.0 {
            let s = (trace + 1.0).sqrt() * 2.0;
            let inv_s = 1.0 / s;
            Quaternion::new((u.z - f.y) * inv_s, (f.x - r.z) * inv_s, (r.y - u.x) * inv_s, 0.25 * s)
        } else if r.x > u.y && r.x > f.z {
            let s = (1.0 + r.x - u.y - f.z).sqrt() * 2.0;
            let inv_s = 1.0 / s;
            Quaternion::new(0.25 * s, (r.y + u.x) * inv_s, (f.x + r.z) * inv_s, (u.z - f.y) * inv_s)
        } else if u.y > f.z {
            let s = (1.0 + u.y - r.x - f.z).sqrt() * 2.0;
            let inv_s = 1.0 / s;
            Quaternion::new((r.y + u.x) * inv_s, 0.25 * s, (u.z + f.y) * inv_s, (f.x - r.z) * inv_s)
        } else {
            let s = (1.0 + f.z - r.x - u.y).sqrt() * 2.0;
            let inv_s = 1.0 / s;
            Quaternion::new((f.x + r.z) * inv_s, (u.z + f.y) * inv_s, 0.25 * s, (r.y - u.x) * inv_s)
        }
    }
}
// instance
impl Quaternion {
    /// Performs spherical linear interpolation between two quaternions.
    ///
    /// `t` should be between 0.0 and 1.0, where:
    /// - 0.0 returns `self`
    /// - 1.0 returns `end`
    pub fn slerp(start: Quaternion, end: Quaternion, t: f32) -> Quaternion {
        // Clamp t to [0, 1]
        let t = t.clamp(0.0, 1.0);

        // Compute the cosine of the angle between the two quaternions
        let mut cos_half_theta = start.w * end.w + start.x * end.x + start.y * end.y + start.z * end.z;

        // If cos is negative, negate end to take the shorter path
        let mut end = end;
        if cos_half_theta < 0.0 {
            end = Quaternion { x: -end.x, y: -end.y, z: -end.z, w: -end.w };
            cos_half_theta = -cos_half_theta;
        }

        // If quaternions are very close, use linear interpolation to avoid divide-by-zero
        const EPSILON: f32 = 1e-6;
        if cos_half_theta > 1.0 - EPSILON {
            // Lerp (linear interpolation)
            let inv_t = 1.0 - t;
            let result = Quaternion {
                x: inv_t * start.x + t * end.x,
                y: inv_t * start.y + t * end.y,
                z: inv_t * start.z + t * end.z,
                w: inv_t * start.w + t * end.w,
            };
            return result.normalized();
        }

        // Compute the actual angle between them
        let half_theta = cos_half_theta.acos();
        let sin_half_theta = (1.0 - cos_half_theta * cos_half_theta).sqrt();

        // Compute interpolation factors
        let ratio_a = ((1.0 - t) * half_theta).sin() / sin_half_theta;
        let ratio_b = (t * half_theta).sin() / sin_half_theta;

        // Perform the slerp
        let result = Quaternion {
            x: start.x * ratio_a + end.x * ratio_b,
            y: start.y * ratio_a + end.y * ratio_b,
            z: start.z * ratio_a + end.z * ratio_b,
            w: start.w * ratio_a + end.w * ratio_b,
        };

        result.normalized()
    }

    /// Returns a normalized copy of the quaternion
    pub fn normalized(&self) -> Quaternion {
        let mag = (self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w).sqrt();
        if mag == 0.0 {
            return *self;
        }
        Quaternion {
            x: self.x / mag,
            y: self.y / mag,
            z: self.z / mag,
            w: self.w / mag,
        }
    }
}

impl Quaternion {
    /// Returns a new instance of Vector3 with the Quaternion converted to euler angles
    pub fn to_euler(&self) -> Vector3 {
        let sinr_cosp = 2.0 * (self.w * self.x + self.y * self.z);
        let cosr_cosp = 1.0 - 2.0 * (self.x * self.x + self.y * self.y);
        let roll = sinr_cosp.atan2(cosr_cosp);

        let sinp = 2.0 * (self.w * self.y - self.z * self.x);
        let pitch = if sinp.abs() >= 1.0 {
            // use 90 degrees if out of range
            sinp.signum() * PI / 2.0
        } else {
            sinp.asin()
        };

        let siny_cosp = 2.0 * (self.w * self.z + self.x * self.y);
        let cosy_cosp = 1.0 - 2.0 * (self.y * self.y + self.z * self.z);
        let yaw = siny_cosp.atan2(cosy_cosp);

        Vector3::new(pitch.to_degrees(), yaw.to_degrees(), roll.to_degrees())
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
// vector3 mult
impl Mul<Vector3> for Quaternion {
    type Output = Vector3;
    fn mul(self, v: Vector3) -> Vector3 {
        let q: Quaternion = self;

        // Quaternion multiplication: q * v * q_conjugate
        let q_vec = Vector3 { x: q.x, y: q.y, z: q.z };
        let uv = Vector3::cross(q_vec, v);
        let uuv = Vector3::cross(q_vec, uv);

        let uv = uv * 2.0 * q.w;
        let uuv = uuv * 2.0;

        v + uv + uuv
    }
}
// display
impl Display for Quaternion {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "Quaternion({}, {}, {}, {})", self.x, self.y, self.z, self.w)
    }
}
