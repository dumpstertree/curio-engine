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
use curio_core::{
    built_in::record::sys_record_time::SysRecordTime,
    collections::{event_queue::EventQueue, ledger::Ledger},
    network_modes::NetworkModes,
    system::system_game_state::RecordCommon,
};
use gameplay::{
    context_3d::Context3D,
    traits::{habit::Habit, impulse::Impulse, scope::Scope},
};
use habit::habit;
use impulse::impulse;
use std::vec;

use crate::state::state_teams::StateTeamAssignments;

#[habit]
#[impulse(GameEvents)]
pub struct Instance {
    lastmove: f64,
    move_time: f64,
}
impl Scope for Instance {
    fn is_enabled(&mut self, ledger: &mut Ledger) -> bool {
        let state_team = ledger.read::<StateTeamAssignments>();
        let active_team = ledger.read::<StateTurn>().active_instance_id;
        let Some(current_guids) = state_team.team_assignments.get(&active_team) else {
            return false;
        };

        let any_ai = ledger
            .read::<StateController>()
            .all_players
            .iter()
            .any(|x| current_guids.contains(x.0) && x.1 == &Controller::Ai);

        ledger
            .read::<StateExploration>()
            .exploration
            .get_cur_room()
            .room_type
            == RoomTypes::Combat
            && any_ai
    }
    fn run_on_instance(&mut self, _ledger: &mut Ledger) -> Vec<NetworkModes> {
        NetworkModes::all_host()
    }
}
impl Habit for Instance {
    fn init(&mut self, _ledger: &mut Ledger, _world: &mut Context3D, _: &mut EventQueue) {}
    fn enable(&mut self, ledger: &mut Ledger, _world: &mut Context3D, _: &mut EventQueue) {
        self.move_time = 1.5;
        self.lastmove = ledger.read::<SysRecordTime>().scaled_time;
    }
    fn tick(&mut self, ledger: &mut Ledger, _: &mut Context3D, events: &mut EventQueue) {
        let state_score = ledger.read::<StateScore>();
        if state_score.all_scores.iter().any(|x| *x.1 <= 0) {
            return;
        }
        let state_team = ledger.read::<StateTeamAssignments>();
        let active_team = ledger.read::<StateTurn>().active_instance_id;
        let Some(current_guids) = state_team.team_assignments.get(&active_team) else {
            return;
        };

        let any_ai = ledger
            .read::<StateController>()
            .all_players
            .iter()
            .any(|x| current_guids.contains(x.0) && x.1 == &Controller::Ai);

        let state_team = ledger.read::<StateTeamAssignments>();
        let state_turn = ledger.read::<StateTurn>();
        let state_control = ledger.read::<StateController>();
        let team = state_team
            .team_assignments
            .get(&state_turn.active_instance_id)
            .unwrap();

        let mut do_move = false;
        for x in &state_control.all_players {
            if team.contains(&x.0) {
                match x.1 {
                    Controller::Ai => do_move = true,
                    _ => {}
                }
            }
        }
        // let is_turn = d == ledger.instance_id;
        if do_move && any_ai && ledger.read::<SysRecordTime>().scaled_time - self.lastmove > self.move_time {
            let simulator = AISimulator::new(Box::new(CustomDelegate {}), Box::new(CustomDataSource {}), Box::new(CustomHasher {}), Box::new(CustomEvaluator {}), |ledger| {
                Ledger::new_single_instance(vec![
                    // copy these states
                    (StateCardAttributeModifierStack::id(), Box::new((*ledger.read::<StateCardAttributeModifierStack>()).clone())),
                    (StateTeamAssignments::id(), Box::new((*ledger.read::<StateTeamAssignments>()).clone())),
                    (StatePositionEntities::id(), Box::new((*ledger.read::<StatePositionEntities>()).clone())),
                    (StatePositionBall::id(), Box::new((*ledger.read::<StatePositionBall>()).clone())),
                    (StateBallMode::id(), Box::new((*ledger.read::<StateBallMode>()).clone())),
                    (StateEnergy::id(), Box::new((*ledger.read::<StateEnergy>()).clone())),
                    (StateDeck::id(), Box::new((*ledger.read::<StateDeck>()).clone())),
                    (StateTurn::id(), Box::new((*ledger.read::<StateTurn>()).clone())),
                    // add this state
                    (StateTerminated::id(), Box::new(StateTerminated { is_terminated: false, is_exhuasted: false })),
                ])
            });

            // println!("try simulate");

            // let uid = current_guids[0];
            let move2 = simulator.simulate(ledger, Fidelity::Medium, Threading::Multi);
            let uid = move2.0;
            match move2.1 {
                SimulationManuevers::EndTurn => {
                    events.enqueue_event(GameEvents::RequestTurnEnd(uid));
                }
                SimulationManuevers::PlayCard(card_instance, filled_card_response) => {
                    match card_instance.get_manuever_type() {
                        crate::state::state_deck::CardTypes::Move => self.move_time = 0.5,
                        _ => self.move_time = 1.5,
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

            // let e = run_ai(ledger);
            // events.enqueue_event(e);
            self.lastmove = ledger.read::<SysRecordTime>().scaled_time;
            println!(
                "did ai
                        
                        
                        
                        "
            );
            unsafe { DO_MOVE = false };
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
    fn dequeue_event(&mut self, ledger: &mut Ledger, _: &mut Context3D, _event_queue: &mut EventQueue, event: &GameEvents) {
        match event {
            GameEvents::DidTurnBegin(_id) => {}
            _ => {
                let _state_time = ledger.read::<SysRecordTime>();
                // self.lastmove = ledger.get::<TimeState>().unscaled_time;
                unsafe { DO_MOVE = true };
            }
        }
    }
}

static mut DO_MOVE: bool = false;
