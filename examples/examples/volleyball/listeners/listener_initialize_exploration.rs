use core::{
    collections::{event_queue::EventQueue, game_state::GameState, vector2_int::Vector2Int},
    dumpster_engine::NetworkModes,
    gameplay::ecs::traits::ecs_event_reciever::{self, InstanceLimiter},
    random::Random,
};
use ecs_event::global_ecs_system_event_reciever;
use hecs::World;

use crate::{
    exploration::exploration_path::RoomTypes,
    game_events::GameEvents,
    listeners::listener_initialize_encounter::{Encounter, Participant, TeamController},
    state::{host::state_exploration::StateExploration, state_score::StateScore, state_teams::Teams},
};

#[derive(Default)]
#[global_ecs_system_event_reciever(GameEvents)]
pub struct Listener {}

// Impl - Instance
impl InstanceLimiter for Listener {
    fn is_enabled(&mut self, _game_state: &mut GameState) -> bool {
        true
    }
    fn run_on_instance(&mut self, _game_state: &mut GameState) -> Vec<NetworkModes> {
        NetworkModes::all_host()
    }
}
// Impl - Listener
impl ecs_event_reciever::EventReciever<GameEvents> for Listener {
    fn dequeue_event(&mut self, game_state: &mut GameState, _: &mut World, event_queue: &mut EventQueue, event: &GameEvents) {
        match event {
            GameEvents::InitializeExploration(exploration) => {
                // log
                println!("Exploration Initialized");

                // assign and start the exploration
                game_state.edit::<StateExploration>(|x| {
                    x.exploration = exploration.clone();
                    x.exploration.start();
                });

                // get the state
                let state_exploration = game_state.get::<StateExploration>();

                // get the exploration from the state
                let cur_exploration = state_exploration.exploration;

                // set the healthpoint total
                game_state.edit::<StateScore>(|x| {
                    x.all_scores.insert(Teams::Red, 10);
                });

                // enter the new room
                event_queue.enqueue_event(GameEvents::ExplorationRoomEnter(cur_exploration.get_cur_room()));

                // completion event
                event_queue.enqueue_event(GameEvents::DidInitializeExploration(cur_exploration.clone()));
            }
            _ => {}
        }
    }
}

pub struct EncounterLibrary {}

impl EncounterLibrary {
    pub fn random() -> Encounter {
        match Random::range_int(0, 3) {
            0 => Self::get_encounter_0(),
            1 => Self::get_encounter_1(),
            2 => Self::get_encounter_2(),
            _ => {
                panic!("Bad Roll")
            }
        }
    }
    fn get_encounter_0() -> Encounter {
        Encounter {
            server: Teams::Red,
            team_red: TeamController::Player,
            team_blue: TeamController::Ai(vec![Participant {
                deck_id: "wild".to_string(),
                starting_location: Vector2Int::zero(),
                energy: 3,
                health: 2,
            }]),
        }
    }
    fn get_encounter_1() -> Encounter {
        Encounter {
            server: Teams::Red,
            team_red: TeamController::Player,
            team_blue: TeamController::Ai(vec![Participant {
                deck_id: "wild".to_string(),
                starting_location: Vector2Int::zero(),
                energy: 3,
                health: 2,
            }]),
        }
    }
    fn get_encounter_2() -> Encounter {
        Encounter {
            server: Teams::Red,
            team_red: TeamController::Player,
            team_blue: TeamController::Ai(vec![Participant {
                deck_id: "wild".to_string(),
                starting_location: Vector2Int::zero(),
                energy: 3,
                health: 2,
            }]),
        }
    }
}
