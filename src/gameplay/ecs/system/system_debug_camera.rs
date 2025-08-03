use std::default;

use crate::system::system_components::gameplay_components::gameplay_component_default::ECSSystemEventless;
use crate::system::system_game_states::state_screeen::StateScreen;
use crate::{
    system::{
        system_components::gameplay_components::gameplay_component_default::{EngineCommands, EventQueue},
        system_game_states::{state_camera::CameraState, state_debug::StateDebug, state_input::InputState, state_time::TimeState},
    },
    Collections::{game_state::GameState, quaternion::Quaternion, vector3::Vector3},
};
use ecs_system::ECSSystem;
use hecs::World;

#[derive(ECSSystem)]

// pub struct thing;
pub struct SystemDebugCamera {
    x: f32,
    y: f32,
}
impl SystemDebugCamera {
    pub fn new() -> Box<SystemDebugCamera> {
        Box::new(SystemDebugCamera { x: 0.0, y: 0.0 })
    }
}
impl ECSSystemEventless for SystemDebugCamera {
    fn is_enabled(&mut self, game_state: &mut GameState, _: &mut World) -> bool {
        game_state.get_value2::<StateDebug>().is_paused
    }
    fn enable(&mut self, _: &mut GameState, _: &mut World) {
        self.x = 0.0;
        self.y = 0.0;
    }
    fn debug(&mut self, game_state: &mut GameState, _: &mut World, _: &mut EventQueue<EngineCommands>) {
        // constants
        const SPEED_ROT: f32 = 10.0;
        const SPEED_MOVE: f32 = 10.0;

        // get states
        let state_time = game_state.get_value2::<TimeState>();
        let state_input = game_state.get_value2::<InputState>();
        let state_screen = game_state.get_value2::<StateScreen>();

        // rotation
        if state_input.cursor_primary.is_down {
            //  calculate the new angles
            let x_angle = (-state_input.cursor.delta.x / state_screen.width() as f32) * SPEED_ROT * state_time.unscaled_delta_time;
            let y_angle = (-state_input.cursor.delta.y / state_screen.height() as f32) * SPEED_ROT * state_time.unscaled_delta_time;

            // update the saved values
            self.x = self.x + x_angle;
            self.y = self.y + y_angle;
        }
        // calculate the rotation
        let rot = Quaternion::from_angle_axis(Vector3::up(), self.x) * Quaternion::from_angle_axis(Vector3::right(), self.y);

        // position
        let mut dir = Vector3::zero();
        if state_input.w.is_down {
            dir = dir + rot * Vector3::forward();
        }
        if state_input.s.is_down {
            dir = dir + rot * Vector3::back();
        }
        if state_input.d.is_down {
            dir = dir + rot * Vector3::right();
        }
        if state_input.a.is_down {
            dir = dir + rot * Vector3::left();
        }

        // alter the speed
        let offset = dir * SPEED_MOVE * state_time.unscaled_delta_time;

        // edit the state
        game_state.edit::<CameraState>(|x| {
            x.position = x.position + offset;
            x.rotation = rot;
        });
    }
}
impl Default for SystemDebugCamera {
    fn default() -> Self {
        Self {
            x: Default::default(),
            y: Default::default(),
        }
    }
}
