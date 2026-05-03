use std::sync::OnceLock;

use crate::{graphics::light_uniform::DrawCallLight, system::record_id::RecordId, Color, LightType, RecordCommon, Vector3};

static SYS_RECORD_ID: OnceLock<i32> = OnceLock::new();

#[derive(Default, Hash, PartialEq, Eq, Clone)]
pub struct SysRecordSun {
    pub cast_shadows: bool,
    pub direction: Vector3,
    pub color: Color,
}
impl SysRecordSun {
    pub fn get_draw_call(&self) -> DrawCallLight {
        DrawCallLight {
            light_type: LightType::Directional,
            position: [0.0, 0.0, 0.0],
            direction: [self.direction.x, self.direction.y, self.direction.z],
            color: [self.color.as_r_01(), self.color.as_g_01(), self.color.as_b_01()],
            intensity: 1.0,
            radius: 1.0,
        }
    }
}
impl RecordCommon for SysRecordSun {
    fn id() -> i32 {
        *SYS_RECORD_ID.get_or_init(|| RecordId::of::<SysRecordSun>())
    }
}
