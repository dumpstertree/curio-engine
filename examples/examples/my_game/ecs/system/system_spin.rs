use built_in::component::component_transform::Transform;
use ecs_system::global_ecs_system;
use hecs::World;

use core::{
    collections::{event_queue::EventQueue2, game_state::GameState, quaternion::Quaternion},
    gameplay::ecs::traits::ecs_system::ECSSystemEventless,
    system::system_game_states::state_time::TimeState,
};

use crate::ecs::component::component_spin::Spin;

#[global_ecs_system]
pub struct SystemSpin {}
impl ECSSystemEventless for SystemSpin {
    fn is_enabled(&mut self, _: &mut GameState, _: &mut World) -> bool {
        true
    }
    fn tick(&mut self, state: &mut GameState, world: &mut World, _: &mut EventQueue2) {
        let t = state.get_value2::<TimeState>();
        for (_, (spin, transform)) in world.query::<(&Spin, &mut Transform)>().iter() {
            transform.rotation = Quaternion::from_angle_axis(spin.axis, spin.speed * t.scaled_delta_time) * transform.rotation;
        }
    }
}
