use crate::exploration::exploration_path::RoomTypes;
use crate::game_events::GameEvents;
use crate::state::host::state_exploration::StateExploration;
use crate::state::peer::state_peer_select_targets::StatePeerSelectTargets;
use curio_core::{
    built_in::record::state_input::InputState,
    collections::{event_queue::EventQueue, game_state::GameState},
    dumpster_engine::NetworkModes,
};
use gameplay::context_3d::Context3D;
use gameplay::traits::{habit::Habit, scope::Scope};
use habit::habit;

#[habit]
pub struct Instance {}
impl Scope for Instance {
    fn run_on_instance(&mut self, _game_state: &mut GameState) -> Vec<NetworkModes> {
        vec![NetworkModes::LocalPeer, NetworkModes::OnlinePeer]
    }
    fn is_enabled(&mut self, game_state: &mut GameState) -> bool {
        game_state
            .get::<StateExploration>()
            .exploration
            .get_cur_room()
            .room_type
            == RoomTypes::Combat
            && !game_state.get::<StateExploration>().is_selecting_next
            && game_state.get::<StatePeerSelectTargets>().enabled.is_none()
        // game_state.get::<StateTurn>().active_instance_id
        //     == game_state
        //         .get::<StateTeamAssignments>()
        //         .team_for(&game_state.instance_id)
        //         .unwrap()
    }
}
impl Habit for Instance {
    fn enable(&mut self, _: &mut GameState, _: &mut Context3D, _: &mut EventQueue) {
        println!("enabled turn end");
    }
    fn tick(&mut self, game_state: &mut GameState, _: &mut Context3D, events: &mut EventQueue) {
        // get input
        let state_input = game_state.get::<InputState>();

        // guard - input for next turn
        let input_next = state_input.mapped[0]
            .get_button_or_default("turn_end")
            .went_up;
        if !input_next {
            return;
        }

        // send event to end turn
        events.enqueue_event(GameEvents::RequestTurnEnd(game_state.instance_id));
    }
}
