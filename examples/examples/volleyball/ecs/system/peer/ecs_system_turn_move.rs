use built_in_state::state_input::InputState;
use ecs_system::global_ecs_system;
use system_component_default_gameplay::{ecs_system::ECSSystemEventless, world_context::WorldContext};

use core::{
    collections::{event_queue::EventQueue, game_state::GameState},
    dumpster_engine::NetworkModes,
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

#[global_ecs_system]
pub struct ECSSystemTurnMove {}
impl ECSSystemEventless for ECSSystemTurnMove {
    fn run_on_instance(&mut self, game_state: &mut GameState) -> Vec<core::dumpster_engine::NetworkModes> {
        vec![NetworkModes::LocalPeer, NetworkModes::OnlinePeer]
    }
    fn is_enabled(&mut self, game_state: &mut GameState, _: &mut WorldContext) -> bool {
        game_state
            .get::<StateExploration>()
            .exploration
            .get_cur_room()
            .room_type
            == RoomTypes::Combat
            && game_state.get::<StatePeerInputMode>().mode == InputModes::Move
            && !game_state.get::<StateExploration>().is_selecting_next
            && game_state.get::<StatePeerSelectTargets>().enabled.is_none()

        // let is_turn = game_state.get::<StateTurn>().active_instance_id
        //     == game_state
        //         .get::<StateTeamAssignments>()
        //         .team_for(&game_state.instance_id)
        //         .unwrap();

        // is_turn && game_state.get::<StatePeerInputMode>().mode == InputModes::Move
    }
    fn tick(&mut self, game_state: &mut GameState, _: &mut WorldContext, event_queue: &mut EventQueue) {
        // currently serving and cant move
        let state_ball = game_state.get::<StateBallMode>();
        if state_ball.mode == BallModes::Serve {
            return;
        }

        let state_team = game_state.get::<StateTeamAssignments>();
        let Some(team) = state_team.team_for(&game_state.instance_id) else {
            return;
        };
        let state_position_player = game_state.get::<StatePositionEntities>();
        let pos = state_position_player
            .positions
            .get(&game_state.instance_id)
            .unwrap();
        // get states
        let state_input = game_state.get::<InputState>();

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
                event_queue.enqueue_event(GameEvents::RequestMoveZPos(game_state.instance_id));
            }

            if move_back && GameBoard::can_move(&team, pos, crate::game_board::Directions::Back) {
                event_queue.enqueue_event(GameEvents::RequestMoveZNeg(game_state.instance_id));
            }

            if move_left && GameBoard::can_move(&team, pos, crate::game_board::Directions::Left) {
                event_queue.enqueue_event(GameEvents::RequestMoveXNeg(game_state.instance_id));
            }

            if move_right && GameBoard::can_move(&team, pos, crate::game_board::Directions::Right) {
                event_queue.enqueue_event(GameEvents::RequestMoveXPos(game_state.instance_id));
            }
        }
    }
}
