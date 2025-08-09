use core::f32;

use crate::{system::system_game_states::state_camera::CameraState, Collections::projection::Projection};
use cgmath::prelude::*;

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
    pub fn update_view_proj(&mut self, camera: &CameraState, projection: &Projection) {
        self.view_position = camera.position.to_point3().to_homogeneous().into();
        self.view_proj = (projection.calc_matrix() * camera.calc_matrix()).into()
    }
}
