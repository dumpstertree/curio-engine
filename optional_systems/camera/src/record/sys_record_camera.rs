use crate::camera_snapshot::CameraSnapshot;
use curio_core::{FieldState, RecordOverride, RecordScope, Vector3};
use record::record;

// #[derive(Default, Hash, Clone)]
#[record(name = "Camera", ownership = RecordScope::Instance)]
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
impl SysRecordCamera {
    #[rustfmt::skip]
    pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::from_cols(
        cgmath::Vector4::new(1.0, 0.0, 0.0, 0.0),
        cgmath::Vector4::new(0.0, 1.0, 0.0, 0.0),
        cgmath::Vector4::new(0.0, 0.0, 0.5, 0.0),
        cgmath::Vector4::new(0.0, 0.0, 0.5, 1.0),
    );
}
impl RecordOverride for SysRecordCamera {
    fn set_state(&mut self, _: &str, _: &str) {}
    fn get_state(&self) -> Vec<FieldState> {
        vec![FieldState::new("camera", &self.cameras)]
    }
}
