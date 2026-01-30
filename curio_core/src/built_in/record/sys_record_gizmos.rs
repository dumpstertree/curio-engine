use crate::{collections::gizmo::Gizmo, system::system_game_state::IState};

#[derive(Default, Hash, Clone)]
pub struct SysRecordGizmos {
    pub draw_calls: Vec<Gizmo>,
}
impl SysRecordGizmos {
    pub fn new<'a>() -> SysRecordGizmos {
        SysRecordGizmos { draw_calls: Vec::new() }
    }
}
impl IState for SysRecordGizmos {
    fn id() -> i32 {
        9827234
    }
}
