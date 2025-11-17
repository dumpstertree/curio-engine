use core::{
    collections::{game_state::GameState, vector2_int::Vector2Int},
    random::Random,
    system::system_game_state::IState,
};
use mcts::{
    self, CycleBehaviour, Evaluator, GameState as MCTSGameState, MCTS, SearchHandle,
    transposition_table::{ApproxTable, TranspositionHash},
    tree_policy::UCTPolicy,
};

use std::{
    collections::VecDeque,
    hash::{DefaultHasher, Hash, Hasher},
    sync::Arc,
    vec,
};

use crate::{
    card_parser::AttributeClearFlag,
    cards::{attribute_target_type_entities::AttribtuteTargetTypesEntities, card_attribute_events::CardAttributeEvents, card_attribute_modifier::CardAttributeModifiers, card_instance::CardInstance, card_modifier::CardModifier, data_dep_empty::DataDepsEmpty, data_dep_filled::DataDepsFilled},
    event_recievers::{event_reciever_apply_card_attribute_event_move_ball_forward, event_reciever_apply_card_attribute_modifier_cost_for_entities, event_reciever_apply_card_attribute_modifier_energy_for_entities, event_reciever_apply_card_attribute_modifier_range_for_entities},
    game_board::GameBoard,
    state::{
        host::state_card_attribute_modifier_stack::StateCardAttributeModifierStack,
        state_ball_mode::StateBallMode,
        state_deck::StateDeck,
        state_energy::StateEnergy,
        state_position_ball::StatePositionBall,
        state_position_player::StatePositionPlayer,
        state_teams::{StateTeamAssignments, Teams},
    },
};

// ----------------- Move Enum -----------------
#[derive(Clone, Debug)]
enum Move {
    Play(Arc<CardInstance>, Vec<DataDepsFilled>),
    Move(Vector2Int),
    // Rest,
    // Serve,
    EndTurn,
}

// ----------------- Player Enum -----------------
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Players {
    Red,
    Blue,
}

// ----------------- Game State -----------------
#[derive(Clone)]
struct AIGameSimulation {
    // game meta
    turn: usize,
    terminal: bool,
    current_player: Players,

    //
    game_state: GameState,
    event_runner: CardEventRunner,
}

// ----------------- GameState Implementation -----------------
impl MCTSGameState for AIGameSimulation {
    type Move = Move;
    type Player = Players;
    type MoveList = Vec<Move>;

    fn current_player(&self) -> Self::Player {
        self.current_player
    }

    fn available_moves(&self) -> Vec<Move> {
        if self.terminal {
            return vec![];
        }

        let mut moves = Vec::new();
        match self.current_player {
            Players::Red => {
                let state_teams = self.game_state.get_value2::<StateTeamAssignments>();
                let state_deck = self.game_state.get_value2::<StateDeck>();

                let Some(uids) = state_teams.team_assignments.get(&Teams::Red) else {
                    return moves;
                };

                let Some(deck) = state_deck.deck.get(&uids[0]) else {
                    return moves;
                };

                for card in &deck.hand_consumable {
                    // make sure this card is valud based on its requirements
                    if card.has_statement(&self.game_state, uids[0]) {
                        let mut modifier_targets: Vec<Vec<DataDepsFilled>> = Vec::new();

                        for modifier in card.get_attributes_modifiers(&self.game_state, uids[0]) {
                            // let mut possible_targets_for_modifier = Vec::new();

                            for dep in modifier.get_data_dependencies_empty() {
                                match dep {
                                    DataDepsEmpty::Tiles(target_type) => {
                                        let targets: Vec<DataDepsFilled> = match target_type {
                                            // crate::cards::attribute_target_type_tiles::AttributeTargetTypesTiles::Select => todo!(),
                                            // crate::cards::attribute_target_type_tiles::AttributeTargetTypesTiles::RandomAny => todo!(),
                                            // crate::cards::attribute_target_type_tiles::AttributeTargetTypesTiles::RandomOnTeamUser => todo!(),
                                            crate::cards::attribute_target_type_tiles::AttributeTargetTypesTiles::RandomOnTeamOpponent => {
                                                let mut permuatations = vec![];

                                                let state_team = self.game_state.get_value2::<StateTeamAssignments>();
                                                let other_team = state_team.team_for(&uids[0]).unwrap().next_team();

                                                let min = GameBoard::get_bounds_min(&other_team);
                                                let max = GameBoard::get_bounds_max(&other_team);
                                                for x in min.x..max.x {
                                                    for y in min.y..max.y {
                                                        permuatations.push(DataDepsFilled::Tiles(vec![Vector2Int::new(x, y)]));
                                                    }
                                                }
                                                permuatations
                                            }
                                            // crate::cards::attribute_target_type_tiles::AttributeTargetTypesTiles::RandomInRangeGlobal(vector2_int, vector2_int1) => todo!(),
                                            // crate::cards::attribute_target_type_tiles::AttributeTargetTypesTiles::RandomInRangeLocal(vector2_int, vector2_int1) => todo!(),
                                            _ => vec![DataDepsFilled::Tiles(vec![])],
                                        };
                                        modifier_targets.push(targets);
                                    }
                                    DataDepsEmpty::Entities(target_type) => {
                                        let targets: Vec<DataDepsFilled> = match target_type {
                                            AttribtuteTargetTypesEntities::User => vec![DataDepsFilled::Entities(vec![uids[0]])],
                                            AttribtuteTargetTypesEntities::Select => {
                                                let mut permuatations = vec![];
                                                for x in self
                                                    .game_state
                                                    .get_value2::<StateTeamAssignments>()
                                                    .team_assignments
                                                {
                                                    for usr in x.1 {
                                                        permuatations.push(DataDepsFilled::Entities(vec![usr]));
                                                    }
                                                }
                                                permuatations
                                            }
                                            AttribtuteTargetTypesEntities::RandomAny => {
                                                let state_team = self.game_state.get_value2::<StateTeamAssignments>();
                                                let random_team = state_team.team_assignments.get(&Teams::random()).unwrap();
                                                let random_user_any = random_team
                                                    .get(Random::range_int(0, random_team.len() as i32) as usize)
                                                    .unwrap();
                                                vec![DataDepsFilled::Entities(vec![*random_user_any])]
                                            }
                                            AttribtuteTargetTypesEntities::RandomOpponent => {
                                                let state_team = self.game_state.get_value2::<StateTeamAssignments>();

                                                let other_team = state_team.team_for(&uids[0]).unwrap().next_team();
                                                let uids = state_team.team_assignments.get(&other_team).unwrap();
                                                let random_user_opponent = uids
                                                    .get(Random::range_int(0, uids.len() as i32) as usize)
                                                    .unwrap();
                                                vec![DataDepsFilled::Entities(vec![*random_user_opponent])]
                                            }
                                        };

                                        modifier_targets.push(targets);
                                    }
                                    _ => {}
                                }
                            }
                        }

                        if !modifier_targets.is_empty() {
                            // Start with an initial "empty" combination
                            let mut combined_permutations: Vec<Vec<DataDepsFilled>> = vec![Vec::new()];

                            // For each modifier's possible target permutations...
                            for modifier_permutations in modifier_targets {
                                let mut new_combinations = Vec::new();

                                // Combine existing partial combinations with each permutation of this modifier
                                for existing in &combined_permutations {
                                    for permutation in &modifier_permutations {
                                        let mut combined = existing.clone();
                                        combined.push(permutation.clone());
                                        new_combinations.push(combined);
                                    }
                                }

                                combined_permutations = new_combinations;
                            }

                            // Now each element of combined_permutations is a Vec<Vec<i32>>
                            // representing all modifier-target sets for one move.
                            for combo in combined_permutations {
                                moves.push(Move::Play(Arc::clone(card), combo));
                            }
                        } else {
                            // No modifier targets — still produce a base move
                            moves.push(Move::Play(Arc::clone(card), vec![]));
                        }
                    }
                }

                for card in &deck.hand_persistent {
                    // make sure this card is valud based on its requirements
                    if card.has_statement(&self.game_state, uids[0]) {
                        let mut modifier_targets: Vec<Vec<DataDepsFilled>> = Vec::new();

                        for modifier in card.get_attributes_modifiers(&self.game_state, uids[0]) {
                            // let mut possible_targets_for_modifier = Vec::new();

                            for dep in modifier.get_data_dependencies_empty() {
                                match dep {
                                    DataDepsEmpty::Tiles(target_type) => {
                                        let targets: Vec<DataDepsFilled> = match target_type {
                                            // crate::cards::attribute_target_type_tiles::AttributeTargetTypesTiles::Select => todo!(),
                                            // crate::cards::attribute_target_type_tiles::AttributeTargetTypesTiles::RandomAny => todo!(),
                                            // crate::cards::attribute_target_type_tiles::AttributeTargetTypesTiles::RandomOnTeamUser => todo!(),
                                            crate::cards::attribute_target_type_tiles::AttributeTargetTypesTiles::RandomOnTeamOpponent => {
                                                let mut permuatations = vec![];

                                                let state_team = self.game_state.get_value2::<StateTeamAssignments>();
                                                let other_team = state_team.team_for(&uids[0]).unwrap().next_team();

                                                let min = GameBoard::get_bounds_min(&other_team);
                                                let max = GameBoard::get_bounds_max(&other_team);
                                                for x in min.x..max.x {
                                                    for y in min.y..max.y {
                                                        permuatations.push(DataDepsFilled::Tiles(vec![Vector2Int::new(x, y)]));
                                                    }
                                                }
                                                permuatations
                                            }
                                            // crate::cards::attribute_target_type_tiles::AttributeTargetTypesTiles::RandomInRangeGlobal(vector2_int, vector2_int1) => todo!(),
                                            // crate::cards::attribute_target_type_tiles::AttributeTargetTypesTiles::RandomInRangeLocal(vector2_int, vector2_int1) => todo!(),
                                            _ => vec![DataDepsFilled::Tiles(vec![])],
                                        };
                                        modifier_targets.push(targets);
                                    }
                                    DataDepsEmpty::Entities(target_type) => {
                                        let targets: Vec<DataDepsFilled> = match target_type {
                                            AttribtuteTargetTypesEntities::User => vec![DataDepsFilled::Entities(vec![uids[0]])],
                                            AttribtuteTargetTypesEntities::Select => {
                                                let mut permuatations = vec![];
                                                for x in self
                                                    .game_state
                                                    .get_value2::<StateTeamAssignments>()
                                                    .team_assignments
                                                {
                                                    for usr in x.1 {
                                                        permuatations.push(DataDepsFilled::Entities(vec![usr]));
                                                    }
                                                }
                                                permuatations
                                            }
                                            AttribtuteTargetTypesEntities::RandomAny => {
                                                let state_team = self.game_state.get_value2::<StateTeamAssignments>();
                                                let random_team = state_team.team_assignments.get(&Teams::random()).unwrap();
                                                let random_user_any = random_team
                                                    .get(Random::range_int(0, random_team.len() as i32) as usize)
                                                    .unwrap();
                                                vec![DataDepsFilled::Entities(vec![*random_user_any])]
                                            }
                                            AttribtuteTargetTypesEntities::RandomOpponent => {
                                                let state_team = self.game_state.get_value2::<StateTeamAssignments>();

                                                let other_team = state_team.team_for(&uids[0]).unwrap().next_team();
                                                let uids = state_team.team_assignments.get(&other_team).unwrap();
                                                let random_user_opponent = uids
                                                    .get(Random::range_int(0, uids.len() as i32) as usize)
                                                    .unwrap();
                                                vec![DataDepsFilled::Entities(vec![*random_user_opponent])]
                                            }
                                        };

                                        modifier_targets.push(targets);
                                    }
                                    _ => {}
                                }
                            }
                        }

                        if !modifier_targets.is_empty() {
                            // Start with an initial "empty" combination
                            let mut combined_permutations: Vec<Vec<DataDepsFilled>> = vec![Vec::new()];

                            // For each modifier's possible target permutations...
                            for modifier_permutations in modifier_targets {
                                let mut new_combinations = Vec::new();

                                // Combine existing partial combinations with each permutation of this modifier
                                for existing in &combined_permutations {
                                    for permutation in &modifier_permutations {
                                        let mut combined = existing.clone();
                                        combined.push(permutation.clone());
                                        new_combinations.push(combined);
                                    }
                                }

                                combined_permutations = new_combinations;
                            }

                            // Now each element of combined_permutations is a Vec<Vec<i32>>
                            // representing all modifier-target sets for one move.
                            for combo in combined_permutations {
                                moves.push(Move::Play(Arc::clone(card), combo));
                            }
                        } else {
                            // No modifier targets — still produce a base move
                            moves.push(Move::Play(Arc::clone(card), vec![]));
                        }
                    }
                }
            }
            Players::Blue => {
                // let state_teams = self.game_state.get_value2::<StateTeamAssignments>();
                // let state_deck = self.game_state.get_value2::<StateDeck>();

                // let Some(uids) = state_teams.team_assignments.get(&Teams::Blue) else {
                //     return moves;
                // };

                // let Some(deck) = state_deck.deck.get(&uids[0]) else {
                //     return moves;
                // };

                // for card in &deck.hand_consumable {
                //     // make sure this card is valud based on its requirements
                //     if card.has_statement(&self.game_state, uids[0]) {
                //         let mut modifier_targets: Vec<Vec<Vec<i32>>> = Vec::new();

                //         for modifier in card.get_attributes_modifiers(&self.game_state, uids[0]) {
                //             // let mut possible_targets_for_modifier = Vec::new();

                //             for dep in modifier.get_data_dependencies_empty() {
                //                 match dep {
                //                     DataDepsEmpty::Entities(target_type) => {
                //                         let targets: Vec<Vec<i32>> = match target_type {
                //                             AttribtuteTargetTypesEntities::User => vec![vec![uids[0]]],
                //                             AttribtuteTargetTypesEntities::Select => {
                //                                 let mut permuatations = vec![];
                //                                 for x in self
                //                                     .game_state
                //                                     .get_value2::<StateTeamAssignments>()
                //                                     .team_assignments
                //                                 {
                //                                     for usr in x.1 {
                //                                         permuatations.push(vec![usr]);
                //                                     }
                //                                 }
                //                                 permuatations
                //                             }
                //                             AttribtuteTargetTypesEntities::RandomAny => {
                //                                 let state_team = self.game_state.get_value2::<StateTeamAssignments>();
                //                                 let random_team = state_team.team_assignments.get(&Teams::random()).unwrap();
                //                                 let random_user_any = random_team
                //                                     .get(Random::range_int(0, random_team.len() as i32) as usize)
                //                                     .unwrap();
                //                                 vec![vec![*random_user_any]]
                //                             }
                //                             AttribtuteTargetTypesEntities::RandomOpponent => {
                //                                 let state_team = self.game_state.get_value2::<StateTeamAssignments>();

                //                                 let other_team = state_team.team_for(&uids[0]).unwrap().next_team();
                //                                 let uids = state_team.team_assignments.get(&other_team).unwrap();
                //                                 let random_user_opponent = uids
                //                                     .get(Random::range_int(0, uids.len() as i32) as usize)
                //                                     .unwrap();
                //                                 vec![vec![*random_user_opponent]]
                //                             }
                //                         };

                //                         modifier_targets.push(targets);
                //                     }
                //                     _ => {}
                //                 }
                //             }
                //         }

                //         if !modifier_targets.is_empty() {
                //             // Start with an initial "empty" combination
                //             let mut combined_permutations: Vec<Vec<Vec<i32>>> = vec![Vec::new()];

                //             // For each modifier's possible target permutations...
                //             for modifier_permutations in modifier_targets {
                //                 let mut new_combinations = Vec::new();

                //                 // Combine existing partial combinations with each permutation of this modifier
                //                 for existing in &combined_permutations {
                //                     for permutation in &modifier_permutations {
                //                         let mut combined = existing.clone();
                //                         combined.push(permutation.clone());
                //                         new_combinations.push(combined);
                //                     }
                //                 }

                //                 combined_permutations = new_combinations;
                //             }

                //             // Now each element of combined_permutations is a Vec<Vec<i32>>
                //             // representing all modifier-target sets for one move.
                //             for combo in combined_permutations {
                //                 moves.push(Move::Play(Arc::clone(card), combo));
                //             }
                //         } else {
                //             // No modifier targets — still produce a base move
                //             moves.push(Move::Play(Arc::clone(card), vec![]));
                //         }
                //     }
                // }

                // for card in &deck.hand_persistent {
                // make sure this card is valud based on its requirements
                //     if card.has_statement(&self.game_state, uids[0]) {
                //         let mut modifier_targets: Vec<Vec<Vec<i32>>> = Vec::new();

                //         for modifier in card.get_attributes_modifiers(&self.game_state, uids[0]) {
                //             // let mut possible_targets_for_modifier = Vec::new();

                //             for dep in modifier.get_data_dependencies_empty() {
                //                 match dep {
                //                     DataDepsEmpty::Entities(target_type) => {
                //                         let targets: Vec<Vec<i32>> = match target_type {
                //                             AttribtuteTargetTypesEntities::User => vec![vec![uids[0]]],
                //                             AttribtuteTargetTypesEntities::Select => {
                //                                 let mut permuatations = vec![];
                //                                 for x in self
                //                                     .game_state
                //                                     .get_value2::<StateTeamAssignments>()
                //                                     .team_assignments
                //                                 {
                //                                     for usr in x.1 {
                //                                         permuatations.push(vec![usr]);
                //                                     }
                //                                 }
                //                                 permuatations
                //                             }
                //                             AttribtuteTargetTypesEntities::RandomAny => {
                //                                 let state_team = self.game_state.get_value2::<StateTeamAssignments>();
                //                                 let random_team = state_team.team_assignments.get(&Teams::random()).unwrap();
                //                                 let random_user_any = random_team
                //                                     .get(Random::range_int(0, random_team.len() as i32) as usize)
                //                                     .unwrap();
                //                                 vec![vec![*random_user_any]]
                //                             }
                //                             AttribtuteTargetTypesEntities::RandomOpponent => {
                //                                 let state_team = self.game_state.get_value2::<StateTeamAssignments>();

                //                                 let other_team = state_team.team_for(&uids[0]).unwrap().next_team();
                //                                 let uids = state_team.team_assignments.get(&other_team).unwrap();
                //                                 let random_user_opponent = uids
                //                                     .get(Random::range_int(0, uids.len() as i32) as usize)
                //                                     .unwrap();
                //                                 vec![vec![*random_user_opponent]]
                //                             }
                //                         };

                //                         modifier_targets.push(targets);
                //                     }
                //                     _ => {}
                //                 }
                //             }
                //         }

                //         if !modifier_targets.is_empty() {
                //             // Start with an initial "empty" combination
                //             let mut combined_permutations: Vec<Vec<Vec<i32>>> = vec![Vec::new()];

                //             // For each modifier's possible target permutations...
                //             for modifier_permutations in modifier_targets {
                //                 let mut new_combinations = Vec::new();

                //                 // Combine existing partial combinations with each permutation of this modifier
                //                 for existing in &combined_permutations {
                //                     for permutation in &modifier_permutations {
                //                         let mut combined = existing.clone();
                //                         combined.push(permutation.clone());
                //                         new_combinations.push(combined);
                //                     }
                //                 }

                //                 combined_permutations = new_combinations;
                //             }

                //             // Now each element of combined_permutations is a Vec<Vec<i32>>
                //             // representing all modifier-target sets for one move.
                //             for combo in combined_permutations {
                //                 moves.push(Move::Play(Arc::clone(card), combo));
                //             }
                //         } else {
                //             // No modifier targets — still produce a base move
                //             moves.push(Move::Play(Arc::clone(card), vec![]));
                //         }
                //     }
                // }
            }
        }

        // movement
        moves.push(Move::Move(Vector2Int::new(0, 1)));
        moves.push(Move::Move(Vector2Int::new(0, -1)));
        moves.push(Move::Move(Vector2Int::new(1, 0)));
        moves.push(Move::Move(Vector2Int::new(-1, 0)));

        // peristent
        // moves.push(Move::Rest);

        // end
        moves.push(Move::EndTurn);

        // return
        moves
    }

    fn make_move(&mut self, mov: &Self::Move) {
        match (self.current_player, mov) {
            (Players::Red, Move::Play(card, data)) => {
                println!("Red: Play");

                let state_teams = self.game_state.get_value2::<StateTeamAssignments>();
                let Some(uids) = state_teams.team_assignments.get(&Teams::Red) else {
                    return;
                };

                let mut data_queue = VecDeque::new();
                data_queue.extend(data);
                // add modifiers
                for modifier in card.get_attributes_modifiers(&self.game_state, uids[0]) {
                    self.event_runner.enqueue_modifier(&modifier);
                }
                // add event
                for event in card.get_attributes_events(&self.game_state, uids[0]) {
                    self.event_runner.enqueue_event(&event, &mut data_queue);
                }
                // post all
                self.event_runner.post_and_drain(&mut self.game_state);
            }
            (Players::Blue, Move::Play(card, data)) => {
                println!("Blue: Play");

                let state_teams = self.game_state.get_value2::<StateTeamAssignments>();
                let Some(uids) = state_teams.team_assignments.get(&Teams::Blue) else {
                    return;
                };

                let mut data_queue = VecDeque::new();
                data_queue.extend(data);
                // add modifiers
                for modifier in card.get_attributes_modifiers(&self.game_state, uids[0]) {
                    self.event_runner.enqueue_modifier(&modifier);
                }
                // add event
                for event in card.get_attributes_events(&self.game_state, uids[0]) {
                    self.event_runner.enqueue_event(&event, &mut data_queue);
                }
                // post all
                self.event_runner.post_and_drain(&mut self.game_state);
            }
            // (Players::Red, Move::Rest) => {
            //     println!("Red: Rest");
            //     // get uid for team
            //     let state_teams = self.game_state.get_value2::<StateTeamAssignments>();
            //     let Some(uids) = state_teams.team_assignments.get(&Teams::Red) else {
            //         return;
            //     };

            //     // edit -> deck
            //     self.game_state.edit::<StateDeck>(|x| {
            //         // get deck
            //         let Some(deck) = x.deck.get_mut(&uids[0]) else {
            //             return;
            //         };

            //         // discard old hand
            //         for i in (0..deck.hand_consumable.len()).rev() {
            //             let c = deck.hand_consumable[i].clone();
            //             deck.discard(c);
            //         }

            //         // draw new hand
            //         for _i in 0..5 {
            //             deck.draw();
            //         }
            //     });

            //     // edit -> energy
            //     self.game_state.edit::<StateEnergy>(|x| {
            //         // reduce max energy by 1
            //         let energy = x.all_players[&uids[0]];
            //         x.all_players.insert(uids[0], (energy.0, energy.1 - 1));
            //     });
            // }
            (Players::Red, Move::EndTurn) => {
                println!("Red: End");

                //
                let state_teams = self.game_state.get_value2::<StateTeamAssignments>();
                let Some(uids) = state_teams.team_assignments.get(&Teams::Red) else {
                    return;
                };
                // edit -> energy
                self.game_state.edit::<StateEnergy>(|x| {
                    // reset energy
                    let energy = x.all_players[&uids[0]];
                    x.all_players.insert(uids[0], (energy.1, energy.1));
                });
                self.turn += 1;
                self.current_player = Players::Blue;
            }

            // (Players::Blue, Move::Rest) => {
            //     println!("Blue: Rest");

            //     // get uid for team
            //     let state_teams = self.game_state.get_value2::<StateTeamAssignments>();
            //     let Some(uids) = state_teams.team_assignments.get(&Teams::Blue) else {
            //         return;
            //     };

            //     // edit -> deck
            //     self.game_state.edit::<StateDeck>(|x| {
            //         // get deck
            //         let Some(deck) = x.deck.get_mut(&uids[0]) else {
            //             return;
            //         };

            //         // discard old hand
            //         for i in (0..deck.hand_consumable.len()).rev() {
            //             let c = deck.hand_consumable[i].clone();
            //             deck.discard(c);
            //         }

            //         // draw new hand
            //         for _i in 0..5 {
            //             deck.draw();
            //         }
            //     });

            //     // edit -> energy
            //     self.game_state.edit::<StateEnergy>(|x| {
            //         // reduce max energy by 1
            //         let energy = x.all_players[&uids[0]];
            //         x.all_players.insert(uids[0], (energy.0, energy.1 - 1));
            //     });
            // }
            (Players::Blue, Move::EndTurn) => {
                println!("Blue: End");

                //
                let state_teams = self.game_state.get_value2::<StateTeamAssignments>();
                let Some(uids) = state_teams.team_assignments.get(&Teams::Blue) else {
                    return;
                };
                // edit -> energy
                self.game_state.edit::<StateEnergy>(|x| {
                    // reset energy
                    let energy = x.all_players[&uids[0]];
                    x.all_players.insert(uids[0], (energy.1, energy.1));
                });

                self.turn += 1;
                self.current_player = Players::Blue;
            }
            (Players::Red, Move::Move(delta)) => {
                println!("Red: Move");

                let state_teams = self.game_state.get_value2::<StateTeamAssignments>();
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
            }
            (Players::Blue, Move::Move(delta)) => {
                println!("Blue: Move");

                let state_teams = self.game_state.get_value2::<StateTeamAssignments>();
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
            }
        }

        // end condition
        match self.current_player {
            Players::Red => {
                if self.game_state.get_value2::<StatePositionBall>().row < 2 {
                    self.terminal = true;
                }
            }
            Players::Blue => {
                if self.game_state.get_value2::<StatePositionBall>().row >= 2 {
                    self.terminal = true;
                }
            }
        }
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
        self.game_state
            .get_value2::<StateTeamAssignments>()
            .hash(&mut hasher);
        self.game_state
            .get_value2::<StatePositionPlayer>()
            .hash(&mut hasher);
        self.game_state
            .get_value2::<StatePositionBall>()
            .hash(&mut hasher);
        self.game_state
            .get_value2::<StateEnergy>()
            .hash(&mut hasher);
        self.game_state.get_value2::<StateDeck>().hash(&mut hasher);
        self.game_state
            .get_value2::<StateCardAttributeModifierStack>()
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
            Players::Red => {
                if state.game_state.get_value2::<StatePositionBall>().row < 2 {
                    0
                } else {
                    let state_teams = state.game_state.get_value2::<StateTeamAssignments>();
                    let state_energy = state.game_state.get_value2::<StateEnergy>();

                    let Some(uids) = state_teams.team_assignments.get(&Teams::Red) else {
                        //
                        return (vec![(); moves.len()], 0);
                    };

                    let Some(energy) = state_energy.all_players.get(&uids[0]) else {
                        //
                        return (vec![(); moves.len()], 0);
                    };

                    // (max_energy * 2 ) + (cur_energy) + opponent_distance_from_ball
                    ((energy.1 * 2) + (energy.0 * 2)) as i64
                }
            }
            Players::Blue => {
                if state.game_state.get_value2::<StatePositionBall>().row >= 2 {
                    0
                } else {
                    let state_teams = state.game_state.get_value2::<StateTeamAssignments>();
                    let state_energy = state.game_state.get_value2::<StateEnergy>();

                    let Some(uids) = state_teams.team_assignments.get(&Teams::Blue) else {
                        //
                        return (vec![(); moves.len()], 0);
                    };

                    let Some(energy) = state_energy.all_players.get(&uids[0]) else {
                        //
                        return (vec![(); moves.len()], 0);
                    };

                    // (max_energy * 2 ) + (cur_energy) + opponent_distance_from_ball
                    ((energy.1 * 2) + (energy.0 * 2)) as i64
                }
            }
        };
        (vec![(); moves.len()], score)
    }

    fn interpret_evaluation_for_player(&self, evaln: &i64, _player: &Players) -> i64 {
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
pub fn run_ai(game_state: &mut GameState) {
    let local_game_state = GameState::new_single_instance(vec![
        (StateCardAttributeModifierStack::id(), Box::new(game_state.get_value2::<StateCardAttributeModifierStack>())),
        (StateTeamAssignments::id(), Box::new(game_state.get_value2::<StateTeamAssignments>())),
        (StatePositionPlayer::id(), Box::new(game_state.get_value2::<StatePositionPlayer>())), //
        (StatePositionBall::id(), Box::new(game_state.get_value2::<StatePositionBall>())),
        (StateBallMode::id(), Box::new(game_state.get_value2::<StateBallMode>())),
        (StateEnergy::id(), Box::new(game_state.get_value2::<StateEnergy>())),
        (StateDeck::id(), Box::new(game_state.get_value2::<StateDeck>())),
    ]);
    // create simulation state
    let sim = AIGameSimulation {
        turn: 0,
        terminal: false,
        current_player: Players::Red, // AI goes first
        game_state: local_game_state,
        event_runner: CardEventRunner::new(),
    };

    // Build MCTS manager — following docs.rs example style
    let policy = UCTPolicy::new(0.5);
    let table = ApproxTable::new(1024); // tune size
    let mut manager = mcts::MCTSManager::new(sim, MyMCTS, AiGameEvaluator, policy, table);

    // Run playouts — choose iterations & threads appropriate for your runtime.
    // e.g. 2000 playouts with 4 threads (tune for performance)
    manager.playout_n_parallel(2000, 4);

    // Retrieve best move from manager
    if let Some(best_move) = manager.best_move() {
        println!("MCTS Best Move: {:?}", best_move);
        // You can also get principal variation: manager.principal_variation(n)
    } else {
        println!("No valid move found by MCTS");
    }
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
            event_reciever_apply_card_attribute_event_move_ball_forward::EventReciever::recieve,
            // events
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
    fn enqueue_event(&mut self, event: &CardAttributeEvents, data: &mut VecDeque<&DataDepsFilled>) {
        match event {
            CardAttributeEvents::MoveBall(_) => self
                .runner
                .enqueue(&CardEvents::ApplyEventMoveBall(0, 0, data.pop_front().unwrap().clone())),

            _ => {}
        }
    }
    fn post_and_drain(&mut self, game_state: &mut GameState) {
        println!("post and drain start");
        self.runner.post_and_drain(game_state);
        println!("post and drain complete");
    }
}
