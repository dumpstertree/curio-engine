use crate::{system::system_game_state::RecordCommon, Gizmo};

#[derive(Default, Hash, Clone)]
pub struct SysRecordGizmos {
    pub draw_calls: Vec<Gizmo>,
}
impl SysRecordGizmos {
    pub fn new<'a>() -> SysRecordGizmos {
        SysRecordGizmos { draw_calls: Vec::new() }
    }
}
impl RecordCommon for SysRecordGizmos {
    fn id() -> i32 {
        105
    }
}
