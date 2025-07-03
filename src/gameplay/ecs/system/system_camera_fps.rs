use hecs::World;

use crate::{
    game_state::GameState,
    gameplay::{
        ecs::component::{component_camera::Camera, component_transform::Transform},
        game_events::GameEvents,
    },
    system::system_components::gameplay_components::gameplay_component_default::{ECSSystem, EventQueue},
    Collections::vector3::Vector3,
};

pub struct FPSCameraECSSystem {}
impl FPSCameraECSSystem {}
impl ECSSystem<GameEvents> for FPSCameraECSSystem {
    fn tick(&mut self, game_state: &mut GameState, world: &mut World, event_queue: &mut EventQueue<GameEvents>) {
        // get the input direction
        let input = game_state.get_input();
        let mut dir = Vector3::zero();
        if input.w.is_down {
            dir = dir + Vector3::forward();
        }
        if input.s.is_down {
            dir = dir + Vector3::back();
        }
        if input.d.is_down {
            dir = dir + Vector3::right();
        }
        if input.a.is_down {
            dir = dir + Vector3::left();
        }

        let t = game_state.get_time();
        // alter the speed
        let speed = 10.0;
        let offset = dir * speed * t.delta_time;

        for (_, (t, _)) in world.query_mut::<(&mut Transform, &Camera)>() {
            t.position = t.position.clone() + offset.clone();
        }
    }
}
