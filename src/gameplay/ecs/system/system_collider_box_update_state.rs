use crate::{
    gameplay::ecs::component::{
        component_collider::ColliderState, component_colliders::component_collider_box::ComponentColliderBox, component_transform::Transform,
    },
    system::system_game_states::state_colliders::StateCollider,
    Collections::game_state::GameState,
};
use hecs::World;

use crate::system::system_components::gameplay_components::gameplay_component_default::ECSSystemEventless;

pub struct SystemColliderSphereUpdateState {}
impl SystemColliderSphereUpdateState {
    pub fn new() -> Box<SystemColliderSphereUpdateState> {
        Box::new(SystemColliderSphereUpdateState {})
    }
}
impl ECSSystemEventless for SystemColliderSphereUpdateState {
    fn is_enabled(&mut self, game_state: &mut GameState, world: &mut World) -> bool {
        true
    }

    fn will_tick(&mut self, game_state: &mut GameState, world: &mut World) {
        let mut state = game_state.get_value2::<StateCollider>();
        for (_, (collider, transform)) in world.query::<(&ComponentColliderBox, &Transform)>().iter() {
            state.colliders.push(ColliderState::new_box(0));
        }
        game_state.set_value2::<StateCollider>(state);
    }
    fn did_tick(&mut self, game_state: &mut GameState, scene: &mut World) {}
}
