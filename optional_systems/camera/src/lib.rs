pub use crate::facet::camera::Camera;
pub use crate::record::sys_record_camera::SysRecordCamera;

pub mod record {
    pub(crate) mod sys_record_camera;
}
pub mod habit {
    pub(crate) mod system_camera_update_state;
}
pub mod facet {
    pub(crate) mod camera;
}

pub mod camera_snapshot;
pub mod camera_uniform;

pub fn main() {}
