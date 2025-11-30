use crate::{
    ai::{
        ai_simulator::AISimulator,
        dependencies::{simulation_data_sources::custom_data_source::CustomDataSource, simulation_delegates::custom_delegate::CustomDelegate, simulation_evaluators::custom_evaluator::CustomEvaluator, simulation_hashers::custom_hasher::CustomHasher},
        enums::{fidelity::Fidelity, threading::Threading},
    },
    cards::enums::simulation_manuevers::SimulationManuevers,
    game_board::Directions,
    game_events::GameEvents,
    state::{
        host::state_card_attribute_modifier_stack::StateCardAttributeModifierStack, other::state_terminated::StateTerminated, state_ball_mode::StateBallMode, state_deck::StateDeck, state_energy::StateEnergy, state_position_ball::StatePositionBall, state_position_player::StatePositionPlayer,
        state_turn::StateTurn,
    },
};
use built_in_state::state_time::TimeState;
use core::{
    collections::{event_queue::EventQueue, game_state::GameState},
    dumpster_engine::NetworkModes,
    gameplay::ecs::traits::{
        ecs_event_reciever::{self, InstanceLimiter},
        ecs_system::ECSSystemEventless,
    },
    system::system_game_state::IState,
};
use ecs_event::global_ecs_system_event_reciever;
use ecs_system::global_ecs_system;
use hecs::World;

use crate::{
    AssetMappingUIDs,
    ecs::components::{component_ball::ComponentBall, component_player::ComponentPlayer, component_view_player::ComponentViewPlayer},
    state::state_teams::{StateTeamAssignments, Teams},
};

#[global_ecs_system]
#[global_ecs_system_event_reciever(GameEvents)]
pub struct ECSSystemPeerStart {
    do_move: bool,
    lastmove: f64,
}
impl ECSSystemEventless for ECSSystemPeerStart {
    fn is_enabled(&mut self, _: &mut GameState, _: &mut World) -> bool {
        true
    }
    fn run_on_instance(&mut self, _: &mut GameState) -> Vec<core::dumpster_engine::NetworkModes> {
        vec![NetworkModes::LocalPeer, NetworkModes::OnlinePeer]
    }
    fn init(&mut self, game_state: &mut GameState, world: &mut World, _: &mut EventQueue) {}
    fn enable(&mut self, game_state: &mut GameState, world: &mut World, _: &mut EventQueue) {}
    fn tick(&mut self, game_state: &mut GameState, _: &mut World, events: &mut EventQueue) {
        let is_turn = game_state.get::<StateTurn>().active_instance_id == game_state.instance_id;
        if is_turn && game_state.get::<TimeState>().unscaled_time - self.lastmove > 1.0 {
            let simulator = AISimulator::new(Box::new(CustomDelegate {}), Box::new(CustomDataSource {}), Box::new(CustomHasher {}), Box::new(CustomEvaluator {}), |game_state| {
                GameState::new_single_instance(vec![
                    // copy these states
                    (StateCardAttributeModifierStack::id(), Box::new(game_state.get::<StateCardAttributeModifierStack>())),
                    (StateTeamAssignments::id(), Box::new(game_state.get::<StateTeamAssignments>())),
                    (StatePositionPlayer::id(), Box::new(game_state.get::<StatePositionPlayer>())), //
                    (StatePositionBall::id(), Box::new(game_state.get::<StatePositionBall>())),
                    (StateBallMode::id(), Box::new(game_state.get::<StateBallMode>())),
                    (StateEnergy::id(), Box::new(game_state.get::<StateEnergy>())),
                    (StateDeck::id(), Box::new(game_state.get::<StateDeck>())),
                    (StateTurn::id(), Box::new(game_state.get::<StateTurn>())),
                    // add this state
                    (StateTerminated::id(), Box::new(StateTerminated { is_terminated: false, is_exhuasted: false })),
                ])
            });

            let uid = game_state.get::<StateTurn>().active_instance_id;
            let move2 = simulator.simulate(game_state, Fidelity::Medium, Threading::Multi);

            match move2 {
                SimulationManuevers::EndTurn => events.enqueue_event(GameEvents::RequestTurnEnd(uid)),
                SimulationManuevers::PlayCard(card_instance, filled_card_response) => events.enqueue_event(GameEvents::RequestUseManeuverPersistent(uid, card_instance.instance_id, filled_card_response)),
                SimulationManuevers::MoveEntity(direction) => match direction {
                    Directions::Forward => events.enqueue_event(GameEvents::RequestMoveZPos(uid)),
                    Directions::Back => events.enqueue_event(GameEvents::RequestMoveZNeg(uid)),
                    Directions::Left => events.enqueue_event(GameEvents::RequestMoveXNeg(uid)),
                    Directions::Right => events.enqueue_event(GameEvents::RequestMoveXPos(uid)),
                },
                _ => {}
            }

            // let e = run_ai(game_state);
            // events.enqueue_event(e);
            self.lastmove = game_state.get::<TimeState>().unscaled_time;
            println!(
                "did ai
                        
                        
                        
                        "
            );
        }
    }
}
impl InstanceLimiter for ECSSystemPeerStart {
    fn is_enabled(&mut self, _: &mut GameState) -> bool {
        true
    }
    fn run_on_instance(&mut self, _: &mut GameState) -> Vec<core::dumpster_engine::NetworkModes> {
        NetworkModes::all_peer()
    }
}
impl ecs_event_reciever::EventReciever<GameEvents> for ECSSystemPeerStart {
    fn dequeue_event(&mut self, game_state: &mut GameState, _: &mut World, event_queue: &mut EventQueue, event: &GameEvents) {
        match event {
            GameEvents::DidTurnBegin(id) => {
                if *id == game_state.instance_id {
                    println!("did begine!!!!");
                    self.do_move = true;
                    self.lastmove = 0.0;
                }
            }
            _ => {}
        }
    }
}
