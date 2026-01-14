use crate::{
    game_events::GameEvents,
    state::{
        state_ball_mode::{BallModes, StateBallMode},
        state_teams::StateTeamAssignments,
        state_turn::StateTurn,
    },
};
use curio_core::{
    collections::{event_queue::EventQueue, game_state::GameState},
    dumpster_engine::NetworkModes,
};
use ecs_event::impulse;
use ecs_system::habit;
use system_component_default_gameplay::{
    context_3d::Context3D,
    traits::{habit::Habit, impulse::Impulse, scope::Scope},
};

#[derive(Default)]
#[impulse(GameEvents)]
pub struct ECSSystemGameRequestTurnEnd {}
impl Scope for ECSSystemGameRequestTurnEnd {
    fn is_enabled(&mut self, _: &mut GameState) -> bool {
        true
    }
    fn run_on_instance(&mut self, _: &mut GameState) -> Vec<NetworkModes> {
        vec![NetworkModes::LocalHost, NetworkModes::OnlineHost]
    }
}
impl Impulse<GameEvents> for ECSSystemGameRequestTurnEnd {
    fn dequeue_event(&mut self, game_state: &mut GameState, _: &mut Context3D, event_queue: &mut EventQueue, event: &GameEvents) {
        match event {
            GameEvents::RequestTurnEnd(id) => {
                let Some(team) = game_state.get::<StateTeamAssignments>().team_for(id) else {
                    return;
                };
                //  guard -> make sure the requested end of turn is for the active player
                let is_active_player = game_state.get::<StateTurn>().active_instance_id == team;
                if !is_active_player {
                    println!("Requested Turn End for non-active player");
                    return;
                }

                let is_serving = game_state.get::<StateBallMode>().mode == BallModes::Serve;
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
