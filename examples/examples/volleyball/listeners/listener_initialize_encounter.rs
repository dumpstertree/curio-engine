use core::{
    collections::{event_queue::EventQueue, game_state::GameState, vector2_int::Vector2Int},
    dumpster_engine::NetworkModes,
    gameplay::{

    },
    io::asset_database::AssetDatabaseListing,
    random::Random,
};

use built_in_state::state_network::StateNetwork;
use ecs_event::global_ecs_system_event_reciever;
use hecs::World;
use serde::{Deserialize, Serialize};
use system_component_default_gameplay::{ecs_event_reciever::{EventReciever, InstanceLimiter}, world_context::WorldContext};

use crate::{
    AssetMappingUIDs,
    cards::deck_library::DeckLibrary,
    game_events::GameEvents,
    state::{
        host::{state_deck_exploration::StateDeckExploration, state_enounter_mode::StateEncounter, state_entity_visual::StateVisualEntity, state_health_exploration::StateHealthExploration, state_heat::StateHeat},
        state_controller::StateController,
        state_deck::{Deck, StateDeck},
        state_energy::StateEnergy,
        state_position_player::StatePositionEntities,
        state_score::StateScore,
        state_teams::{StateTeamAssignments, Teams},
    },
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
impl EventReciever<GameEvents> for Listener {
    fn dequeue_event(&mut self, game_state: &mut GameState, _: &mut WorldContext, event_queue: &mut EventQueue, event: &GameEvents) {
        match event {
            GameEvents::InitializeEncounter(encounter) => {
                // log
                println!("Encounter Initialized");

                // store the encounter
                game_state.edit::<StateEncounter>(|x| {
                    x.encounter = encounter.clone();
                });

                // clear old
                game_state.edit::<StateTeamAssignments>(|x| {
                    x.team_assignments.insert(Teams::Red, Vec::new());
                    x.team_assignments.insert(Teams::Blue, Vec::new());
                });
                game_state.edit::<StateEnergy>(|x| {
                    x.all_players.clear();
                });
                game_state.edit::<StateDeck>(|x| {
                    x.deck.clear();
                });
                game_state.edit::<StateController>(|x| {
                    x.all_players.clear();
                });
                game_state.edit::<StatePositionEntities>(|x| {
                    x.positions.clear();
                });
                game_state.edit::<StateHeat>(|x| {
                    x.all_players.clear();
                });

                // insert new
                match &encounter.team_blue {
                    TeamController::Ai(participants) => {
                        for p in participants {
                            let guid = Random::range_int(-9999, 9999);

                            // intialize the team
                            game_state.edit::<StateTeamAssignments>(|x| {
                                x.team_assignments.get_mut(&Teams::Blue).unwrap().push(guid);
                            });
                            // initialize the deck
                            game_state.edit::<StateDeck>(|x| {
                                x.deck
                                    .insert(guid, DeckLibrary::get_deck_for_uid(&p.deck_id));
                            });
                            // initialize the energy max
                            game_state.edit::<StateEnergy>(|x| {
                                x.all_players.insert(guid, (0, p.energy));
                            });
                            // initialize the energy max
                            game_state.edit::<StatePositionEntities>(|x| {
                                x.positions.insert(guid, (0, 0));
                            });
                            // initialze the controll state
                            game_state.edit::<StateController>(|x| {
                                x.all_players.insert(guid, Controller::Ai);
                            });
                            // initialze the visual
                            game_state.edit::<StateVisualEntity>(|x| {
                                x.all.insert(guid, p.visual.clone());
                            });
                            // initialize heat
                            game_state.edit::<StateHeat>(|x| {
                                x.all_players.insert(guid, 0);
                            });
                        }
                    }
                    TeamController::Player => {
                        let state_network = game_state.get::<StateNetwork>();
                        let guids = state_network.peer_instance_ids();
                        for guid in guids {
                            // intialize the team
                            game_state.edit::<StateTeamAssignments>(|x| {
                                x.team_assignments
                                    .get_mut(&Teams::Blue)
                                    .unwrap()
                                    .push(*guid);
                            });
                            // initialize the deck
                            let state_deck_exploration = game_state.get::<StateDeckExploration>();
                            game_state.edit::<StateDeck>(|x| {
                                x.deck
                                    .insert(*guid, state_deck_exploration.deck.get(guid).unwrap().clone());
                            });
                            // initialize the energy max
                            game_state.edit::<StateEnergy>(|x| {
                                x.all_players.insert(*guid, (0, 5));
                            });
                            // initialize the energy max
                            game_state.edit::<StatePositionEntities>(|x| {
                                x.positions.insert(*guid, (0, 0));
                            });
                            // initialze the controll state
                            game_state.edit::<StateController>(|x| {
                                x.all_players.insert(*guid, Controller::Player);
                            });
                            // initialize heat
                            game_state.edit::<StateHeat>(|x| {
                                x.all_players.insert(*guid, 0);
                            });
                        }
                    }
                    TeamController::Invald => todo!(),
                }

                match &encounter.team_red {
                    TeamController::Ai(participants) => {
                        for p in participants {
                            // update the healthpoint total
                            let guid = Random::range_int(-9999, 9999);

                            // intialize the team
                            game_state.edit::<StateTeamAssignments>(|x| {
                                x.team_assignments.get_mut(&Teams::Red).unwrap().push(guid);
                            });
                            // initialize the deck
                            game_state.edit::<StateDeck>(|x| {
                                x.deck
                                    .insert(guid, DeckLibrary::get_deck_for_uid(&p.deck_id));
                            });
                            // initialize the energy max
                            game_state.edit::<StateEnergy>(|x| {
                                x.all_players.insert(guid, (0, p.energy));
                            });
                            // initialize the energy max
                            game_state.edit::<StatePositionEntities>(|x| {
                                x.positions.insert(guid, (0, 0));
                            });
                            // initialze the controll state
                            game_state.edit::<StateController>(|x| {
                                x.all_players.insert(guid, Controller::Ai);
                            });
                            // initialze the visual
                            game_state.edit::<StateVisualEntity>(|x| {
                                x.all.insert(guid, p.visual.clone());
                            });
                            // initialize heat
                            game_state.edit::<StateHeat>(|x| {
                                x.all_players.insert(guid, 0);
                            });
                        }
                    }
                    TeamController::Player => {
                        let state_network = game_state.get::<StateNetwork>();
                        let guids = state_network.peer_instance_ids();
                        for guid in guids {
                            // intialize the team
                            game_state.edit::<StateTeamAssignments>(|x| {
                                x.team_assignments.get_mut(&Teams::Red).unwrap().push(*guid);
                            });
                            // initialize the deck
                            let state_deck_exploration = game_state.get::<StateDeckExploration>();
                            game_state.edit::<StateDeck>(|x| {
                                x.deck
                                    .insert(*guid, state_deck_exploration.deck.get(guid).unwrap().clone());
                            });
                            // initialize the energy max
                            game_state.edit::<StateEnergy>(|x| {
                                x.all_players.insert(*guid, (0, 5));
                            });
                            // initialize the energy max
                            game_state.edit::<StatePositionEntities>(|x| {
                                x.positions.insert(*guid, (0, 0));
                            });
                            // initialze the controll state
                            game_state.edit::<StateController>(|x| {
                                x.all_players.insert(*guid, Controller::Player);
                            });
                            // initialize heat
                            game_state.edit::<StateHeat>(|x| {
                                x.all_players.insert(*guid, 0);
                            });
                        }
                    }
                    TeamController::Invald => todo!(),
                }

                // set scores
                let state_network = game_state.get::<StateNetwork>();
                let state_health_exploration = game_state.get::<StateHealthExploration>();
                game_state.edit::<StateScore>(|x| {
                    let mut score_red = 0;
                    let mut score_blue = 0;
                    match &encounter.team_red {
                        TeamController::Ai(participants) => {
                            for p in participants {
                                score_red += p.health;
                            }
                        }
                        TeamController::Player => {
                            for p in state_network.peer_instance_ids() {
                                //
                                let Some(health) = state_health_exploration.all.get(p) else {
                                    println!("ExplorationHealth not found");
                                    continue;
                                };

                                score_red += health.0;
                            }
                        }
                        _ => {}
                    }
                    match &encounter.team_blue {
                        TeamController::Ai(participants) => {
                            for p in participants {
                                score_blue += p.health;
                            }
                        }
                        TeamController::Player => {
                            for p in state_network.peer_instance_ids() {
                                //
                                let Some(health) = state_health_exploration.all.get(p) else {
                                    println!("ExplorationHealth not found");
                                    continue;
                                };

                                score_blue += health.0;
                            }
                        }
                        _ => {}
                    }
                    x.all_scores.insert(Teams::Red, score_red);
                    x.all_scores.insert(Teams::Blue, score_blue);
                });

                // reset the board now that the encounter has been updated
                event_queue.enqueue_event(GameEvents::ResetBoard(encounter.server.clone()));

                // send notification
                event_queue.enqueue_event(GameEvents::DidInitializeEncounter(encounter.clone()));
            }
            _ => {}
        }
    }
}

#[derive(PartialEq, Eq, Hash, Default, Clone, Deserialize, Serialize)]
pub enum TeamController {
    #[default]
    Invald,
    Ai(Vec<Participant>),
    Player,
}
#[derive(PartialEq, Eq, Hash, Default, Clone, Deserialize, Serialize)]
pub struct Encounter {
    pub server: Teams,
    pub team_red: TeamController,
    pub team_blue: TeamController,
}
#[derive(PartialEq, Eq, Hash, Default, Clone, Deserialize, Serialize)]
pub struct Participant {
    pub deck_id: String,
    pub visual: AssetMappingUIDs,
    pub energy: i32,
    pub health: i32,
}
#[derive(PartialEq, Eq, Hash, Default, Clone, Deserialize, Serialize)]
pub struct TeamAssignment {
    pub controller: TeamController,
    pub participants: Vec<Participant>,
}

#[derive(PartialEq, Eq, Hash, Default, Clone, Deserialize, Serialize)]
pub enum Controller {
    #[default]
    Invald,
    Ai,
    Player,
}
