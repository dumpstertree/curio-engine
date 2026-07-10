use crate::{DrawCallLight, LightType};
use curio_core::{Color, FieldState, RecordOverride, Vector3};
use record::record;

#[record(name = "Sun", ownership = curio_core::RecordScope::Instance)]
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
impl RecordOverride for SysRecordSun {
    fn set_state(&mut self, _: &str, _: &str) {}
    fn get_state(&self) -> Vec<FieldState> {
        vec![
            FieldState::new("cast_shadows", self.cast_shadows), //
            FieldState::new("direction", self.direction),
            FieldState::new("color", self.color),
        ]
    }
}
