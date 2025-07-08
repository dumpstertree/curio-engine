use hecs::World;

use crate::{
    gameplay::{ecs::component::component_transform::Transform, game_events::GameEvents},
    my_game::ecs::component::component_spin::Spin,
    system::{
        system_components::gameplay_components::gameplay_component_default::{ECSSystem, EngineCommands, EventQueue},
        system_game_states::state_time::TimeState,
    },
    Collections::game_state::GameState,
    Collections::quaternion::Quaternion,
};

pub struct SystemSpin {}
impl SystemSpin {
    pub fn new() -> Box<SystemSpin> {
        Box::new(SystemSpin {})
    }
}
impl ECSSystem<GameEvents> for SystemSpin {
    fn is_enabled(&mut self, game_state: &mut GameState, world: &mut World, event_queue: &mut EventQueue<GameEvents>) -> bool {
        true
    }
    fn tick(&mut self, game_state: &mut GameState, world: &mut World, event_queue: &mut EventQueue<GameEvents>) {
        let t = game_state.get_value2::<TimeState>();
        for (_, (spin, transform)) in world.query::<(&Spin, &mut Transform)>().iter() {
            transform.rotation = Quaternion::from_angle_axis(spin.axis, spin.speed * t.delta_time) * transform.rotation;
        }
    }
}
