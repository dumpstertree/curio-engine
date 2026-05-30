use crate::camera_snapshot::CameraSnapshot;
use bytemuck::{Pod, Zeroable};
use cgmath::{Point3, prelude::*};
use core::f32;
use curio_core::Frustrum;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
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
