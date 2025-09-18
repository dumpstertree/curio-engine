use core::{
    collections::{game_state::GameState, vector2_int::Vector2Int},
    random::Random,
};

use built_in_state::state_network::StateNetwork;

use crate::{
    cards::{
        attribute_target_type_cards::AttributeTargetTypesCards, attribute_target_type_entities::AttribtuteTargetTypesEntities, attribute_target_type_players::AtrributeTargetTypesPlayers, attribute_target_type_tiles::AttributeTargetTypesTiles, data_dep_empty::DataDepsEmpty,
        data_dep_filled::DataDepsFilled,
    },
    game_board::GameBoard,
    state::{state_deck::StateDeck, state_teams::StateTeamAssignments, state_turn::StateTurn},
};

pub struct DependencyFiller {}

impl DependencyFiller {
    pub fn fill_events(game_state: &GameState, data_dep_empty: &Vec<DataDepsEmpty>) -> Vec<DataDepsFilled> {
        let mut filled = vec![];
        for dep in data_dep_empty {
            match dep {
                DataDepsEmpty::Players(target_types_players) => match target_types_players {
                    AtrributeTargetTypesPlayers::User => filled.push(DependencyFiller::get_player_user(game_state)),
                    AtrributeTargetTypesPlayers::Select => filled.push(DependencyFiller::get_player_select(game_state)),
                    AtrributeTargetTypesPlayers::Random => filled.push(DependencyFiller::get_player_random(game_state)),
                    AtrributeTargetTypesPlayers::Opponent => filled.push(DependencyFiller::get_player_opponent(game_state)),
                },
                DataDepsEmpty::Entities(target_types_entities) => match target_types_entities {
                    AttribtuteTargetTypesEntities::User => filled.push(DependencyFiller::get_entity_user(game_state)),
                    AttribtuteTargetTypesEntities::Select => filled.push(DependencyFiller::get_entity_select(game_state)),
                    AttribtuteTargetTypesEntities::RandomAny => filled.push(DependencyFiller::get_entity_random(game_state)),
                    AttribtuteTargetTypesEntities::RandomOpponent => filled.push(DependencyFiller::get_entity_opponent(game_state)),
                },
                DataDepsEmpty::Cards(target_types_cards) => match target_types_cards {
                    AttributeTargetTypesCards::SelectUser => filled.push(DependencyFiller::get_card_user_selected(game_state)),
                    AttributeTargetTypesCards::RandomUser => filled.push(DependencyFiller::get_card_user_random(game_state)),
                    AttributeTargetTypesCards::SelectOpponent => filled.push(DependencyFiller::get_card_opponent_selected(game_state)),
                    AttributeTargetTypesCards::RandomOpponent => filled.push(DependencyFiller::get_card_opponent_random(game_state)),
                    AttributeTargetTypesCards::AllUser => filled.push(DependencyFiller::get_card_user_all(game_state)),
                    AttributeTargetTypesCards::AllOpponent => filled.push(DependencyFiller::get_card_opponent_all(game_state)),
                },
                DataDepsEmpty::Tiles(target_types_tiles) => match target_types_tiles {
                    AttributeTargetTypesTiles::Select => filled.push(DependencyFiller::get_tiles_select(game_state)),
                    AttributeTargetTypesTiles::RandomAny => filled.push(DependencyFiller::get_tiles_random_any(game_state)),
                    AttributeTargetTypesTiles::RandomOnTeamUser => filled.push(DependencyFiller::get_tiles_random_on_team_user(game_state)),
                    AttributeTargetTypesTiles::RandomOnTeamOpponent => filled.push(DependencyFiller::get_tiles_random_on_team_opponent(game_state)),
                },
            }
        }
        filled
    }
}
// get -> player
impl DependencyFiller {
    fn get_player_user(game_state: &GameState) -> DataDepsFilled {
        let cur_player_id = game_state.get_value2::<StateTurn>().active_instance_id;
        DataDepsFilled::Players(vec![cur_player_id])
    }
    fn get_player_select(game_state: &GameState) -> DataDepsFilled {
        todo!()
    }
    fn get_player_random(game_state: &GameState) -> DataDepsFilled {
        let state_peers = game_state.get_value2::<StateNetwork>();
        let peer_instance_ids = state_peers.peer_instance_ids();
        let random_index = Random::range_int(0, peer_instance_ids.len().try_into().unwrap());
        let selected = peer_instance_ids.get(random_index as usize).unwrap();
        DataDepsFilled::Players(vec![*selected])
    }
    fn get_player_opponent(game_state: &GameState) -> DataDepsFilled {
        let state_turn = game_state.get_value2::<StateTurn>();
        let state_team_assignment = game_state.get_value2::<StateTeamAssignments>();
        let opponent_team = state_team_assignment
            .team_for(&state_turn.active_instance_id)
            .unwrap()
            .next_team();
        let opponent_ids = state_team_assignment
            .team_assignments
            .get(&opponent_team)
            .unwrap();

        DataDepsFilled::Players(vec![*opponent_ids.first().unwrap()])
    }
}
// get -> entity
impl DependencyFiller {
    fn get_entity_user(game_state: &GameState) -> DataDepsFilled {
        let cur_player_id = game_state.get_value2::<StateTurn>().active_instance_id;
        DataDepsFilled::Entities(vec![cur_player_id])
    }
    fn get_entity_select(game_state: &GameState) -> DataDepsFilled {
        todo!()
    }
    fn get_entity_random(game_state: &GameState) -> DataDepsFilled {
        let state_peers = game_state.get_value2::<StateNetwork>();
        let peer_instance_ids = state_peers.peer_instance_ids();
        let random_index = Random::range_int(0, peer_instance_ids.len().try_into().unwrap());
        let selected = peer_instance_ids.get(random_index as usize).unwrap();
        DataDepsFilled::Entities(vec![*selected])
    }
    fn get_entity_opponent(game_state: &GameState) -> DataDepsFilled {
        let state_turn = game_state.get_value2::<StateTurn>();
        let state_team_assignment = game_state.get_value2::<StateTeamAssignments>();
        let opponent_team = state_team_assignment
            .team_for(&state_turn.active_instance_id)
            .unwrap()
            .next_team();
        let opponent_ids = state_team_assignment
            .team_assignments
            .get(&opponent_team)
            .unwrap();

        DataDepsFilled::Entities(vec![*opponent_ids.first().unwrap()])
    }
}
// get -> cards
impl DependencyFiller {
    fn get_card_user_selected(game_state: &GameState) -> DataDepsFilled {
        todo!()
    }
    fn get_card_user_random(game_state: &GameState) -> DataDepsFilled {
        let state_turn = game_state.get_value2::<StateTurn>();
        let state_deck = game_state.get_value2::<StateDeck>();

        let playere_id = &state_turn.active_instance_id;
        let Some(deck) = state_deck.deck.get(playere_id) else {
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
    fn get_card_opponent_selected(game_state: &GameState) -> DataDepsFilled {
        todo!()
    }
    fn get_card_user_all(game_state: &GameState) -> DataDepsFilled {
        let state_turn = game_state.get_value2::<StateTurn>();
        let state_deck = game_state.get_value2::<StateDeck>();

        let Some(deck) = state_deck.deck.get(&state_turn.active_instance_id) else {
            panic!("");
        };

        let mut result = vec![];
        for x in &deck.hand_consumable {
            result.push(x.instance_id);
        }

        DataDepsFilled::Cards(result)
    }
    fn get_card_opponent_all(game_state: &GameState) -> DataDepsFilled {
        let state_turn = game_state.get_value2::<StateTurn>();
        let state_deck = game_state.get_value2::<StateDeck>();
        let state_team_assignment = game_state.get_value2::<StateTeamAssignments>();
        let opponent_team = state_team_assignment
            .team_for(&state_turn.active_instance_id)
            .unwrap()
            .next_team();
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
    fn get_card_opponent_random(game_state: &GameState) -> DataDepsFilled {
        let state_turn = game_state.get_value2::<StateTurn>();
        let state_deck = game_state.get_value2::<StateDeck>();
        let state_team_assignment = game_state.get_value2::<StateTeamAssignments>();
        let opponent_team = state_team_assignment
            .team_for(&state_turn.active_instance_id)
            .unwrap()
            .next_team();
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
impl DependencyFiller {
    pub fn get_tiles_select(game_state: &GameState) -> DataDepsFilled {
        todo!()
    }
    pub fn get_tiles_random_any(game_state: &GameState) -> DataDepsFilled {
        DataDepsFilled::Tiles(vec![Vector2Int::new(Random::range_int(0, 4), Random::range_int(0, 4))])
    }
    pub fn get_tiles_random_on_team_user(game_state: &GameState) -> DataDepsFilled {
        let state_turn = game_state.get_value2::<StateTurn>();
        let state_team_assignment = game_state.get_value2::<StateTeamAssignments>();

        let Some(cur_team) = state_team_assignment.team_for(&state_turn.active_instance_id) else {
            panic!("");
        };

        let bounds_min = GameBoard::get_bounds_min(&cur_team);
        let bounds_max = GameBoard::get_bounds_max(&cur_team);

        DataDepsFilled::Tiles(vec![Vector2Int::new(Random::range_int(bounds_min.x, bounds_max.x), Random::range_int(bounds_min.y, bounds_max.y))])
    }
    pub fn get_tiles_random_on_team_opponent(game_state: &GameState) -> DataDepsFilled {
        let state_turn = game_state.get_value2::<StateTurn>();
        let state_team_assignment = game_state.get_value2::<StateTeamAssignments>();

        let Some(cur_team) = state_team_assignment.team_for(&state_turn.active_instance_id) else {
            panic!("");
        };

        // itereate to next
        let cur_team = cur_team.next_team();
        let bounds_min = GameBoard::get_bounds_min(&cur_team);
        let bounds_max = GameBoard::get_bounds_max(&cur_team);

        DataDepsFilled::Tiles(vec![Vector2Int::new(Random::range_int(bounds_min.x, bounds_max.x), Random::range_int(bounds_min.y, bounds_max.y))])
    }
}
