use crate::exploration::exploration_path::RoomTypes;
use crate::game_events::GameEvents;
use crate::state::host::state_exploration::StateExploration;
use crate::state::peer::state_peer_select_targets::StatePeerSelectTargets;
use curio_core::{
    built_in::record::sys_record_input::SysRecordInput,
    collections::{event_queue::EventQueue, ledger::Ledger},
    network_modes::NetworkModes,
};
use gameplay::context_3d::Context3D;
use gameplay::traits::{habit::Habit, scope::Scope};
use habit::habit;

#[habit]
pub struct Instance {}
impl Scope for Instance {
    fn run_on_instance(&mut self, _ledger: &mut Ledger) -> Vec<NetworkModes> {
        vec![NetworkModes::LocalPeer, NetworkModes::OnlinePeer]
    }
    fn is_enabled(&mut self, ledger: &mut Ledger) -> bool {
        ledger
            .read::<StateExploration>()
            .exploration
            .get_cur_room()
            .room_type
            == RoomTypes::Combat
            && !ledger.read::<StateExploration>().is_selecting_next
            && ledger.read::<StatePeerSelectTargets>().enabled.is_none()
        // ledger.get::<StateTurn>().active_instance_id
        //     == ledger
        //         .get::<StateTeamAssignments>()
        //         .team_for(&ledger.network.me().guid)
        //         .unwrap()
    }
}
impl Habit for Instance {
    fn enable(&mut self, _: &mut Ledger, _: &mut Context3D, _: &mut EventQueue) {
        println!("enabled turn end");
    }
    fn tick(&mut self, ledger: &mut Ledger, _: &mut Context3D, events: &mut EventQueue) {
        // get input
        let state_input = ledger.read::<SysRecordInput>();

        // guard - input for next turn
        let input_next = state_input.mapped[0]
            .get_button_or_default("turn_end")
            .went_up;
        if !input_next {
            return;
        }

        // send event to end turn
        events.enqueue_event(GameEvents::RequestTurnEnd(ledger.network.me().guid));
    }
}
