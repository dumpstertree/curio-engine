use built_in_state::state_camera::CameraState;
use built_in_state::state_debug::StateDebug;
use built_in_state::state_input::InputState;
use built_in_state::state_screeen::StateScreen;
use built_in_state::state_time::TimeState;
use core::collections::event_queue::EventQueue;
use core::collections::{game_state::GameState, quaternion::Quaternion, vector3::Vector3};
use core::gameplay::ecs::traits::ecs_system::ECSSystemEventless;
use core::input::axis_code::AxisCode;
use core::input::key_code::KeyCode;
use ecs_system::global_ecs_system;
use hecs::World;

#[global_ecs_system]
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
        game_state.get_value2::<StateDebug>().is_inspecting && game_state.get_value2::<StateDebug>().is_paused
    }
    fn enable(&mut self, _: &mut GameState, _: &mut World, _: &mut EventQueue) {
        self.x = 0.0;
        self.y = 0.0;
    }
    fn tick(&mut self, state: &mut GameState, _: &mut World, _: &mut EventQueue) {
        // constants
        const SPEED_ROT: f32 = 10.0;
        const SPEED_MOVE: f32 = 10.0;

        // get states
        let state_time = state.get_value2::<TimeState>();
        let state_input = state.get_value2::<InputState>();
        let state_screen = state.get_value2::<StateScreen>();

        // rotation
        if state_input.raw.get_button(&KeyCode::MousePrimary).is_down {
            //  calculate the new angles
            let x_angle = (-state_input.raw.get_axis(&AxisCode::Cursor).delta.x / state_screen.width() as f32) * SPEED_ROT * state_time.unscaled_delta_time;
            let y_angle = (-state_input.raw.get_axis(&AxisCode::Cursor).delta.y / state_screen.height() as f32) * SPEED_ROT * state_time.unscaled_delta_time;

            // update the saved values
            self.x = self.x + x_angle;
            self.y = self.y + y_angle;
        }
        // calculate the rotation
        let rot = Quaternion::from_angle_axis(Vector3::up(), self.x) * Quaternion::from_angle_axis(Vector3::right(), self.y);

        // position
        let mut dir = Vector3::zero();
        if state_input.raw.get_button(&KeyCode::KeyW).is_down {
            dir = dir + rot * Vector3::forward();
        }
        if state_input.raw.get_button(&KeyCode::KeyS).is_down {
            dir = dir + rot * Vector3::back();
        }
        if state_input.raw.get_button(&KeyCode::KeyD).is_down {
            dir = dir + rot * Vector3::right();
        }
        if state_input.raw.get_button(&KeyCode::KeyA).is_down {
            dir = dir + rot * Vector3::left();
        }

        // alter the speed
        let offset = dir * SPEED_MOVE * state_time.unscaled_delta_time;

        // edit the state
        state.edit::<CameraState>(|x| {
            x.cameras.position = x.cameras.position + offset;
            x.cameras.rotation = rot;
        });

        println!("edit camera");
    }
}
