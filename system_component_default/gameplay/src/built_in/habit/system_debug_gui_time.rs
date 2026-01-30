use curio_core::{
    built_in::record::{sys_record_debug::SysRecordDebug, sys_record_debug_gui::SysRecordDebugGui, sys_record_time::SysRecordTime},
    collections::{event_queue::EventQueue, game_state::GameState},
    dumpster_engine::NetworkModes,
};

use crate::{
    context_3d::Context3D,
    traits::{habit::Habit, scope::Scope},
};

#[derive(Default)]
pub struct Instance {}
impl Instance {
    pub fn new() -> Box<Instance> {
        Box::new(Instance {})
    }
}
impl Scope for Instance {
    fn is_enabled(&mut self, game_state: &mut GameState) -> bool {
        game_state.get::<SysRecordDebug>().is_inspecting
    }
    fn run_on_instance(&mut self, _game_state: &mut GameState) -> Vec<NetworkModes> {
        NetworkModes::all()
    }
}
impl Habit for Instance {
    fn tick(&mut self, game_state: &mut GameState, _: &mut Context3D, _: &mut EventQueue) {
        // get state
        let state_time = game_state.get::<SysRecordTime>();

        game_state.edit::<SysRecordDebugGui>(|x| {
            x.append(format!("FPS: {} / Target FPS: {}", state_time.average_fps, state_time.target_frame_rate));
            x.append(format!("Scaled Time: {}", state_time.scaled_time));
            x.append(format!("Unscaled Time: {}", state_time.unscaled_time));
            x.append(format!("Frame Num: {}", state_time.frame_num));
        });
    }
}
