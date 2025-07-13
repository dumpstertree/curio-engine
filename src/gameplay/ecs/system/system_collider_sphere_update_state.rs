use crate::Collections::game_state::GameState;
use hecs::World;

use crate::system::system_components::gameplay_components::gameplay_component_default::ECSSystemEventless;

pub struct SystemColliderBoxUpdateState {}
impl SystemColliderBoxUpdateState {
    pub fn new() -> Box<SystemColliderBoxUpdateState> {
        Box::new(SystemColliderBoxUpdateState {})
    }
}
impl ECSSystemEventless for SystemColliderBoxUpdateState {
    fn is_enabled(&mut self, game_state: &mut GameState, world: &mut World) -> bool {
        true
    }

    fn will_tick(&mut self, game_state: &mut GameState, world: &mut World) {}
    fn did_tick(&mut self, game_state: &mut GameState, scene: &mut World) {}
}
