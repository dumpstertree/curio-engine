use core::{
    collections::{
        event_queue::EventQueue,
        game_state::{self, GameState},
        vector2_int::{self, Vector2Int},
    },
    random::Random,
    system::system_game_state::IState,
};
use mcts::{
    self, CycleBehaviour, Evaluator, GameState as MCTSGameState, MCTS, SearchHandle,
    transposition_table::{ApproxTable, TranspositionHash},
    tree_policy::UCTPolicy,
};
use rand::Fill;
use serde::{Deserialize, Serialize};

use std::{
    clone,
    collections::VecDeque,
    fmt::Display,
    hash::{DefaultHasher, Hash, Hasher},
    panic,
    sync::Arc,
    vec,
};

use crate::{
    card_parser::AttributeClearFlag,
    cards::{
        attribute_target_type_entities::AttribtuteTargetTypesEntities, attribute_target_type_tiles::AttributeTargetTypesTiles, card_attribute_events::CardAttributeEvents, card_attribute_modifier::CardAttributeModifiers, card_instance::CardInstance, card_modifier::CardModifier,
        data_dep_empty::DataDepsEmpty, data_dep_filled::DataDepsFilled,
    },
    event_recievers::{
        event_reciever_apply_card_attribute_event_move_ball_forward, event_reciever_apply_card_attribute_event_set_ball_mode, event_reciever_apply_card_attribute_modifier_cost_for_entities, event_reciever_apply_card_attribute_modifier_energy_for_entities,
        event_reciever_apply_card_attribute_modifier_range_for_entities,
    },
    game_board::{self, GameBoard},
    game_events::{FilledAttribute, FilledCardResponse, GameEvents},
    state::{
        self,
        host::state_card_attribute_modifier_stack::StateCardAttributeModifierStack,
        state_ball_mode::{self, BallModes, StateBallMode},
        state_deck::StateDeck,
        state_energy::StateEnergy,
        state_position_ball::{self, StatePositionBall},
        state_position_player::StatePositionPlayer,
        state_teams::{StateTeamAssignments, Teams},
        state_turn::StateTurn,
    },
};

// ----------------- Move Enum -----------------
#[derive(Clone, Debug)]
enum Move {
    Play(Arc<CardInstance>, FilledCardResponse),
    Move(Vector2Int),
    EndTurn,
}
impl Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Move::Play(card_instance, filled_card_response) => f.write_str(&format!("play card {}", card_instance.card_id)),
            Move::Move(vector2_int) => f.write_str("move"),
            Move::EndTurn => f.write_str("end turn"),
        }
    }
}

// ----------------- Player Enum -----------------
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum SimulationTeams {
    Red,
    Blue,
}

// ----------------- Game State -----------------
#[derive(Clone)]
struct AIGameSimulation {
    // game meta
    turn: usize,
    terminal: bool,
    current_player: SimulationTeams,

    //
    game_state: GameState,
    event_runner: CardEventRunner,
}

pub struct FilledAttributeWithPermutation {
    pub filled: Vec<DataDepsFilledAllPermutations>,
}
impl FilledAttributeWithPermutation {
    pub fn new(filled: Vec<DataDepsFilledAllPermutations>) -> FilledAttributeWithPermutation {
        FilledAttributeWithPermutation { filled }
    }
}

pub struct DataDepsFilledAllPermutations {
    permutations: Vec<DataDepsFilled>,
}
impl DataDepsFilledAllPermutations {
    pub fn new() -> DataDepsFilledAllPermutations {
        DataDepsFilledAllPermutations { permutations: vec![] }
    }
    pub fn add_permutation(&mut self, permutation: DataDepsFilled) {
        self.permutations.push(permutation);
    }
}
struct DataDepsFilledForModifiers {
    modifiers_atts: Vec<FilledAttributeWithPermutation>,
    modifiers_events: Vec<FilledAttributeWithPermutation>,
}
impl DataDepsFilledForModifiers {
    pub fn new() -> DataDepsFilledForModifiers {
        DataDepsFilledForModifiers { modifiers_atts: vec![], modifiers_events: vec![] }
    }

    pub fn add_modifier_atts(&mut self, permutation: FilledAttributeWithPermutation) {
        self.modifiers_atts.push(permutation);
    }

    pub fn add_modifier_event(&mut self, permutation: FilledAttributeWithPermutation) {
        self.modifiers_events.push(permutation);
    }
}
impl DataDepsFilledForModifiers {
    pub fn get_data_stack_permutations(&self) -> Vec<FilledCardResponse> {
        let mut output_mods = Vec::new();
        for x in &self.modifiers_atts {
            let mut filled_att = Vec::new();
            for att in &x.filled {
                filled_att.push(att.permutations[0].clone());
            }
            output_mods.push(FilledAttribute::new(filled_att));
        }
        let mut output_events = Vec::new();
        for x in &self.modifiers_events {
            let mut filled_att = Vec::new();
            for att in &x.filled {
                filled_att.push(att.permutations[0].clone());
            }
            output_events.push(FilledAttribute::new(filled_att));
        }

        vec![FilledCardResponse::new(output_mods, output_events)]
    }
    // pub fn get_data_stack_permutations(&self) -> Vec<FilledCardResponse> {
    //     // STEP 1: Build permutations for EACH attribute slot
    //     let mut attribute_slots: Vec<Vec<FilledAttribute>> = Vec::new();

    //     for slot in &self.modifiers_atts {
    //         let mut slot_results: Vec<Vec<DataDepsFilled>> = vec![Vec::new()];

    //         // Each slot has "groups", each group has "permutations"
    //         for group in &slot.filled {
    //             let mut new_results = Vec::new();

    //             for existing in &slot_results {
    //                 for p in &group.permutations {
    //                     let mut combined = existing.clone();
    //                     combined.push(p.clone());
    //                     new_results.push(combined);
    //                 }
    //             }

    //             slot_results = new_results;
    //         }

    //         // Convert each completed combination into a FilledAttribute
    //         let filled_attributes_for_slot = slot_results
    //             .into_iter()
    //             .map(|combo| FilledAttribute::new(combo))
    //             .collect::<Vec<_>>();

    //         attribute_slots.push(filled_attributes_for_slot);
    //     }

    //     // STEP 2: Cartesian product across ALL attribute slots
    //     let mut att_results: Vec<Vec<FilledAttribute>> = vec![Vec::new()];

    //     for slot in attribute_slots {
    //         let mut new_results = Vec::new();

    //         for existing in &att_results {
    //             for item in &slot {
    //                 let mut combined = existing.clone();
    //                 combined.push(item.clone());
    //                 new_results.push(combined);
    //             }
    //         }

    //         att_results = new_results;
    //     }

    //     // STEP 3: Now do the same for events
    //     let mut event_slots: Vec<Vec<FilledAttribute>> = Vec::new();

    //     for slot in &self.modifiers_events {
    //         let mut slot_results: Vec<Vec<DataDepsFilled>> = vec![Vec::new()];

    //         for group in &slot.filled {
    //             let mut new_results = Vec::new();

    //             for existing in &slot_results {
    //                 for p in &group.permutations {
    //                     let mut combined = existing.clone();
    //                     combined.push(p.clone());
    //                     new_results.push(combined);
    //                 }
    //             }

    //             slot_results = new_results;
    //         }

    //         let filled_event_slot = slot_results
    //             .into_iter()
    //             .map(|combo| FilledAttribute::new(combo))
    //             .collect::<Vec<_>>();

    //         event_slots.push(filled_event_slot);
    //     }

    //     let mut event_results: Vec<Vec<FilledAttribute>> = vec![Vec::new()];

    //     for slot in event_slots {
    //         let mut new_results = Vec::new();

    //         for existing in &event_results {
    //             for item in &slot {
    //                 let mut combined = existing.clone();
    //                 combined.push(item.clone());
    //                 new_results.push(combined);
    //             }
    //         }

    //         event_results = new_results;
    //     }

    //     // STEP 4: Final Cartesian product of attributes × events
    //     let mut output = Vec::new();

    //     for att in &att_results {
    //         for evt in &event_results {
    //             output.push(FilledCardResponse::new(att.clone(), evt.clone()));
    //         }
    //     }

    //     output
    // }
}

impl AIGameSimulation {
    fn fill_dependency_tiles(game_state: &GameState, uid: &i32, empty: AttributeTargetTypesTiles) -> DataDepsFilledAllPermutations {
        // create the list of permutations
        let mut permuatations = DataDepsFilledAllPermutations::new();

        // match for empty type
        match empty {
            AttributeTargetTypesTiles::RandomOnTeamUser => {
                // get state
                let state_team = game_state.get::<StateTeamAssignments>();

                // get the team for this user
                let Some(my_team) = state_team.team_for(&uid) else {
                    println!("Failed to find team for uid: {}", uid);
                    return permuatations;
                };

                let min = GameBoard::get_bounds_min(&my_team);
                let max = GameBoard::get_bounds_max(&my_team);

                let random_x = Random::range_int(min.x, max.x);
                let random_z = Random::range_int(min.y, max.y);

                permuatations.add_permutation(DataDepsFilled::Tiles(vec![Vector2Int::new(random_x, random_z)]));
            }
            AttributeTargetTypesTiles::RandomOnTeamOpponent => {
                // get state
                let state_team = game_state.get::<StateTeamAssignments>();

                // get the team for this user
                let Some(my_team) = state_team.team_for(&uid) else {
                    println!("Failed to find team for uid: {}", uid);
                    return permuatations;
                };

                let other_team = my_team.next_team();

                let min = GameBoard::get_bounds_min(&other_team);
                let max = GameBoard::get_bounds_max(&other_team);

                let random_x = Random::range_int(min.x, max.x);
                let random_z = Random::range_int(min.y, max.y);

                permuatations.add_permutation(DataDepsFilled::Tiles(vec![Vector2Int::new(random_x, random_z)]));
            }
            AttributeTargetTypesTiles::RandomInRangeLocal(min, max) => {
                let state_position_ball = game_state.get::<StatePositionBall>();

                let random_x = Random::range_int(min.x, max.x);
                let random_z = Random::range_int(min.y, max.y);

                let col = state_position_ball.column + random_x;
                let row = state_position_ball.row + random_z;

                permuatations.add_permutation(DataDepsFilled::Tiles(vec![Vector2Int::new(col, row)]));
            }
            AttributeTargetTypesTiles::RandomInRangeGlobal(min, max) => {
                let random_x = Random::range_int(min.x, max.x);
                let random_z = Random::range_int(min.y, max.y);

                permuatations.add_permutation(DataDepsFilled::Tiles(vec![Vector2Int::new(random_x, random_z)]));
            }
            _ => {}
        }

        // return the now filled permutations
        permuatations
    }
    fn fill_dependency_entities(game_state: &GameState, uid: &i32, empty: AttribtuteTargetTypesEntities) -> DataDepsFilledAllPermutations {
        // create the list of permutations
        let mut permuatations = DataDepsFilledAllPermutations::new();

        match empty {
            AttribtuteTargetTypesEntities::User => {
                // add a permutation of your uid
                permuatations.add_permutation(DataDepsFilled::Entities(vec![uid.clone()]))
            }
            AttribtuteTargetTypesEntities::Select => {
                // get state
                let state_team = game_state.get::<StateTeamAssignments>();

                // iterate over each team + uids
                for team_ids in state_team.team_assignments {
                    // iterate over each uid on each team
                    for uid in team_ids.1 {
                        // "select" each uid as a different permutation
                        permuatations.add_permutation(DataDepsFilled::Entities(vec![uid]));
                    }
                }
            }
            AttribtuteTargetTypesEntities::RandomAny => {
                // get state
                let state_team = game_state.get::<StateTeamAssignments>();

                // get the uids of a random team
                let Some(random_team) = state_team.team_assignments.get(&Teams::random()) else {
                    println!("Failed to get random team");
                    return permuatations;
                };

                // roll an index between 0 and the max num of users on the team
                let random_user_index = Random::range_int(0, random_team.len() as i32);

                // get the user for the rolled index
                let Some(random_user_id) = random_team.get(random_user_index as usize) else {
                    println!("Failed to get random user");
                    return permuatations;
                };

                // add a permutation for random
                permuatations.add_permutation(DataDepsFilled::Entities(vec![*random_user_id]));
            }
            AttribtuteTargetTypesEntities::RandomOpponent => {
                // get state
                let state_team = game_state.get::<StateTeamAssignments>();

                // get the team for this user
                let Some(my_team) = state_team.team_for(&uid) else {
                    println!("Failed to find team for uid: {}", uid);
                    return permuatations;
                };

                // get the opposing team based on our team
                let opponent_team = my_team.next_team();

                // get all uids on opponent team
                let Some(opponent_uids) = state_team.team_assignments.get(&opponent_team) else {
                    println!("Failed to find uids for team: {}", opponent_team);
                    return permuatations;
                };

                // roll an index between 0 and the max num of users on the team
                let random_user_index = Random::range_int(0, opponent_uids.len() as i32);

                // get the user for the rolled index
                let Some(random_user_id) = opponent_uids.get(random_user_index as usize) else {
                    println!("Failed to get random user for index: {}", random_user_index);
                    return permuatations;
                };

                // add a permutation for random
                permuatations.add_permutation(DataDepsFilled::Entities(vec![*random_user_id]));
            }
        }

        // return the now filled permutations
        permuatations
    }
    fn get_available_manuevers(game_state: &GameState, uid: &i32) -> Vec<Move> {
        // create the return object containing all the moves
        let mut all_manuevers = Vec::new();

        // get state
        let state_deck = game_state.get::<StateDeck>();

        // get deck from state
        let Some(deck) = state_deck.deck.get(uid) else {
            return all_manuevers;
        };

        // iterate over each card in hand
        for card in &deck.hand_consumable {
            // the data that stores all the different permutations
            let mut all_data = DataDepsFilledForModifiers::new();

            // check if we can play the cur card  in hand based on gamestate
            let can_play_card = card.has_statement(game_state, uid.clone());
            if !can_play_card {
                continue;
            }

            // iterate over each modifier in the card and populate the list of dependencies
            for modifier in card.get_attributes_modifiers(game_state, uid.clone()) {
                // iterate over each empty dependency for modifier and fill it
                let mut filled = Vec::new();
                for empty in modifier.get_data_dependencies_empty() {
                    match empty {
                        // dependency is a tile - fill the dependency based on type
                        DataDepsEmpty::Tiles(target_type) => {
                            filled.push(Self::fill_dependency_tiles(game_state, uid, target_type));
                        }

                        // dependency is a entity - fill the dependency based on type
                        DataDepsEmpty::Entities(target_type) => {
                            filled.push(Self::fill_dependency_entities(game_state, uid, target_type));
                        }
                        _ => {}
                    }
                }
                all_data.add_modifier_atts(FilledAttributeWithPermutation::new(filled));
            }
            // iterate over each event in the card and populate the list of dependencies
            for event in card.get_attributes_events(game_state, uid.clone()) {
                // iterate over each empty dependency for modifier and fill it
                let mut filled = Vec::new();
                for empty in event.get_data_dependencies_empty() {
                    match empty {
                        // dependency is a tile - fill the dependency based on type
                        DataDepsEmpty::Tiles(target_type) => {
                            filled.push(Self::fill_dependency_tiles(game_state, uid, target_type));
                        }

                        // dependency is a entity - fill the dependency based on type
                        DataDepsEmpty::Entities(target_type) => {
                            filled.push(Self::fill_dependency_entities(game_state, uid, target_type));
                        }
                        _ => {}
                    }
                }
                all_data.add_modifier_event(FilledAttributeWithPermutation::new(filled));
            }

            // if we didnt end up filling anything in dependencies break early
            // let has_filled_dependencies = !all_data.modifiers_atts.is_empty() && !all_data.modifiers_events.is_empty();
            // if has_filled_dependencies {
            // get all the different permutation combinations
            let combined_permutations = all_data.get_data_stack_permutations();

            // convert those permutations into a play
            for combo in combined_permutations {
                all_manuevers.push(Move::Play(Arc::clone(card), combo));
            }
            // } else {
            //     // No modifier targets — still produce a base move
            //     all_manuevers.push(Move::Play(Arc::clone(card), FilledCardResponse::new(vec![], vec![])));
            // }
        }

        // iterate over each card in hand
        for card in &deck.hand_persistent {
            if card.card_id == "rest" {
                println!(
                    "card id: {}, evnt atts{}, mod atts{}",
                    card.card_id,
                    card.get_attributes_events(game_state, game_state.instance_id)
                        .len(),
                    card.get_attributes_modifiers(game_state, game_state.instance_id)
                        .len()
                );

                panic!("");
            }

            // the data that stores all the different permutations
            let mut all_data = DataDepsFilledForModifiers::new();

            // check if we can play the cur card  in hand based on gamestate
            let can_play_card = card.has_statement(game_state, uid.clone());
            if !can_play_card {
                continue;
            }

            // iterate over each modifier in the card and populate the list of dependencies
            for modifier in card.get_attributes_modifiers(game_state, uid.clone()) {
                // iterate over each empty dependency for modifier and fill it
                let mut filled = Vec::new();
                for empty in modifier.get_data_dependencies_empty() {
                    match empty {
                        // dependency is a tile - fill the dependency based on type
                        DataDepsEmpty::Tiles(target_type) => {
                            filled.push(Self::fill_dependency_tiles(game_state, uid, target_type));
                        }

                        // dependency is a entity - fill the dependency based on type
                        DataDepsEmpty::Entities(target_type) => {
                            filled.push(Self::fill_dependency_entities(game_state, uid, target_type));
                        }
                        _ => {}
                    }
                }
                all_data.add_modifier_atts(FilledAttributeWithPermutation::new(filled));
            }
            // iterate over each event in the card and populate the list of dependencies
            for event in card.get_attributes_events(game_state, uid.clone()) {
                // iterate over each empty dependency for modifier and fill it
                let mut filled = Vec::new();
                for empty in event.get_data_dependencies_empty() {
                    match empty {
                        // dependency is a tile - fill the dependency based on type
                        DataDepsEmpty::Tiles(target_type) => {
                            filled.push(Self::fill_dependency_tiles(game_state, uid, target_type));
                        }

                        // dependency is a entity - fill the dependency based on type
                        DataDepsEmpty::Entities(target_type) => {
                            filled.push(Self::fill_dependency_entities(game_state, uid, target_type));
                        }
                        _ => {}
                    }
                }
                all_data.add_modifier_event(FilledAttributeWithPermutation::new(filled));
            }

            if card.card_id == "serve" {
                println!("num event atts {}", all_data.modifiers_events.len());
                println!("num event deps {}", all_data.modifiers_events[1].filled.len());
            }
            // if we didnt end up filling anything in dependencies break early
            // let has_filled_dependencies = !all_data.modifiers_atts.is_empty() || !all_data.modifiers_events.is_empty();
            // if has_filled_dependencies {
            //     // get all the different permutation combinations
            let combined_permutations = all_data.get_data_stack_permutations();

            if card.card_id == "serve" {
                println!("combined num event atts {}", combined_permutations[0].event.len());
                println!("combined num event deps {}", combined_permutations[0].event[1].filled.len());
            }
            // convert those permutations into a play
            for combo in combined_permutations {
                all_manuevers.push(Move::Play(Arc::clone(card), combo));
            }
            // } else {
            //     // No modifier targets — still produce a base move
            //     all_manuevers.push(Move::Play(Arc::clone(card), FilledCardResponse::new(vec![], vec![])));
            // }
        }

        // return all
        all_manuevers
    }
}
// ----------------- GameState Implementation -----------------
impl MCTSGameState for AIGameSimulation {
    type Move = Move;
    type Player = Teams;
    type MoveList = Vec<Move>;

    fn current_player(&self) -> Teams {
        // get state
        let state_teams = self.game_state.get::<StateTeamAssignments>();
        let state_turn = self.game_state.get::<StateTurn>();

        // get team
        let Some(team) = state_teams.team_for(&state_turn.active_instance_id) else {
            panic!("");
        };

        // return
        team
    }

    fn available_moves(&self) -> Vec<Move> {
        // this has been marked as terminal so we know there is nothing we can do
        if self.terminal {
            return vec![];
        }

        // get state
        let state_teams = self.game_state.get::<StateTeamAssignments>();
        let state_energy = self.game_state.get::<StateEnergy>();
        let state_ball_mode = self.game_state.get::<StateBallMode>();

        // get uis for player
        let uid = match self.current_player {
            SimulationTeams::Red => {
                // get uids
                let Some(uids) = state_teams.team_assignments.get(&Teams::Red) else {
                    panic!("Failed to get uids for team : {}", Teams::Red);
                };
                // return first
                uids[0]
            }
            SimulationTeams::Blue => {
                // get uids
                let Some(uids) = state_teams.team_assignments.get(&Teams::Blue) else {
                    panic!("Failed to get uids for team : {}", Teams::Blue);
                };
                // return first
                uids[0]
            }
        };

        // create the output
        let mut output = Vec::new();

        // get the amount of energy this uid has left
        let Some(energy_for_uid) = state_energy.all_players.get(&uid) else {
            panic!("Failed to find energy for uid: {}", uid);
        };

        if self.game_state.get::<StateBallMode>().mode != BallModes::Serve {
            // if we have enough energy to move add all the directions
            let has_energy_for_move = energy_for_uid.0 > 0;
            let has_mode_for_move = state_ball_mode.mode != BallModes::Serve;
            if has_energy_for_move && has_mode_for_move {
                let state_pos = self.game_state.get::<StatePositionPlayer>();
                let team = state_teams.team_for(&uid).unwrap();
                let pos = state_pos.positions.get(&uid);
                let pos = pos.unwrap();

                // movement
                let min = GameBoard::get_bounds_min(&team);
                let max = GameBoard::get_bounds_max(&team);

                let offset = team.convert_dir(0, 1);
                if pos.1 + offset.1 < max.y && pos.1 + offset.1 >= min.y && pos.0 + offset.0 < max.x && pos.0 + offset.0 >= min.x {
                    output.push(Move::Move(Vector2Int::new(0, 1)));
                }
                let offset = team.convert_dir(0, -1);
                if pos.1 + offset.1 < max.y && pos.1 + offset.1 >= min.y && pos.0 + offset.0 < max.x && pos.0 + offset.0 >= min.x {
                    output.push(Move::Move(Vector2Int::new(0, -1)));
                }
                let offset = team.convert_dir(1, 0);
                if pos.1 + offset.1 < max.y && pos.1 + offset.1 >= min.y && pos.0 + offset.0 < max.x && pos.0 + offset.0 >= min.x {
                    output.push(Move::Move(Vector2Int::new(1, 0)));
                }
                let offset = team.convert_dir(-1, 0);
                if pos.1 + offset.1 < max.y && pos.1 + offset.1 >= min.y && pos.0 + offset.0 < max.x && pos.0 + offset.0 >= min.x {
                    output.push(Move::Move(Vector2Int::new(-1, 0)));
                }
            }

            // end
            output.push(Move::EndTurn);
        }

        // append get all manuevers available for this uid
        output.extend(Self::get_available_manuevers(&self.game_state, &uid));

        // return
        output
    }

    fn make_move(&mut self, mov: &Self::Move) {
        // println!("{}", mov);
        match (self.current_player, mov) {
            (SimulationTeams::Red, Move::Play(card, data)) => {
                // println!("Red: Play");

                let state_teams = self.game_state.get::<StateTeamAssignments>();
                let Some(uids) = state_teams.team_assignments.get(&Teams::Red) else {
                    return;
                };

                // add modifiers
                for modifier in card.get_attributes_modifiers(&self.game_state, uids[0]) {
                    self.event_runner.enqueue_modifier(&modifier);
                }
                // add event
                let e = card.get_attributes_events(&self.game_state, uids[0]);
                for i in 0..e.len() {
                    let attribute = &e[i];
                    let filled_attribute_deps = &data.event[i];

                    self.event_runner
                        .enqueue_event(attribute, filled_attribute_deps);
                }
                // post all
                self.event_runner.post_and_drain(&mut self.game_state);
            }
            (SimulationTeams::Blue, Move::Play(card, data)) => {
                // println!("Blue: Play");

                let state_teams = self.game_state.get::<StateTeamAssignments>();
                let Some(uids) = state_teams.team_assignments.get(&Teams::Blue) else {
                    return;
                };

                // add modifiers
                for modifier in card.get_attributes_modifiers(&self.game_state, uids[0]) {
                    self.event_runner.enqueue_modifier(&modifier);
                }
                // add event
                let e = card.get_attributes_events(&self.game_state, uids[0]);
                for i in 0..e.len() {
                    let attribute = &e[i];
                    let filled_attribute_deps = &data.event[i];

                    self.event_runner
                        .enqueue_event(attribute, filled_attribute_deps);
                }
                // post all
                self.event_runner.post_and_drain(&mut self.game_state);
            }
            (SimulationTeams::Red, Move::EndTurn) => {
                //
                let state_teams = self.game_state.get::<StateTeamAssignments>();
                let Some(red_uids) = state_teams.team_assignments.get(&Teams::Red) else {
                    return;
                };
                let Some(blue_uids) = state_teams.team_assignments.get(&Teams::Blue) else {
                    return;
                };

                // // edit -> energy
                // self.game_state.edit::<StateEnergy>(|x| {
                //     // reset energy
                //     let energy = x.all_players[&red_uids[0]];
                //     x.all_players.insert(red_uids[0], (energy.1, energy.1));
                // });

                // next turn
                self.turn += 1;
                self.terminal = true;

                // update next player
                self.current_player = SimulationTeams::Blue;
                self.game_state
                    .edit::<StateTurn>(|x| x.active_instance_id = blue_uids[0]);
            }
            (SimulationTeams::Blue, Move::EndTurn) => {
                //
                let state_teams = self.game_state.get::<StateTeamAssignments>();
                let Some(red_uids) = state_teams.team_assignments.get(&Teams::Red) else {
                    return;
                };
                let Some(blue_uids) = state_teams.team_assignments.get(&Teams::Blue) else {
                    return;
                };

                // edit -> energy
                self.game_state.edit::<StateEnergy>(|x| {
                    // reset energy
                    let energy = x.all_players[&blue_uids[0]];
                    x.all_players.insert(blue_uids[0], (energy.1, energy.1));
                });

                // next turn
                self.turn += 1;
                self.terminal = true;

                // update next player
                self.current_player = SimulationTeams::Blue;
                self.game_state
                    .edit::<StateTurn>(|x| x.active_instance_id = red_uids[0]);
            }
            (SimulationTeams::Red, Move::Move(delta)) => {
                // println!("Red: Move");

                let state_teams = self.game_state.get::<StateTeamAssignments>();
                let Some(uids) = state_teams.team_assignments.get(&Teams::Red) else {
                    return;
                };

                // edit -> energy
                self.game_state.edit::<StatePositionPlayer>(|x| {
                    // reset energy
                    let pos = x.positions[&uids[0]];
                    x.positions
                        .insert(uids[0], (pos.0 + delta.x, pos.1 + delta.y));
                });

                self.game_state.edit::<StateEnergy>(|x| {
                    let uid = uids[0];
                    let cur = x.all_players[&uid];
                    x.all_players.insert(uid, (cur.0 - 1, cur.1));
                });
            }
            (SimulationTeams::Blue, Move::Move(delta)) => {
                // println!("Blue: Move");

                let state_teams = self.game_state.get::<StateTeamAssignments>();
                let Some(uids) = state_teams.team_assignments.get(&Teams::Blue) else {
                    return;
                };

                // edit -> energy
                self.game_state.edit::<StatePositionPlayer>(|x| {
                    // reset energy
                    let pos = x.positions[&uids[0]];
                    x.positions
                        .insert(uids[0], (pos.0 + delta.x, pos.1 + delta.y));
                });

                self.game_state.edit::<StateEnergy>(|x| {
                    let uid = uids[0];
                    let cur = x.all_players[&uid];
                    x.all_players.insert(uid, (cur.0 - 1, cur.1));
                });
            }
        }

        let uid = self.game_state.get::<StateTurn>().active_instance_id;
        let energy = self
            .game_state
            .get::<StateEnergy>()
            .all_players
            .get(&uid)
            .unwrap()
            .0;
        // println!("turn num {}, with energy {} ", self.turn, energy);
        if energy <= 0 {
            self.terminal = true;
        }

        // // end condition
        // match self.current_player {
        //     SimulationTeams::Red => {
        //         if self.game_state.get_value2::<StatePositionBall>().row < 2 {
        //             self.terminal = true;
        //         }
        //     }
        //     SimulationTeams::Blue => {
        //         if self.game_state.get_value2::<StatePositionBall>().row >= 2 {
        //             self.terminal = true;
        //         }
        //     }
        // }
    }
}

// ----------------- Hash -----------------
impl TranspositionHash for AIGameSimulation {
    fn hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();

        // Hash basic fields
        self.turn.hash(&mut hasher);
        self.terminal.hash(&mut hasher);
        self.current_player.hash(&mut hasher);

        // Hash nested data
        self.game_state.get::<StateTurn>().hash(&mut hasher);

        self.game_state
            .get::<StateTeamAssignments>()
            .hash(&mut hasher);
        self.game_state
            .get::<StatePositionPlayer>()
            .hash(&mut hasher);
        self.game_state.get::<StatePositionBall>().hash(&mut hasher);
        self.game_state.get::<StateEnergy>().hash(&mut hasher);
        self.game_state.get::<StateDeck>().hash(&mut hasher);
        self.game_state
            .get::<StateCardAttributeModifierStack>()
            .hash(&mut hasher);

        hasher.finish()
    }
}

// ----------------- Evaluator -----------------
struct AiGameEvaluator;

impl Evaluator<MyMCTS> for AiGameEvaluator {
    type StateEvaluation = i64;

    fn evaluate_new_state(&self, state: &AIGameSimulation, moves: &Vec<Move>, _handle: Option<SearchHandle<MyMCTS>>) -> (Vec<()>, i64) {
        // Simple scoring:
        // - prefer pushing the ball toward enemy side (higher Y for red)
        // - prefer more energy left
        let score: i64 = match state.current_player {
            SimulationTeams::Red => {
                if state.game_state.get::<StatePositionBall>().row < 2 {
                    -99
                } else {
                    let state_teams = state.game_state.get::<StateTeamAssignments>();
                    let state_energy = state.game_state.get::<StateEnergy>();

                    let Some(uids) = state_teams.team_assignments.get(&Teams::Red) else {
                        //
                        return (vec![(); moves.len()], 0);
                    };

                    let Some(energy) = state_energy.all_players.get(&uids[0]) else {
                        //
                        return (vec![(); moves.len()], 0);
                    };

                    // (max_energy * 2 ) + (cur_energy) + opponent_distance_from_ball
                    (energy.1 * 3) as i64
                }
            }
            SimulationTeams::Blue => {
                if state.game_state.get::<StatePositionBall>().row >= 2 {
                    -99
                } else {
                    let state_teams = state.game_state.get::<StateTeamAssignments>();
                    let state_energy = state.game_state.get::<StateEnergy>();

                    let Some(uids) = state_teams.team_assignments.get(&Teams::Blue) else {
                        //
                        return (vec![(); moves.len()], 0);
                    };

                    let Some(energy) = state_energy.all_players.get(&uids[0]) else {
                        //
                        return (vec![(); moves.len()], 0);
                    };

                    // (max_energy * 2 ) + (cur_energy) + opponent_distance_from_ball
                    (energy.1 * 3) as i64
                }
            }
        };

        println!("score check: {}", score);
        (vec![(); moves.len()], score)
    }

    fn interpret_evaluation_for_player(&self, evaln: &i64, _player: &Teams) -> i64 {
        *evaln
    }

    fn evaluate_existing_state(&self, _state: &AIGameSimulation, evaln: &i64, _handle: SearchHandle<MyMCTS>) -> i64 {
        *evaln
    }
}

// ----------------- MCTS Type -----------------
#[derive(Default)]
struct MyMCTS;

impl MCTS for MyMCTS {
    type State = AIGameSimulation;
    type Eval = AiGameEvaluator;
    type NodeData = ();
    type ExtraThreadData = ();
    type TreePolicy = UCTPolicy;
    type TranspositionTable = ApproxTable<Self>;

    fn cycle_behaviour(&self) -> CycleBehaviour<Self> {
        CycleBehaviour::UseCurrentEvalWhenCycleDetected
    }
}

// ----------------- MAIN -----------------
pub fn run_ai(game_state: &mut GameState) -> GameEvents {
    // for _ in 0..5 {
    let local_game_state = GameState::new_single_instance(vec![
        (StateCardAttributeModifierStack::id(), Box::new(game_state.get::<StateCardAttributeModifierStack>())),
        (StateTeamAssignments::id(), Box::new(game_state.get::<StateTeamAssignments>())),
        (StatePositionPlayer::id(), Box::new(game_state.get::<StatePositionPlayer>())), //
        (StatePositionBall::id(), Box::new(game_state.get::<StatePositionBall>())),
        (StateBallMode::id(), Box::new(game_state.get::<StateBallMode>())),
        (StateEnergy::id(), Box::new(game_state.get::<StateEnergy>())),
        (StateDeck::id(), Box::new(game_state.get::<StateDeck>())),
        (StateTurn::id(), Box::new(game_state.get::<StateTurn>())),
    ]);
    // create simulation state
    let sim = AIGameSimulation {
        turn: 0,
        terminal: false,
        current_player: SimulationTeams::Red, // AI goes first
        game_state: local_game_state,
        event_runner: CardEventRunner::new(),
    };

    // Build MCTS manager — following docs.rs example style
    let policy = UCTPolicy::new(0.5);
    let table = ApproxTable::new(1024); // tune size
    let mut manager = mcts::MCTSManager::new(sim, MyMCTS, AiGameEvaluator, policy, table);

    // Run playouts — choose iterations & threads appropriate for your runtime.
    // manager.playout_n_parallel(2000, 4);

    manager.playout_n(100);

    // Retrieve best move from manager
    if let Some(best_move) = manager.best_move() {
        println!("MCTS Best Move: {:?}", best_move);

        let uid = game_state.get::<StateTurn>().active_instance_id;
        match best_move {
            Move::Play(card_instance, data_deps_filleds) => {
                return GameEvents::RequestUseManeuverPersistent(uid, card_instance.instance_id.clone(), data_deps_filleds);
            }
            Move::Move(vector2_int) => {
                let t = game_state
                    .get::<StateTeamAssignments>()
                    .team_for(&uid)
                    .unwrap();

                let converted = t.convert_dir(vector2_int.x, vector2_int.y);
                let vector2_int = Vector2Int::new(converted.0, converted.1);
                if vector2_int == Vector2Int::new(-1, 0) {
                    return GameEvents::RequestMoveXNeg(uid);
                } else if vector2_int == Vector2Int::new(1, 0) {
                    return GameEvents::RequestMoveXPos(uid);
                } else if vector2_int == Vector2Int::new(0, -1) {
                    return GameEvents::RequestMoveZNeg(uid);
                } else if vector2_int == Vector2Int::new(0, 1) {
                    return GameEvents::RequestMoveZPos(uid);
                } else {
                    panic!("");
                }
            }
            Move::EndTurn => {
                return GameEvents::RequestTurnEnd(uid);
            }
        }
    } else {
        panic!("No valid move found by MCTS");
    }

    // make play
    // }
}
#[derive(Clone, Hash, PartialEq, Eq)]
pub struct EventRunner<T, U>
where
    T: Clone + 'static,
    U: Clone + 'static,
{
    recievers: Vec<fn(&T, &mut U) -> Vec<T>>,
    queue: Vec<T>,
}

impl<T, U> EventRunner<T, U>
where
    T: Clone + 'static,
    U: Clone + 'static,
{
    pub fn new(recievers: Vec<fn(&T, &mut U) -> Vec<T>>) -> Self {
        Self { recievers, queue: Vec::new() }
    }

    pub fn enqueue(&mut self, event: &T) {
        self.queue.push(event.clone());
    }

    pub fn post_and_drain(&mut self, data: &mut U) {
        while let Some(event) = self.queue.pop() {
            for func in &self.recievers {
                let new_events = func(&event, data);
                // Optionally enqueue new events generated by handlers
                for new_event in new_events {
                    self.queue.push(new_event);
                }
            }
        }
    }
}
#[derive(Clone)]
pub enum CardEvents {
    // modifier
    ApplyModifierEnergyForEntities(AttributeClearFlag, Vec<i32>, i32),
    ApplyModifierCostForEntities(AttributeClearFlag, Vec<i32>, i32),
    ApplyModifierRangeForEntities(AttributeClearFlag, Vec<i32>, i32),
    // events
    ApplyEventRefillEnergy,
    ApplyEventGainEnergy,
    ApplyEventMoveEntity,
    ApplyEventDrawCards,
    ApplyEventDiscardCards,
    ApplyEventSetBallMode(BallModes),
    /// i32:EntityID, i32:CardID, Vec<i32>:TargetTileIDs
    ApplyEventMoveBall(i32, i32, DataDepsFilled),
}
#[derive(Clone)]
pub struct CardEventRunner {
    runner: EventRunner<CardEvents, GameState>,
}

impl CardEventRunner {
    fn new() -> CardEventRunner {
        // create the list of all the recievers
        let recievers: Vec<fn(&CardEvents, &mut GameState) -> Vec<CardEvents>> = vec![
            // modifiers
            event_reciever_apply_card_attribute_modifier_energy_for_entities::EventReciever::recieve,
            event_reciever_apply_card_attribute_modifier_cost_for_entities::EventReciever::recieve,
            event_reciever_apply_card_attribute_modifier_range_for_entities::EventReciever::recieve,
            // events
            event_reciever_apply_card_attribute_event_move_ball_forward::EventReciever::recieve,
            event_reciever_apply_card_attribute_event_set_ball_mode::EventReciever::recieve,
        ];

        // create the instance
        CardEventRunner { runner: EventRunner::new(recievers) }
    }
    fn enqueue_modifier(&mut self, event: &CardAttributeModifiers) {
        match event {
            CardAttributeModifiers::EditEnergyForEntities(attribute_clear_flag, _, count) => self
                .runner
                .enqueue(&CardEvents::ApplyModifierEnergyForEntities(
                    attribute_clear_flag.clone(), //
                    vec![],
                    *count,
                )),
            CardAttributeModifiers::EditRangeForEntities(attribute_clear_flag, _, count) => self
                .runner
                .enqueue(&CardEvents::ApplyModifierRangeForEntities(
                    attribute_clear_flag.clone(), //
                    vec![],
                    *count,
                )),
            CardAttributeModifiers::EditCostForEntities(attribute_clear_flag, _, count) => self
                .runner
                .enqueue(&CardEvents::ApplyModifierCostForEntities(
                    attribute_clear_flag.clone(), //
                    vec![],
                    *count,
                )),
        }
    }
    fn enqueue_event(&mut self, event: &CardAttributeEvents, data: &FilledAttribute) {
        match event {
            CardAttributeEvents::SetBallMode(mode) => {
                self.runner
                    .enqueue(&CardEvents::ApplyEventSetBallMode(mode.clone()));
            }
            CardAttributeEvents::MoveBall(_) => {
                self.runner
                    .enqueue(&CardEvents::ApplyEventMoveBall(0, 0, data.filled[0].clone()));
            }
            _ => {}
        }
    }
    fn post_and_drain(&mut self, game_state: &mut GameState) {
        self.runner.post_and_drain(game_state);
    }
}
