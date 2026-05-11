use crate::{
    game_events::GameEvents,
    state::{
        state_ball_mode::{BallModes, StateBallMode},
        state_teams::StateTeamAssignments,
        state_turn::StateTurn,
    },
};
use curio_core::{
    collections::{event_queue::Nerve, ledger::Ledger},
    network_modes::NetworkModes,
};
use gameplay::{
    context_3d::Context3D,
    traits::{impulse::Impulse, scope::Scope},
};
use impulse::impulse;

#[derive(Default)]
#[impulse(GameEvents)]
pub struct ECsystemGameRequestTurnEnd {}
impl Scope for ECsystemGameRequestTurnEnd {
    fn is_enabled(&mut self, _: &mut Ledger) -> bool {
        true
    }
    fn run_on_instance(&mut self, _: &mut Ledger) -> Vec<NetworkModes> {
        vec![NetworkModes::LocalHost, NetworkModes::OnlineHost]
    }
}
impl Impulse<GameEvents> for ECsystemGameRequestTurnEnd {
    fn dequeue_event(&mut self, ledger: &mut Ledger, _: &mut Context3D, event_queue: &mut Nerve, event: &GameEvents) {
        match event {
            GameEvents::RequestTurnEnd(id) => {
                let Some(team) = ledger.read::<StateTeamAssignments>().team_for(id) else {
                    return;
                };
                //  guard -> make sure the requested end of turn is for the active player
                let is_active_player = ledger.read::<StateTurn>().active_instance_id == team;
                if !is_active_player {
                    println!("Requested Turn End for non-active player");
                    return;
                }

                let is_serving = ledger.read::<StateBallMode>().mode == BallModes::Serve;
                if is_serving {
                    println!("Requested Turn End in Serve Mode");
                    return;
                }

                // end the players turn
                event_queue.enqueue_event(GameEvents::TurnEnd(team));
            }
            _ => {}
        }
    }
}
