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
        event_reciever_apply_card_attribute_event_cards_draw, event_reciever_apply_card_attribute_event_cards_energy_edit, event_reciever_apply_card_attribute_event_move_ball_forward, event_reciever_apply_card_attribute_event_set_ball_mode,
        event_reciever_apply_card_attribute_modifier_cost_for_entities, event_reciever_apply_card_attribute_modifier_energy_for_entities, event_reciever_apply_card_attribute_modifier_range_for_entities,
    },
    game_board::{self, GameBoard},
    game_events::{FilledAttribute, FilledCardResponse, GameEvents},
    state::{
        self,
        host::state_card_attribute_modifier_stack::StateCardAttributeModifierStack,
        state_ball_mode::{self, BallModes, StateBallMode},
        state_deck::StateDeck,
        state_energy::{self, StateEnergy},
        state_position_ball::{self, StatePositionBall},
        state_position_player::StatePositionPlayer,
        state_teams::{self, StateTeamAssignments, Teams},
        state_turn::StateTurn,
    },
};

struct AIGameSimulation<T, U> {}
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
                let state_turn = game_state.get::<StateTurn>();
                let state_teams = game_state.get::<StateTeamAssignments>();
                let state_position_ball = game_state.get::<StatePositionBall>();

                let team = state_teams
                    .team_for(&state_turn.active_instance_id)
                    .unwrap();
                let min = team.convert_dir(min.x, min.y);
                let max = team.convert_dir(max.x, max.y);
                let random_x = Random::range_int(min.0, max.0);
                let random_z = Random::range_int(min.1, max.1);

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
            let e = game_state
                .get::<StateEnergy>()
                .all_players
                .get(uid)
                .unwrap()
                .0;
            if e - card.get_cost(game_state, *uid) < 0 {
                continue;
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

impl AIGameSimulation {
    fn make_manuever_card(game_state: &mut GameState, data: &FilledCardResponse, card: &Arc<CardInstance>, event_runner: &mut CardEventRunner) {
        // println!("Red: Play");

        let state_turn = game_state.get::<StateTurn>();
        let uid = state_turn.active_instance_id;

        // add modifiers
        for modifier in card.get_attributes_modifiers(&game_state, uid) {
            event_runner.enqueue_modifier(&modifier);
        }
        // add event
        let e = card.get_attributes_events(&game_state, uid);
        for i in 0..e.len() {
            let attribute = &e[i];
            let filled_attribute_deps = &data.event[i];

            event_runner.enqueue_event(attribute, filled_attribute_deps);
        }
        // post all
        event_runner.post_and_drain(game_state);
    }
    fn make_manuever_move(ai: &mut AIGameSimulation, delta: &Vector2Int) {
        // println!("Red: Move");
        let uid = ai.game_state.get::<StateTurn>().active_instance_id;
        let state_teams = ai.game_state.get::<StateTeamAssignments>();
        let team = state_teams.team_for(&uid).unwrap();
        // let delta = team.convert_dir(delta.x, delta.y);
        let delta = (delta.x, delta.y);
        // edit -> energy
        ai.game_state.edit::<StatePositionPlayer>(|x| {
            // reset energy
            let pos = x.positions[&uid];
            x.positions.insert(uid, (pos.0 + delta.0, pos.1 + delta.1));
        });

        ai.game_state.edit::<StateEnergy>(|x| {
            let cur = x.all_players[&uid];
            x.all_players.insert(uid, (cur.0 - 1, cur.1));
        });
    }
    fn make_manuever_end(ai: &mut AIGameSimulation) {
        // next turn
        ai.turn_num += 1;
        ai.is_terminated = true;
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
        if self.is_terminated {
            return vec![];
        }

        // get state
        let state_turn = self.game_state.get::<StateTurn>();
        let state_teams = self.game_state.get::<StateTeamAssignments>();
        let state_energy = self.game_state.get::<StateEnergy>();
        let state_ball_mode = self.game_state.get::<StateBallMode>();

        let current_player = state_teams
            .team_for(&state_turn.active_instance_id)
            .unwrap();
        // get uis for player
        let uid = match current_player {
            Teams::Red => {
                // get uids
                let Some(uids) = state_teams.team_assignments.get(&Teams::Red) else {
                    panic!("Failed to get uids for team : {}", Teams::Red);
                };
                // return first
                uids[0]
            }
            Teams::Blue => {
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
            if has_energy_for_move {
                let state_pos = self.game_state.get::<StatePositionPlayer>();
                let team = state_teams.team_for(&uid).unwrap();
                let pos = state_pos.positions.get(&uid);
                let pos = pos.unwrap();

                // movement
                let min = GameBoard::get_bounds_min(&team);
                let max = GameBoard::get_bounds_max(&team);

                let offset = (0, 1);
                if pos.1 + offset.1 <= max.y && pos.1 + offset.1 >= min.y && pos.0 + offset.0 <= max.x && pos.0 + offset.0 >= min.x {
                    // println!("move a for team {}", team);
                    output.push(Move::Move(Vector2Int::new(0, 1)));
                }
                let offset = (0, -1);
                if pos.1 + offset.1 <= max.y && pos.1 + offset.1 >= min.y && pos.0 + offset.0 <= max.x && pos.0 + offset.0 >= min.x {
                    // println!("fmove b or team {}", team);

                    output.push(Move::Move(Vector2Int::new(0, -1)));
                }
                let offset = (1, 0);
                if pos.1 + offset.1 <= max.y && pos.1 + offset.1 >= min.y && pos.0 + offset.0 <= max.x && pos.0 + offset.0 >= min.x {
                    // println!("move c for team {}", team);

                    output.push(Move::Move(Vector2Int::new(1, 0)));
                }
                let offset = (-1, 0);
                if pos.1 + offset.1 <= max.y && pos.1 + offset.1 >= min.y && pos.0 + offset.0 <= max.x && pos.0 + offset.0 >= min.x {
                    // println!("move d for team {}", team);

                    output.push(Move::Move(Vector2Int::new(-1, 0)));
                }
                // println!("num manuever {} for team {}", Self::get_available_manuevers(&self.game_state, &uid).len(), team);
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
        match mov {
            Move::Play(card, data) => Self::make_manuever_card(&mut self.game_state, &data, card, &mut self.event_runner),
            Move::Move(delta) => Self::make_manuever_move(self, delta),
            Move::EndTurn => Self::make_manuever_end(self),
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
            self.is_exhuasted = true;
        }
    }
}

// ----------------- Hash -----------------
impl TranspositionHash for AIGameSimulation {
    fn hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();

        // Hash basic fields
        self.turn_num.hash(&mut hasher);
        self.is_terminated.hash(&mut hasher);

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
