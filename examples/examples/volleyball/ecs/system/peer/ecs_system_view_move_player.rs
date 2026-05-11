use crate::ecs::components::component_player::ComponentPlayer;
use crate::ecs::components::component_view_player::ComponentViewPlayer;
use crate::game_board::GameBoard;
use crate::state::state_position_player::StatePositionEntities;
use curio_core::built_in::record::sys_record_time::SysRecordTime;
use curio_core::collections::{event_queue::Nerve, ledger::Ledger};
use curio_core::network_modes::NetworkModes;
use curio_core::{Quaternion, Vector3};
use gameplay::built_in::facet::renderer::renderer_dynamic::RendererDynamic;
use gameplay::built_in::facet::transform::transform3d::Transform3D;
use gameplay::context_3d::Context3D;
use gameplay::traits::habit::Habit;
use gameplay::traits::scope::Scope;
use gameplay::traits_internal::world_context_common::ContextCommon;
use habit::habit;

#[habit]
pub struct Instance {}
impl Scope for Instance {
    fn is_enabled(&mut self, _ledger: &mut Ledger) -> bool {
        true
    }
    fn run_on_instance(&mut self, _ledger: &mut Ledger) -> Vec<NetworkModes> {
        NetworkModes::all_peer()
    }
}
impl Habit for Instance {
    fn tick(&mut self, ledger: &mut Ledger, world: &mut Context3D, _events: &mut Nerve) {
        let state_position_player = ledger.read::<StatePositionEntities>();
        let state_time = ledger.read::<SysRecordTime>();

        world.edit::<(&mut Transform3D, &ComponentPlayer, &ComponentViewPlayer, &mut RendererDynamic)>(|query| {
            for (_, (transform, player, _, renderer)) in query {
                let Some(loc) = state_position_player.positions.get(&player.player_id) else {
                    continue;
                };

                // get pos
                let cur_pos = transform.position;
                let tar_pos = GameBoard::get_world_position(loc.0, loc.1);

                //move towards position and get back the delta
                let move_delta = transform.move_towards_position(tar_pos, 10.0 * state_time.scaled_delta_time);
                if move_delta > 0.0 {
                    // rotate mesh based on direction
                    let move_left = (tar_pos - cur_pos).x > 0.0;
                    if move_left {
                        transform.rotation = Quaternion::from_euler(Vector3::new(0.0, 180.0, 0.0));
                    } else {
                        transform.rotation = Quaternion::from_euler(Vector3::new(0.0, 0.0, 0.0));
                    }

                    // apply walking animation
                    renderer.set_animation("run", true);
                } else {
                    // clear animations
                    renderer.set_animation("idle", true);
                }
            }
        });
    }
}
