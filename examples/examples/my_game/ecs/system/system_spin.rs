use built_in::component::component_transform::Transform;
use built_in_state::state_time::TimeState;
use ecs_system::global_ecs_system;
use hecs::World;

use core::{
    collections::{event_queue::EventQueue, game_state::GameState, quaternion::Quaternion},
    gameplay::ecs::traits::ecs_system::ECSSystemEventless,
};

use crate::ecs::component::component_spin::Spin;

#[global_ecs_system]
pub struct SystemSpin {}
impl ECSSystemEventless for SystemSpin {
    fn is_enabled(&mut self, _: &mut GameState, _: &mut World) -> bool {
        true
    }
    fn tick(&mut self, state: &mut GameState, world: &mut World, _: &mut EventQueue) {
        let t = state.get::<TimeState>();
        for (_, (spin, transform)) in world.query::<(&Spin, &mut Transform)>().iter() {
            transform.rotation = Quaternion::from_angle_axis(spin.axis, spin.speed * t.scaled_delta_time) * transform.rotation;
        }
    }
}
