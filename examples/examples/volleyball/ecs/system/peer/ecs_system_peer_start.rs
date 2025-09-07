use built_in_state::state_camera::CameraState;
use ecs_system::global_ecs_system;
use hecs::World;

use core::{
    collections::{event_queue::EventQueue, game_state::GameState},
    dumpster_engine::NetworkModes,
    gameplay::ecs::traits::ecs_system::ECSSystemEventless,
};

#[global_ecs_system]
pub struct ECSSystemPeerStart {}
impl ECSSystemEventless for ECSSystemPeerStart {
    fn is_enabled(&mut self, _: &mut GameState, _: &mut World) -> bool {
        true
    }
    fn run_on_instance(&mut self, _: &mut GameState) -> Vec<core::dumpster_engine::NetworkModes> {
        vec![NetworkModes::LocalPeer, NetworkModes::OnlinePeer]
    }
    fn enable(&mut self, game_state: &mut GameState, _: &mut World, _: &mut EventQueue) {
        println!("Instance: {}. Peer Startup", game_state.instance_id);

        // set resolution
        game_state.edit::<CameraState>(|x| {
            x.resolution_width = 1920 / 1;
            x.resolution_height = 1080 / 1;
        });
    }
}
