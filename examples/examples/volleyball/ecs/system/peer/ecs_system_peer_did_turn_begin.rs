use crate::{
    ai::{
        ai_simulator::AISimulator,
        dependencies::{simulation_data_sources::custom_data_source::CustomDataSource, simulation_delegates::custom_delegate::CustomDelegate, simulation_evaluators::custom_evaluator::CustomEvaluator, simulation_hashers::custom_hasher::CustomHasher},
        enums::{fidelity::Fidelity, threading::Threading},
    },
    cards::enums::simulation_manuevers::SimulationManuevers,
    exploration::exploration_path::RoomTypes,
    game_events::GameEvents,
    listeners::listener_initialize_encounter::Controller,
    state::{
        host::{state_card_attribute_modifier_stack::StateCardAttributeModifierStack, state_exploration::StateExploration},
        other::state_terminated::StateTerminated,
        state_ball_mode::StateBallMode,
        state_controller::StateController,
        state_deck::StateDeck,
        state_energy::StateEnergy,
        state_position_ball::StatePositionBall,
        state_position_player::StatePositionEntities,
        state_score::StateScore,
        state_turn::StateTurn,
    },
};
use built_in_state::state_time::TimeState;
use core::{
    collections::{event_queue::EventQueue, game_state::GameState},
    dumpster_engine::NetworkModes,
    system::system_game_state::IState,
};
use ecs_event::impulse;
use ecs_system::habit;
use std::vec;
use system_component_default_gameplay::{
    traits::{habit::Habit, impulse::Impulse, scope::Scope},
    world_context_3d::WorldContext,
};

use crate::state::state_teams::StateTeamAssignments;

#[habit]
#[impulse(GameEvents)]
pub struct Instance {
    lastmove: f64,
    move_time: f64,
}
impl Scope for Instance {
    fn is_enabled(&mut self, game_state: &mut GameState) -> bool {
        let state_team = game_state.get::<StateTeamAssignments>();
        let active_team = game_state.get::<StateTurn>().active_instance_id;
        let Some(current_guids) = state_team.team_assignments.get(&active_team) else {
            return false;
        };

        let any_ai = game_state
            .get::<StateController>()
            .all_players
            .iter()
            .any(|x| current_guids.contains(x.0) && x.1 == &Controller::Ai);

        game_state
            .get::<StateExploration>()
            .exploration
            .get_cur_room()
            .room_type
            == RoomTypes::Combat
            && any_ai
    }
    fn run_on_instance(&mut self, game_state: &mut GameState) -> Vec<NetworkModes> {
        NetworkModes::all_host()
    }
}
impl Habit for Instance {
    fn init(&mut self, game_state: &mut GameState, world: &mut WorldContext, _: &mut EventQueue) {}
    fn enable(&mut self, game_state: &mut GameState, world: &mut WorldContext, _: &mut EventQueue) {
        self.move_time = 3.0;
        self.lastmove = game_state.get::<TimeState>().scaled_time;
    }
    fn tick(&mut self, game_state: &mut GameState, _: &mut WorldContext, events: &mut EventQueue) {
        let state_score = game_state.get::<StateScore>();
        if state_score.all_scores.iter().any(|x| *x.1 <= 0) {
            return;
        }
        let state_team = game_state.get::<StateTeamAssignments>();
        let active_team = game_state.get::<StateTurn>().active_instance_id;
        let Some(current_guids) = state_team.team_assignments.get(&active_team) else {
            return;
        };

        let any_ai = game_state
            .get::<StateController>()
            .all_players
            .iter()
            .any(|x| current_guids.contains(x.0) && x.1 == &Controller::Ai);

        // let is_turn = d == game_state.instance_id;
        if unsafe { DO_MOVE } && any_ai && game_state.get::<TimeState>().scaled_time - self.lastmove > self.move_time {
            unsafe { DO_MOVE = false };

            let simulator = AISimulator::new(Box::new(CustomDelegate {}), Box::new(CustomDataSource {}), Box::new(CustomHasher {}), Box::new(CustomEvaluator {}), |game_state| {
                GameState::new_single_instance(vec![
                    // copy these states
                    (StateCardAttributeModifierStack::id(), Box::new(game_state.get::<StateCardAttributeModifierStack>())),
                    (StateTeamAssignments::id(), Box::new(game_state.get::<StateTeamAssignments>())),
                    (StatePositionEntities::id(), Box::new(game_state.get::<StatePositionEntities>())), //
                    (StatePositionBall::id(), Box::new(game_state.get::<StatePositionBall>())),
                    (StateBallMode::id(), Box::new(game_state.get::<StateBallMode>())),
                    (StateEnergy::id(), Box::new(game_state.get::<StateEnergy>())),
                    (StateDeck::id(), Box::new(game_state.get::<StateDeck>())),
                    (StateTurn::id(), Box::new(game_state.get::<StateTurn>())),
                    // add this state
                    (StateTerminated::id(), Box::new(StateTerminated { is_terminated: false, is_exhuasted: false })),
                ])
            });

            // let uid = current_guids[0];
            let move2 = simulator.simulate(game_state, Fidelity::Medium, Threading::Multi);
            let uid = move2.0;
            match move2.1 {
                SimulationManuevers::EndTurn => {
                    events.enqueue_event(GameEvents::RequestTurnEnd(uid));
                }
                SimulationManuevers::PlayCard(card_instance, filled_card_response) => {
                    match card_instance.get_manuever_type() {
                        crate::state::state_deck::CardTypes::Move => self.move_time = 0.5,
                        _ => self.move_time = 3.0,
                    }
                    events.enqueue_event(GameEvents::RequestUseManeuverPersistent(uid, card_instance.instance_id, filled_card_response));
                }
                // SimulationManuevers::MoveEntity(direction) => match direction {
                //     Directions::Forward => events.enqueue_event(GameEvents::RequestMoveZPos(uid)),
                //     Directions::Back => events.enqueue_event(GameEvents::RequestMoveZNeg(uid)),
                //     Directions::Left => events.enqueue_event(GameEvents::RequestMoveXNeg(uid)),
                //     Directions::Right => events.enqueue_event(GameEvents::RequestMoveXPos(uid)),
                // },
                _ => {}
            }

            // let e = run_ai(game_state);
            // events.enqueue_event(e);
            self.lastmove = game_state.get::<TimeState>().scaled_time;
            println!(
                "did ai
                        
                        
                        
                        "
            );
        }
    }
}
// impl Scope for Instance {
//     fn is_enabled(&mut self, _: &mut GameState) -> bool {
//         true
//     }
//     fn run_on_instance(&mut self, _: &mut GameState) -> Vec<core::dumpster_engine::NetworkModes> {
//         NetworkModes::all_host()
//     }
// }
impl Impulse<GameEvents> for Instance {
    fn dequeue_event(&mut self, game_state: &mut GameState, _: &mut WorldContext, event_queue: &mut EventQueue, event: &GameEvents) {
        match event {
            GameEvents::DidTurnBegin(id) => {}
            _ => {
                let state_time = game_state.get::<TimeState>();
                // self.lastmove = game_state.get::<TimeState>().unscaled_time;
                unsafe { DO_MOVE = true };
            }
        }
    }
}

static mut DO_MOVE: bool = false;
