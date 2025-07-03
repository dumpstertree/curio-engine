use crate::{
    game_state,
    system::{system_component::ISystemComponent, system_components::graphics_components::graphics_component_wgpu::DrawCallsState},
    Window::CameraState::{self},
};
pub trait IGraphicsComponent: ISystemComponent {}

const KEY: i32 = 123;
const KEY_DRAW_CALLS: i32 = 124;
impl game_state::GameState {
    pub fn set_camera(&mut self, state: CameraState::CameraState) {
        self.add(KEY, state);
    }
    pub fn get_camera(&self) -> CameraState::CameraState {
        if !self.has_value(KEY) {
            return CameraState::CameraState::default();
            // self.add(KEY, );
        }
        let x = self.get_value::<CameraState::CameraState>(KEY);
        x.unwrap().clone()
    }
    pub fn set_draw_calls(&mut self, state: DrawCallsState) {
        self.add(KEY_DRAW_CALLS, state);
    }
    pub fn get_draw_calls(&self) -> DrawCallsState {
        if !self.has_value(KEY_DRAW_CALLS) {
            return DrawCallsState::new();
            // println!("add draw calls");
            // self.add(KEY_DRAW_CALLS, DrawCallsState::new());
        }
        let x = self.get_value::<DrawCallsState>(KEY_DRAW_CALLS);
        x.unwrap().clone()
    }
}
