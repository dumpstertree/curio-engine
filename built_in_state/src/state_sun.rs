use core::{
    collections::{
        color::Color,
        light_uniform::{DrawCallLight, LightType},
        vector3::Vector3,
    },
    system::system_game_state::IState,
};
use macro_state::global_state;

#[global_state]
pub struct StateSun {
    pub cast_shadows: bool,
    pub direction: Vector3,
    pub color: Color,
}
impl StateSun {
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
impl IState for StateSun {
    fn id() -> i32 {
        98067666
    }
}
