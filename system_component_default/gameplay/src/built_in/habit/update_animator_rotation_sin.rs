use crate::{
    built_in::facet::{animator::animator_rotation_sin::AnimatorRotationSin, transform::transform2d::Transform2D},
    context_3d::Context3D,
    traits::{habit::Habit, scope::Scope},
    traits_internal::world_context_common::ContextCommon,
};
use curio_core::{
    Quaternion,
    built_in::record::sys_record_time::SysRecordTime,
    collections::{event_queue::EventQueue, game_state::GameState, network_modes::NetworkModes},
};

#[derive(Default)]
pub struct Instance {}
impl Instance {
    pub fn new() -> Box<Instance> {
        Box::new(Instance {})
    }
}
impl Scope for Instance {
    fn is_enabled(&mut self, _game_state: &mut GameState) -> bool {
        true
    }
    fn run_on_instance(&mut self, _game_state: &mut GameState) -> Vec<NetworkModes> {
        NetworkModes::all()
    }
}
impl Habit for Instance {
    fn tick(&mut self, game_state: &mut GameState, context3d: &mut Context3D, _: &mut EventQueue) {
        // get state
        let state_time = game_state.get::<SysRecordTime>();
        let time = state_time.scaled_time as f32;

        // edit the forms in context
        context3d.edit::<(&AnimatorRotationSin, &mut Transform2D)>(|x| {
            //
            for (_, (animator, transform2d)) in x {
                // if enabled update
                if animator.enabled() {
                    let scaled_time = animator.speed() * time;
                    let sin = f32::sin(scaled_time);
                    let max = animator.max();
                    let min = animator.min();
                    let delta = max - min;
                    let target = min + (delta * ((sin + 1.0) / 2.0));
                    transform2d.rotation = Quaternion::from_euler(target);
                } else {
                    let max = animator.max();
                    let min = animator.min();
                    transform2d.rotation = Quaternion::from_euler(min + (max - min) / 2.0);
                }
            }
        });
    }
}
