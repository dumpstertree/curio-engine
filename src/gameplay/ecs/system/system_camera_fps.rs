use crate::Collections::game_state::GameState;
use hecs::World;

use crate::{
    gameplay::ecs::component::{component_camera::Camera, component_transform::Transform},
    system::{
        system_components::gameplay_components::gameplay_component_default::ECSSystemEventless,
        system_game_states::{state_input::InputState, state_time::TimeState},
    },
    Collections::vector3::Vector3,
};

pub struct FPSCameraECSSystem {
    enabled: bool,
}
impl FPSCameraECSSystem {
    pub fn new() -> FPSCameraECSSystem {
        FPSCameraECSSystem { enabled: false }
    }
}
impl ECSSystemEventless for FPSCameraECSSystem {
    fn is_enabled(&mut self, game_state: &mut GameState, world: &mut World) -> bool {
        true
    }
    fn tick(&mut self, game_state: &mut GameState, world: &mut World) {
        // get the input direction
        let input = game_state.get_value2::<InputState>();

        // flip enabled
        if input.debug.went_down {
            println!("flip");
            self.enabled = !self.enabled;
            return;
        }

        if !self.enabled {
            return;
        }

        //
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

        let t = game_state.get_value2::<TimeState>();
        // alter the speed
        let speed = 10.0;
        let offset = dir * speed * t.delta_time;

        for (_, (t, _)) in world.query_mut::<(&mut Transform, &Camera)>() {
            t.position = t.position.clone() + offset.clone();
        }
    }
}
