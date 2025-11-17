use core::collections::camera_uniform::CameraSnapshot;

use core::{collections::vector3::Vector3, system::system_game_state::IState};
use macro_state::global_state;

#[derive(Hash)]
#[global_state]
pub struct CameraState {
    pub resolution_width: i32,
    pub resolution_height: i32,
    pub cameras: CameraSnapshot,
}
impl CameraState {
    pub fn new() -> CameraState {
        CameraState {
            resolution_height: 0,
            resolution_width: 0,
            cameras: CameraSnapshot::new(Vector3::zero()),
        }
    }
}

impl IState for CameraState {
    fn id() -> i32 {
        9879897
    }
}
impl CameraState {
    #[rustfmt::skip]
    pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::from_cols(
        cgmath::Vector4::new(1.0, 0.0, 0.0, 0.0),
        cgmath::Vector4::new(0.0, 1.0, 0.0, 0.0),
        cgmath::Vector4::new(0.0, 0.0, 0.5, 0.0),
        cgmath::Vector4::new(0.0, 0.0, 0.5, 1.0),
    );
}
