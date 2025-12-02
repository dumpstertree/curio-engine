use built_in_state::{state_camera::CameraState, state_network::StateNetwork};
use ecs_system::global_ecs_system;
use hecs::World;

use core::{
    collections::{event_queue::EventQueue, game_state::GameState, vector2_int::Vector2Int},
    dumpster_engine::NetworkModes,
    gameplay::ecs::traits::ecs_system::ECSSystemEventless,
};
use std::vec;

use crate::{
    game_events::GameEvents,
    listeners::listener_start_encounter::{Encounter, Participant, TeamAssignment, TeamController},
    state::{
        state_deck::{Deck, StateDeck},
        state_energy::StateEnergy,
        state_position_player::StatePositionEntities,
        state_teams::{StateTeamAssignments, Teams},
    },
};

#[global_ecs_system]
pub struct ECSSystemGameStart {}
impl ECSSystemEventless for ECSSystemGameStart {
    fn is_enabled(&mut self, _: &mut GameState, _: &mut World) -> bool {
        true
    }
    fn run_on_instance(&mut self, game_state: &mut GameState) -> Vec<core::dumpster_engine::NetworkModes> {
        vec![NetworkModes::LocalHost, NetworkModes::OnlineHost]
    }
    fn enable(&mut self, game_state: &mut GameState, _: &mut World, event_queue: &mut EventQueue) {
        println!("Instance: {}. Host Startup", game_state.instance_id);

        // set resolution
        game_state.edit::<CameraState>(|x| {
            x.resolution_width = 1920 / 1;
            x.resolution_height = 1080 / 1;
        });

        event_queue.enqueue_event(GameEvents::InitializeEncounter(Encounter {
            server: Teams::Red,
            team_red: TeamController::Player,
            team_blue: TeamController::Ai(vec![Participant {
                deck_id: "".to_string(),
                starting_location: Vector2Int::zero(),
                energy: 5,
            }]),
        }));
    }
}
