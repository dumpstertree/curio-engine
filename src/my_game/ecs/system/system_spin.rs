use ecs_system::ECSSystem;
use hecs::World;

use crate::{
    gameplay::ecs::{component::component_transform::Transform, traits::ecs_system::ECSSystemEventless},
    my_game::ecs::component::component_spin::Spin,
    system::system_game_states::state_time::TimeState,
    Collections::{event_queue::EventQueue2, game_state::GameState, quaternion::Quaternion},
};

#[ECSSystem]
pub struct SystemSpin {}
impl SystemSpin {
    pub fn new() -> Box<SystemSpin> {
        Box::new(SystemSpin {})
    }
}
impl ECSSystemEventless for SystemSpin {
    fn is_enabled(&mut self, game_state: &mut GameState, world: &mut World) -> bool {
        true
    }
    fn tick(&mut self, state: &mut GameState, world: &mut World, events: &mut EventQueue2) {
        let t = state.get_value2::<TimeState>();
        for (_, (spin, transform)) in world.query::<(&Spin, &mut Transform)>().iter() {
            transform.rotation = Quaternion::from_angle_axis(spin.axis, spin.speed * t.scaled_delta_time) * transform.rotation;
        }
    }
}
