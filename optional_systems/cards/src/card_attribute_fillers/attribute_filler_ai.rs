use curio_core::{Vector2Int, collections::ledger::Ledger, random::Random};

use crate::{
    cards::{
        card_attributes_targets::{attribute_target_type_cards::AttributeTargetTypesCards, attribute_target_type_entities::AttribtuteTargetTypesEntities, attribute_target_type_tiles::AttributeTargetTypesTiles},
        card_dependencies::{builder::data_dep_filled_all_permutations::DataDepsFilledAllPermutations, data_dep_filled::DataDepsFilled},
    },
    game_board::GameBoard,
    state::{
        host::state_card_attribute_modifier_stack::StateCardAttributeModifierStack,
        state_deck::StateDeck,
        state_position_ball::StatePositionBall,
        state_position_player::StatePositionEntities,
        state_teams::{StateTeamAssignments, Teams},
        state_turn::StateTurn,
    },
};
pub struct CardAttributeFillerAI {}
impl CardAttributeFillerAI {
    pub fn fill_dependency_tiles(ledger: &Ledger, uid: &i32, empty: AttributeTargetTypesTiles) -> DataDepsFilledAllPermutations {
        // create the list of permutations
        let mut permuatations = DataDepsFilledAllPermutations::new();

        // match for empty type
        match empty {
            AttributeTargetTypesTiles::SelectInRangeLocalToBall(min, max) => {
                let state_modifiers = ledger.read::<StateCardAttributeModifierStack>();
                let state_turn = ledger.read::<StateTurn>();
                let state = ledger.read::<StatePositionEntities>();
                let state_position_ball = state.positions.get(uid).unwrap();

                // get the modifiers for stack
                let modifier_stack = state_modifiers.get_flat_stack_for_entity(*uid);
                let team = &state_turn.active_instance_id;

                // get the min and max taking into account any modifiers
                let min = team.convert_dir(min.x, min.y + modifier_stack.range);
                let max = team.convert_dir(max.x, max.y + modifier_stack.range);

                // get between min and max in range
                let random_x = Random::range_int(min.0, max.0);
                let random_z = Random::range_int(min.1, max.1);

                let col = state_position_ball.0 + random_x;
                let row = state_position_ball.1 + random_z;

                permuatations.add_permutation(DataDepsFilled::Tiles(vec![Vector2Int::new(col, row)]));
            }
            AttributeTargetTypesTiles::SelectOnTeamUser | AttributeTargetTypesTiles::RandomOnTeamUser => {
                // get state
                let state_team = ledger.read::<StateTeamAssignments>();

                // get the team for this user
                let Some(my_team) = state_team.team_for(&uid) else {
                    println!("Failed to find team for uid: {}", uid);
                    return permuatations;
                };

                let min = GameBoard::get_bounds_min_for_team(&my_team);
                let max = GameBoard::get_bounds_max_for_team(&my_team);

                let random_x = Random::range_int(min.x, max.x);
                let random_z = Random::range_int(min.y, max.y);

                permuatations.add_permutation(DataDepsFilled::Tiles(vec![Vector2Int::new(random_x, random_z)]));
            }
            AttributeTargetTypesTiles::SelectOnTeamOpponent | AttributeTargetTypesTiles::RandomOnTeamOpponent => {
                // get state
                let state_team = ledger.read::<StateTeamAssignments>();

                // get the team for this user
                let Some(my_team) = state_team.team_for(&uid) else {
                    println!("Failed to find team for uid: {}", uid);
                    return permuatations;
                };

                let other_team = my_team.next_team();

                let min = GameBoard::get_bounds_min_for_team(&other_team);
                let max = GameBoard::get_bounds_max_for_team(&other_team);

                let random_x = Random::range_int(min.x, max.x);
                let random_z = Random::range_int(min.y, max.y);

                permuatations.add_permutation(DataDepsFilled::Tiles(vec![Vector2Int::new(random_x, random_z)]));
            }
            AttributeTargetTypesTiles::RandomInRangeLocalToBall(min, max) => {
                let state_modifiers = ledger.read::<StateCardAttributeModifierStack>();
                let state_turn = ledger.read::<StateTurn>();
                let state_position_ball = ledger.read::<StatePositionBall>();

                // get the modifiers for stack
                let modifier_stack = state_modifiers.get_flat_stack_for_entity(*uid);
                let team = &state_turn.active_instance_id;

                // get the min and max taking into account any modifiers
                let min = team.convert_dir(min.x, min.y + modifier_stack.range);
                let max = team.convert_dir(max.x, max.y + modifier_stack.range);

                // get between min and max in range
                let random_x = Random::range_int(min.0, max.0);
                let random_z = Random::range_int(min.1, max.1);

                let col = state_position_ball.column + random_x;
                let row = state_position_ball.row + random_z;

                permuatations.add_permutation(DataDepsFilled::Tiles(vec![Vector2Int::new(col, row)]));
            }
            AttributeTargetTypesTiles::RandomInRangeLocalToUser(min, max) => {
                let state_modifiers = ledger.read::<StateCardAttributeModifierStack>();
                let state_turn = ledger.read::<StateTurn>();
                let state = ledger.read::<StatePositionEntities>();
                let state_position_ball = state.positions.get(uid).unwrap();

                // get the modifiers for stack
                let modifier_stack = state_modifiers.get_flat_stack_for_entity(*uid);
                let team = &state_turn.active_instance_id;

                // get the min and max taking into account any modifiers
                let min = team.convert_dir(min.x, min.y + modifier_stack.range);
                let max = team.convert_dir(max.x, max.y + modifier_stack.range);

                // get between min and max in range
                let random_x = Random::range_int(min.0, max.0);
                let random_z = Random::range_int(min.1, max.1);

                let col = state_position_ball.0 + random_x;
                let row = state_position_ball.1 + random_z;

                permuatations.add_permutation(DataDepsFilled::Tiles(vec![Vector2Int::new(col, row)]));
            }
            AttributeTargetTypesTiles::RandomInRangeGlobal(min, max) => {
                let random_x = Random::range_int(min.x, max.x);
                let random_z = Random::range_int(min.y, max.y);

                permuatations.add_permutation(DataDepsFilled::Tiles(vec![Vector2Int::new(random_x, random_z)]));
            }
            AttributeTargetTypesTiles::SelectAny => todo!(),
            AttributeTargetTypesTiles::RandomAny => todo!(),
            AttributeTargetTypesTiles::SelectOpponentBackCorner => todo!(),
        }

        // return the now filled permutations
        permuatations
    }
    pub fn fill_dependency_entities(ledger: &Ledger, uid: &i32, empty: AttribtuteTargetTypesEntities) -> DataDepsFilledAllPermutations {
        // create the list of permutations
        let mut permuatations = DataDepsFilledAllPermutations::new();

        match empty {
            AttribtuteTargetTypesEntities::User => {
                // add a permutation of your uid
                permuatations.add_permutation(DataDepsFilled::Entities(vec![uid.clone()]))
            }
            AttribtuteTargetTypesEntities::Select => {
                // get state
                let state_team = ledger.read::<StateTeamAssignments>();

                // iterate over each team + uids
                for team_ids in &state_team.team_assignments {
                    // iterate over each uid on each team
                    for uid in team_ids.1 {
                        // "select" each uid as a different permutation
                        permuatations.add_permutation(DataDepsFilled::Entities(vec![uid.clone()]));
                    }
                }
            }
            AttribtuteTargetTypesEntities::RandomAny => {
                // get state
                let state_team = ledger.read::<StateTeamAssignments>();

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
                let state_team = ledger.read::<StateTeamAssignments>();

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
    pub fn fill_dependency_cards(ledger: &Ledger, uid: &i32, empty: AttributeTargetTypesCards) -> DataDepsFilledAllPermutations {
        // create the list of permutations
        let mut permuatations = DataDepsFilledAllPermutations::new();

        match empty {
            AttributeTargetTypesCards::SelectUser => todo!(),
            AttributeTargetTypesCards::SelectOpponent => todo!(),
            AttributeTargetTypesCards::RandomUser => todo!(),
            AttributeTargetTypesCards::RandomOpponent => todo!(),
            AttributeTargetTypesCards::AllUser => {
                let state_deck = ledger.read::<StateDeck>();

                // if we cant find a deck break
                let Some(deck) = state_deck.deck.get(uid) else {
                    return permuatations;
                };

                // iterate through each consumable card in hand and record its uids
                let mut card_uids = Vec::new();
                for x in &deck.hand_consumable {
                    card_uids.push(x.instance_id);
                }

                // add all cards in hand
                permuatations.add_permutation(DataDepsFilled::Cards(card_uids));
            }
            AttributeTargetTypesCards::AllOpponent => todo!(),
        }

        permuatations
    }
}
