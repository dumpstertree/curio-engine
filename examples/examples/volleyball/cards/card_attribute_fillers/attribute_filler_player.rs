use curio_core::{Vector2Int, built_in::record::sys_record_network::SysRecordNetwork, collections::ledger::Ledger, random::Random};
use std::panic;

use crate::{
    cards::{
        card_attributes_targets::{attribute_target_type_cards::AttributeTargetTypesCards, attribute_target_type_entities::AttribtuteTargetTypesEntities, attribute_target_type_tiles::AttributeTargetTypesTiles},
        card_dependencies::{data_dep_empty::DataDepsEmpty, data_dep_filled::DataDepsFilled},
    },
    game_board::GameBoard,
    state::{host::state_card_attribute_modifier_stack::StateCardAttributeModifierStack, state_deck::StateDeck, state_position_ball::StatePositionBall, state_position_player::StatePositionEntities, state_teams::StateTeamAssignments, state_turn::StateTurn},
};

pub struct CardAttributeFillerPlayer {}

impl CardAttributeFillerPlayer {
    pub fn fill_event(ledger: &Ledger, id: &i32, dep: &DataDepsEmpty) -> DataDepsFilled {
        match dep {
            DataDepsEmpty::Entities(target_types_entities) => match target_types_entities {
                AttribtuteTargetTypesEntities::User => CardAttributeFillerPlayer::get_entity_user(id, ledger),
                AttribtuteTargetTypesEntities::Select => CardAttributeFillerPlayer::get_entity_select(ledger),
                AttribtuteTargetTypesEntities::RandomAny => CardAttributeFillerPlayer::get_entity_random(ledger),
                AttribtuteTargetTypesEntities::RandomOpponent => CardAttributeFillerPlayer::get_entity_opponent(id, ledger),
            },
            DataDepsEmpty::Cards(target_types_cards) => match target_types_cards {
                AttributeTargetTypesCards::SelectUser => CardAttributeFillerPlayer::get_card_user_selected(ledger),
                AttributeTargetTypesCards::RandomUser => CardAttributeFillerPlayer::get_card_user_random(id, ledger),
                AttributeTargetTypesCards::SelectOpponent => CardAttributeFillerPlayer::get_card_opponent_selected(ledger),
                AttributeTargetTypesCards::RandomOpponent => CardAttributeFillerPlayer::get_card_opponent_random(id, ledger),
                AttributeTargetTypesCards::AllUser => CardAttributeFillerPlayer::get_card_user_all(id, ledger),
                AttributeTargetTypesCards::AllOpponent => CardAttributeFillerPlayer::get_card_opponent_all(id, ledger),
            },
            DataDepsEmpty::Tiles(target_types_tiles) => match target_types_tiles {
                AttributeTargetTypesTiles::SelectOpponentBackCorner => {
                    todo!()
                }
                AttributeTargetTypesTiles::SelectAny => todo!(),
                AttributeTargetTypesTiles::RandomAny => CardAttributeFillerPlayer::get_tiles_random_any(ledger),
                AttributeTargetTypesTiles::RandomOnTeamUser => CardAttributeFillerPlayer::get_tiles_random_on_team_user(id, ledger),
                AttributeTargetTypesTiles::RandomOnTeamOpponent => CardAttributeFillerPlayer::get_tiles_random_on_team_opponent(id, ledger),
                AttributeTargetTypesTiles::RandomInRangeLocalToBall(min, max) => CardAttributeFillerPlayer::get_tiles_random_in_range_local(id, ledger, min, max),
                AttributeTargetTypesTiles::RandomInRangeGlobal(min, max) => CardAttributeFillerPlayer::get_tiles_random_in_range_global(id, ledger, min, max),
                AttributeTargetTypesTiles::RandomInRangeLocalToUser(min, max) => {
                    let uid = id;
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

                    DataDepsFilled::Tiles(vec![Vector2Int::new(col, row)])
                }
                AttributeTargetTypesTiles::SelectOnTeamUser => todo!(),
                AttributeTargetTypesTiles::SelectOnTeamOpponent => todo!(),
                AttributeTargetTypesTiles::SelectInRangeLocalToBall(_, _) => todo!(),
            },
        }
    }
    pub async fn fill_events(ledger: &Ledger, id: &i32, data_dep_empty: &Vec<DataDepsEmpty>) -> Vec<DataDepsFilled> {
        let mut filled = vec![];
        for dep in data_dep_empty {
            match dep {
                DataDepsEmpty::Entities(target_types_entities) => match target_types_entities {
                    AttribtuteTargetTypesEntities::User => filled.push(CardAttributeFillerPlayer::get_entity_user(id, ledger)),
                    AttribtuteTargetTypesEntities::Select => filled.push(CardAttributeFillerPlayer::get_entity_select(ledger)),
                    AttribtuteTargetTypesEntities::RandomAny => filled.push(CardAttributeFillerPlayer::get_entity_random(ledger)),
                    AttribtuteTargetTypesEntities::RandomOpponent => filled.push(CardAttributeFillerPlayer::get_entity_opponent(id, ledger)),
                },
                DataDepsEmpty::Cards(target_types_cards) => match target_types_cards {
                    AttributeTargetTypesCards::SelectUser => filled.push(CardAttributeFillerPlayer::get_card_user_selected(ledger)),
                    AttributeTargetTypesCards::RandomUser => filled.push(CardAttributeFillerPlayer::get_card_user_random(id, ledger)),
                    AttributeTargetTypesCards::SelectOpponent => filled.push(CardAttributeFillerPlayer::get_card_opponent_selected(ledger)),
                    AttributeTargetTypesCards::RandomOpponent => filled.push(CardAttributeFillerPlayer::get_card_opponent_random(id, ledger)),
                    AttributeTargetTypesCards::AllUser => filled.push(CardAttributeFillerPlayer::get_card_user_all(id, ledger)),
                    AttributeTargetTypesCards::AllOpponent => filled.push(CardAttributeFillerPlayer::get_card_opponent_all(id, ledger)),
                },
                DataDepsEmpty::Tiles(target_types_tiles) => match target_types_tiles {
                    AttributeTargetTypesTiles::SelectOpponentBackCorner => {
                        todo!()
                    }
                    AttributeTargetTypesTiles::SelectAny => filled.push(CardAttributeFillerPlayer::get_tiles_select(ledger)),
                    AttributeTargetTypesTiles::RandomAny => filled.push(CardAttributeFillerPlayer::get_tiles_random_any(ledger)),
                    AttributeTargetTypesTiles::RandomOnTeamUser => filled.push(CardAttributeFillerPlayer::get_tiles_random_on_team_user(id, ledger)),
                    AttributeTargetTypesTiles::RandomOnTeamOpponent => filled.push(CardAttributeFillerPlayer::get_tiles_random_on_team_opponent(id, ledger)),
                    AttributeTargetTypesTiles::RandomInRangeLocalToBall(min, max) => filled.push(CardAttributeFillerPlayer::get_tiles_random_in_range_local(id, ledger, min, max)),
                    AttributeTargetTypesTiles::RandomInRangeGlobal(min, max) => filled.push(CardAttributeFillerPlayer::get_tiles_random_in_range_global(id, ledger, min, max)),
                    AttributeTargetTypesTiles::RandomInRangeLocalToUser(min, max) => {
                        let uid = id;
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

                        filled.push(DataDepsFilled::Tiles(vec![Vector2Int::new(col, row)]));
                    }
                    AttributeTargetTypesTiles::SelectOnTeamUser => todo!(),
                    AttributeTargetTypesTiles::SelectOnTeamOpponent => todo!(),
                    AttributeTargetTypesTiles::SelectInRangeLocalToBall(_, _) => todo!(),
                },
            }
        }
        filled
    }
}

// get -> entity
impl CardAttributeFillerPlayer {
    fn get_entity_user(id: &i32, ledger: &Ledger) -> DataDepsFilled {
        let _cur_player_id = ledger.read::<StateTurn>().active_instance_id;
        DataDepsFilled::Entities(vec![*id])
    }
    fn get_entity_select(_ledger: &Ledger) -> DataDepsFilled {
        todo!()
    }
    fn get_entity_random(ledger: &Ledger) -> DataDepsFilled {
        let state_peers = ledger.read::<SysRecordNetwork>();
        let peer_instance_ids = state_peers.peer_instance_ids();
        let random_index = Random::range_int(0, peer_instance_ids.len().try_into().unwrap());
        let selected = peer_instance_ids.get(random_index as usize).unwrap();
        DataDepsFilled::Entities(vec![*selected])
    }
    fn get_entity_opponent(id: &i32, ledger: &Ledger) -> DataDepsFilled {
        let _state_turn = ledger.read::<StateTurn>();
        let state_team_assignment = ledger.read::<StateTeamAssignments>();
        let opponent_team = state_team_assignment.team_for(&id).unwrap().next_team();
        let opponent_ids = state_team_assignment
            .team_assignments
            .get(&opponent_team)
            .unwrap();

        DataDepsFilled::Entities(vec![*opponent_ids.first().unwrap()])
    }
}
// get -> cards
impl CardAttributeFillerPlayer {
    fn get_card_user_selected(_ledger: &Ledger) -> DataDepsFilled {
        todo!()
    }
    fn get_card_user_random(id: &i32, ledger: &Ledger) -> DataDepsFilled {
        let _state_turn = ledger.read::<StateTurn>();
        let state_deck = ledger.read::<StateDeck>();

        let Some(deck) = state_deck.deck.get(id) else {
            panic!("");
        };

        let min_index = 0 as i32;
        let max_index = deck.hand_consumable.len() as i32;
        if max_index - min_index <= 0 {
            panic!("");
        }

        let index = Random::range_int(min_index, max_index);
        let card = &deck.hand_consumable[index as usize];
        DataDepsFilled::Cards(vec![(card.instance_id)])
    }
    fn get_card_opponent_selected(_ledger: &Ledger) -> DataDepsFilled {
        todo!()
    }
    fn get_card_user_all(id: &i32, ledger: &Ledger) -> DataDepsFilled {
        let _state_turn = ledger.read::<StateTurn>();
        let state_deck = ledger.read::<StateDeck>();

        let Some(deck) = state_deck.deck.get(id) else {
            panic!("");
        };

        let mut result = vec![];
        for x in &deck.hand_consumable {
            result.push(x.instance_id);
        }

        DataDepsFilled::Cards(result)
    }
    fn get_card_opponent_all(id: &i32, ledger: &Ledger) -> DataDepsFilled {
        let _state_turn = ledger.read::<StateTurn>();
        let state_deck = ledger.read::<StateDeck>();
        let state_team_assignment = ledger.read::<StateTeamAssignments>();
        let opponent_team = state_team_assignment.team_for(id).unwrap().next_team();
        let opponent_ids = state_team_assignment
            .team_assignments
            .get(&opponent_team)
            .unwrap();

        let opponent_id = opponent_ids.first().unwrap();
        let Some(deck) = state_deck.deck.get(opponent_id) else {
            panic!("");
        };

        let mut result = vec![];
        for x in &deck.hand_consumable {
            result.push(x.instance_id);
        }

        DataDepsFilled::Cards(result)
    }
    fn get_card_opponent_random(id: &i32, ledger: &Ledger) -> DataDepsFilled {
        let _state_turn = ledger.read::<StateTurn>();
        let state_deck = ledger.read::<StateDeck>();
        let state_team_assignment = ledger.read::<StateTeamAssignments>();
        let opponent_team = state_team_assignment.team_for(id).unwrap().next_team();
        let opponent_ids = state_team_assignment
            .team_assignments
            .get(&opponent_team)
            .unwrap();

        let opponent_id = opponent_ids.first().unwrap();
        let Some(deck) = state_deck.deck.get(opponent_id) else {
            panic!("");
        };

        let min_index = 0 as i32;
        let max_index = deck.hand_consumable.len() as i32;
        if max_index - min_index <= 0 {
            panic!("");
        }

        let index = Random::range_int(min_index, max_index);
        let card = &deck.hand_consumable[index as usize];

        DataDepsFilled::Cards(vec![card.instance_id])
    }
}
// get -> tiles
impl CardAttributeFillerPlayer {
    pub fn get_tiles_select(_ledger: &Ledger) -> DataDepsFilled {
        panic!("");
    }
    pub fn get_tiles_random_any(_ledger: &Ledger) -> DataDepsFilled {
        DataDepsFilled::Tiles(vec![Vector2Int::new(Random::range_int(0, 4), Random::range_int(0, 4))])
    }
    pub fn get_tiles_random_on_team_user(id: &i32, ledger: &Ledger) -> DataDepsFilled {
        let _state_turn = ledger.read::<StateTurn>();
        let state_team_assignment = ledger.read::<StateTeamAssignments>();

        let Some(cur_team) = state_team_assignment.team_for(id) else {
            panic!("");
        };

        let bounds_min = GameBoard::get_bounds_min_for_team(&cur_team);
        let bounds_max = GameBoard::get_bounds_max_for_team(&cur_team);

        DataDepsFilled::Tiles(vec![Vector2Int::new(Random::range_int(bounds_min.x, bounds_max.x), Random::range_int(bounds_min.y, bounds_max.y))])
    }
    pub fn get_tiles_random_on_team_opponent(id: &i32, ledger: &Ledger) -> DataDepsFilled {
        let _state_turn = ledger.read::<StateTurn>();
        let state_team_assignment = ledger.read::<StateTeamAssignments>();

        let Some(cur_team) = state_team_assignment.team_for(id) else {
            panic!("");
        };

        // itereate to next
        let cur_team = cur_team.next_team();
        let bounds_min = GameBoard::get_bounds_min_for_team(&cur_team);
        let bounds_max = GameBoard::get_bounds_max_for_team(&cur_team);

        DataDepsFilled::Tiles(vec![Vector2Int::new(Random::range_int(bounds_min.x, bounds_max.x), Random::range_int(bounds_min.y, bounds_max.y))])
    }
    pub fn get_tiles_random_in_range_global(_id: &i32, _ledger: &Ledger, min: &Vector2Int, max: &Vector2Int) -> DataDepsFilled {
        // get between min and max in range
        let random_x = Random::range_int(min.x, max.x);
        let random_z = Random::range_int(min.y, max.y);

        DataDepsFilled::Tiles(vec![Vector2Int::new(random_x, random_z)])
    }
    pub fn get_tiles_random_in_range_local(id: &i32, ledger: &Ledger, min: &Vector2Int, max: &Vector2Int) -> DataDepsFilled {
        let state_modifiers = ledger.read::<StateCardAttributeModifierStack>();
        let _state_turn = ledger.read::<StateTurn>();
        let state_teams = ledger.read::<StateTeamAssignments>();
        let state_position_ball = ledger.read::<StatePositionBall>();

        // get the modifiers for stack
        let modifier_stack = state_modifiers.get_flat_stack_for_entity(*id);
        let team = state_teams.team_for(id).unwrap();

        // get the min and max taking into account any modifiers
        let min = team.convert_dir(min.x, min.y + modifier_stack.range);
        let max = team.convert_dir(max.x, max.y + modifier_stack.range);

        // get between min and max in range
        let random_x = Random::range_int(min.0, max.0);
        let random_z = Random::range_int(min.1, max.1);

        let col = state_position_ball.column + random_x;
        let row = state_position_ball.row + random_z;

        DataDepsFilled::Tiles(vec![Vector2Int::new(col, row)])
    }
}
