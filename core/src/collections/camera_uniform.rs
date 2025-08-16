use core::f32;

use crate::{collections::projection::Projection, system::system_game_states::state_camera::CameraSnapshot};
use cgmath::{prelude::*, Point3};

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

    // UPDATED!
    // pub fn update_view_proj(&mut self, camera: &CameraState, projection: &Projection) {
    //     let p = Point3::new(camera.position.x, camera.position.y, camera.position.z);
    //     self.view_position = p.to_homogeneous().into();
    //     self.view_proj = (projection.calc_matrix() * camera.calc_matrix()).into()
    // }
    pub fn update_view_proj2(&mut self, camera: &CameraSnapshot, projection: &Projection) {
        let p = Point3::new(camera.position.x, camera.position.y, camera.position.z);
        self.view_position = p.to_homogeneous().into();
        self.view_proj = (projection.calc_matrix() * camera.calc_matrix()).into()
    }
}
