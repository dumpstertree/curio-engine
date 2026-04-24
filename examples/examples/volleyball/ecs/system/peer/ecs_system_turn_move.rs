use gameplay::{
    context_3d::Context3D,
    traits::{habit::Habit, scope::Scope},
};
use habit::habit;

use curio_core::{
    built_in::record::sys_record_input::SysRecordInput,
    collections::{event_queue::EventQueue, ledger::Ledger},
    collections::network_modes::NetworkModes
};

use crate::{
    exploration::exploration_path::RoomTypes,
    game_board::GameBoard,
    game_events::GameEvents,
    state::{
        host::state_exploration::StateExploration,
        peer::{
            state_peer_input_mode::{InputModes, StatePeerInputMode},
            state_peer_select_targets::StatePeerSelectTargets,
        },
        state_ball_mode::{BallModes, StateBallMode},
        state_position_player::StatePositionEntities,
        state_teams::StateTeamAssignments,
    },
};

#[habit]
pub struct Instance {}
impl Scope for Instance {
    fn is_enabled(&mut self, ledger: &mut Ledger) -> bool {
        ledger
            .get::<StateExploration>()
            .exploration
            .get_cur_room()
            .room_type
            == RoomTypes::Combat
            && ledger.get::<StatePeerInputMode>().mode == InputModes::Move
            && !ledger.get::<StateExploration>().is_selecting_next
            && ledger.get::<StatePeerSelectTargets>().enabled.is_none()
    }
    fn run_on_instance(&mut self, _ledger: &mut Ledger) -> Vec<NetworkModes> {
        NetworkModes::all_peer()
    }
}
impl Habit for Instance {
    fn tick(&mut self, ledger: &mut Ledger, _: &mut Context3D, event_queue: &mut EventQueue) {
        // currently serving and cant move
        let state_ball = ledger.get::<StateBallMode>();
        if state_ball.mode == BallModes::Serve {
            return;
        }

        let state_team = ledger.get::<StateTeamAssignments>();
        let Some(team) = state_team.team_for(&ledger.instance_id) else {
            return;
        };
        let state_position_player = ledger.get::<StatePositionEntities>();
        let pos = state_position_player
            .positions
            .get(&ledger.instance_id)
            .unwrap();
        // get states
        let state_input = ledger.get::<SysRecordInput>();

        // get inputs from mapping
        let move_forward = state_input.mapped[0]
            .get_button_or_default("move_forward")
            .went_up;
        let move_back: bool = state_input.mapped[0]
            .get_button_or_default("move_back")
            .went_up;
        let move_left = state_input.mapped[0]
            .get_button_or_default("move_left")
            .went_up;
        let move_right: bool = state_input.mapped[0]
            .get_button_or_default("move_right")
            .went_up;

        // if any movement detected
        if move_forward || move_back || move_left || move_right {
            if move_forward && GameBoard::can_move(&team, pos, crate::game_board::Directions::Forward) {
                event_queue.enqueue_event(GameEvents::RequestMoveZPos(ledger.instance_id));
            }

            if move_back && GameBoard::can_move(&team, pos, crate::game_board::Directions::Back) {
                event_queue.enqueue_event(GameEvents::RequestMoveZNeg(ledger.instance_id));
            }

            if move_left && GameBoard::can_move(&team, pos, crate::game_board::Directions::Left) {
                event_queue.enqueue_event(GameEvents::RequestMoveXNeg(ledger.instance_id));
            }

            if move_right && GameBoard::can_move(&team, pos, crate::game_board::Directions::Right) {
                event_queue.enqueue_event(GameEvents::RequestMoveXPos(ledger.instance_id));
            }
        }
    }
}
