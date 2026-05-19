use core::f32;
use std::hash::Hash;

use crate::{extensions::extensions_f32::ExtensionsF32, Frustrum, Quaternion, Vector3};
use cgmath::{prelude::*, Matrix4, Point3};

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub view_position: [f32; 4],
    pub view_proj: [[f32; 4]; 4],
}
impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_position: [0.0; 4],
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    pub fn update_view_proj2(&mut self, camera: &CameraSnapshot, projection: &Frustrum) {
        let p = Point3::new(camera.position.x, camera.position.y, camera.position.z);
        self.view_position = p.to_homogeneous().into();
        self.view_proj = (projection.calc_matrix() * camera.calc_matrix()).into()
    }
}

#[derive(Clone, serde::Serialize, serde::Deserialize, PartialEq)]
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
