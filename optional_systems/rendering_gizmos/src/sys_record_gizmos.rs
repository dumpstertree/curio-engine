// use std::sync::OnceLock;

// use crate::{
//     system::{record_common::RecordOverride, record_id::RecordId},
//     Gizmo, RecordCommon,
// };

// static SYS_RECORD_ID: OnceLock<i32> = OnceLock::new();

// #[derive(Default, Hash, Clone)]
// pub struct SysRecordGizmos {
//     pub draw_calls: Vec<Gizmo>,
// }
// impl SysRecordGizmos {
//     pub fn new<'a>() -> SysRecordGizmos {
//         SysRecordGizmos { draw_calls: Vec::new() }
//     }
// }
// impl RecordCommon for SysRecordGizmos {
//     fn name(&self) -> String {
//         String::from("Gizmos")
//     }
//     fn id() -> i32 {
//         *SYS_RECORD_ID.get_or_init(|| RecordId::of::<SysRecordGizmos>())
//     }
// }
// impl RecordOverride for SysRecordGizmos {
//     fn apply(&mut self, field: &str, val: &str) {}
//     fn get_state(&self) -> Vec<crate::FieldState> {
//         vec![]
//     }
// }
