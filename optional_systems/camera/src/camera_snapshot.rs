use cgmath::{Matrix4, Point3};
use curio_core::{ExtensionsF32, Frustrum, Quaternion, Vector3};
use serde::{Deserialize, Serialize};
use std::hash::Hash;

use crate::camera_uniform::CameraUniform;

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct CameraSnapshot {
    pub position: Vector3,
    pub rotation: Quaternion,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}
impl Eq for CameraSnapshot {}
impl Hash for CameraSnapshot {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.fovy.hash(state);
        self.znear.hash(state);
        self.zfar.hash(state);
        self.position.hash(state);
        self.rotation.hash(state);
    }
}
impl Default for CameraSnapshot {
    fn default() -> Self {
        Self {
            position: Default::default(),
            rotation: Default::default(),
            fovy: 60.0,
            znear: 0.1,
            zfar: 100.0,
        }
    }
}
impl CameraSnapshot {
    pub fn new(position: Vector3) -> CameraSnapshot {
        CameraSnapshot {
            position: position.into(),
            rotation: Quaternion::identity(),
            fovy: 60.0,
            znear: 0.1,
            zfar: 512.0,
        }
    }

    pub fn calc_matrix(&self) -> Matrix4<f32> {
        let p3 = Point3::new(self.position.x, self.position.y, self.position.z);
        let f0 = self.rotation * Vector3::forward();
        let f = cgmath::Vector3::new(f0.x, f0.y, f0.z);
        let u0 = self.rotation * Vector3::up();
        let u = cgmath::Vector3::new(u0.x, u0.y, u0.z);

        Matrix4::look_to_rh(p3, f, u)
    }
    pub fn get_projection(&self, width: i32, height: i32) -> Frustrum {
        Frustrum::new(width as u32, height as u32, cgmath::Deg(self.fovy), self.znear, self.zfar)
    }
    pub fn get_uniform(&self, width: i32, height: i32) -> CameraUniform {
        let mut c = CameraUniform::new();
        c.update_view_proj2(self, &self.get_projection(width, height));
        c
    }
}
