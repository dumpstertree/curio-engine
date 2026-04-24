use crate::{collections::camera_uniform::CameraSnapshot, system::system_game_state::RecordCommon, Vector3};

#[derive(Default, Hash, Clone)]
pub struct SysRecordCamera {
    pub resolution_width: i32,
    pub resolution_height: i32,
    pub cameras: CameraSnapshot,
}
impl SysRecordCamera {
    pub fn new() -> SysRecordCamera {
        SysRecordCamera {
            resolution_height: 0,
            resolution_width: 0,
            cameras: CameraSnapshot::new(Vector3::zero()),
        }
    }
}

impl RecordCommon for SysRecordCamera {
    fn id() -> i32 {
        9879897
    }
}
impl SysRecordCamera {
    #[rustfmt::skip]
    pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::from_cols(
        cgmath::Vector4::new(1.0, 0.0, 0.0, 0.0),
        cgmath::Vector4::new(0.0, 1.0, 0.0, 0.0),
        cgmath::Vector4::new(0.0, 0.0, 0.5, 0.0),
        cgmath::Vector4::new(0.0, 0.0, 0.5, 1.0),
    );
}
