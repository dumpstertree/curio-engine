use crate::{
    cards::card_instance::CardInstance,
    game_events::{FilledCardResponse, GameEvents},
    state::{state_deck::StateDeck, state_energy::StateEnergy, state_turn::StateTurn},
};
use core::{
    collections::{event_queue::EventQueue, game_state::GameState},
    dumpster_engine::NetworkModes,
    gameplay::ecs::traits::{ecs_event_reciever, ecs_system::ECSSystemEventless},
};
use ecs_event::global_ecs_system_event_reciever;
use ecs_system::global_ecs_system;
use hecs::World;
use std::sync::Arc;

#[global_ecs_system]
pub struct ECSSystemGameRequestManuever {}
impl ECSSystemEventless for ECSSystemGameRequestManuever {
    fn is_enabled(&mut self, _: &mut GameState, _: &mut World) -> bool {
        true
    }
    fn run_on_instance(&mut self, game_state: &mut GameState) -> Vec<NetworkModes> {
        vec![NetworkModes::LocalHost, NetworkModes::OnlineHost]
    }
}
#[global_ecs_system_event_reciever(GameEvents)]
impl ecs_event_reciever::EventReciever<GameEvents> for ECSSystemGameRequestManuever {
    fn dequeue_event(&mut self, game_state: &mut GameState, _: &mut World, event_queue: &mut EventQueue, event: &GameEvents) {
        match event {
            GameEvents::RequestUseManeuverPersistent(id, card_instance, data) => {
                // make sure the correct player is sending an event
                if !ECSSystemGameRequestManuever::check_player_id(game_state, *id) {
                    return;
                }
                // make sure the card they sent is a valid index
                // if !ECSSystemGameRequestManuever::check_card_index_persistent(game_state, *id, *card_instance) {
                //     return;
                // }

                // get state
                // let state_deck = &game_state.get_value2::<StateDeck>();

                // get the deck for play id
                // let Some(deck) = &state_deck.deck.get(id) else {
                //     return;
                // };

                // get the card instance
                // let card_instance = &deck.hand_persistent[*card_instance as usize];

                let state_deck = &game_state.get_value2::<StateDeck>();
                let Some(deck) = state_deck.deck.get(id) else {
                    return;
                };

                let card_instance = deck.get_instance(*card_instance);

                // play
                ECSSystemGameRequestManuever::try_play_card(game_state, event_queue, card_instance, data, id);
            }
            GameEvents::RequestUseManeuverConsumable(id, card_instance, data) => {
                // make sure the correct player is sending an event
                if !ECSSystemGameRequestManuever::check_player_id(game_state, *id) {
                    return;
                }
                // make sure the card they sent is a valid index
                // if !ECSSystemGameRequestManuever::check_card_index_consumable(game_state, *id, *card_index) {
                //     return;
                // }

                // let state_deck = &game_state.get_value2::<StateDeck>();

                // get the deck for play id
                // let Some(deck) = &state_deck.deck.get(id) else {
                //     return;
                // };

                // get the card instance
                // let card_instance = &deck.hand_consumable[*card_index as usize];

                let state_deck = &game_state.get_value2::<StateDeck>();
                let Some(deck) = state_deck.deck.get(id) else {
                    return;
                };

                let card_instance = deck.get_instance(*card_instance);
                // play
                ECSSystemGameRequestManuever::try_play_card(game_state, event_queue, card_instance, data, id);
            }

            _ => {}
        }
    }
}
impl ECSSystemGameRequestManuever {
    fn check_dependencies_match(card: &Arc<CardInstance>, response: &FilledCardResponse) -> bool {
        // get the full list of dependencies
        let deps_empty = card.get_attributes_events();
        let deps_filled = &response.event;

        // check that they have the same length
        if deps_empty.len() != deps_filled.len() {
            println!("Event dependency length mismatch");
            return false;
        }

        // get the count of one because they should be the same
        let count = deps_empty.len();

        // iterate over the range
        for i in 0..count {
            // get the two
            let dep_empty = &deps_empty[i].get_data_dependencies_empty();
            let dep_filled = &deps_filled[i];

            // check that they have the same length
            if dep_empty.len() != dep_filled.len() {
                println!("Event Dependency '{}' length mismatch:, {}, {}", i, dep_empty.len(), dep_filled.len());
                return false;
            }
            // get the count of one because they should be the same
            let count = dep_empty.len();

            // iterate over the range
            for i in 0..count {
                // check if the dependencies are aligned
                if !dep_filled[i].is_aligned(&dep_empty[i]) {
                    return false;
                }
            }
        }
        let deps_empty = card.get_attributes_modifiers();
        let deps_filled = &response.modifiers;
        let count = deps_empty.len();

        // check that they have the same length
        if deps_empty.len() != deps_filled.len() {
            println!("Modifier dependency length mismatch");
            return false;
        }

        // iterate over the range
        for i in 0..count {
            // get the two
            let dep_empty = &deps_empty[i].get_data_dependencies_empty();
            let dep_filled = &deps_filled[i];

            // check that they have the same length
            if dep_empty.len() != dep_filled.len() {
                println!("Modifier Dependency '{}' length mismatch:, {}, {}", i, dep_empty.len(), dep_filled.len());
                return false;
            }
            // get the count of one because they should be the same
            let count = dep_empty.len();

            // iterate over the range
            for i in 0..count {
                // check if the dependencies are aligned
                if !dep_filled[i].is_aligned(&dep_empty[i]) {
                    return false;
                }
            }
        }

        return true;
    }
    fn check_energy(game_state: &mut GameState, id: i32, cost: i32) -> bool {
        let has_energy = game_state.get_value2::<StateEnergy>().all_players[&id].0 - cost >= 0;
        if !has_energy {
            println!("Requested manuever for not enough energy cur: ({}) cost: ({})", game_state.get_value2::<StateEnergy>().all_players[&id].0, cost);
            return false;
        }

        return true;
    }
    fn check_player_id(game_state: &mut GameState, id: i32) -> bool {
        let is_active_player = game_state.get_value2::<StateTurn>().active_instance_id == id;
        if !is_active_player {
            println!("Requested for non-active player");
            return false;
        }

        return true;
    }
    fn check_card_index_persistent(game_state: &mut GameState, id: i32, card_index: i32) -> bool {
        let my_deck = &game_state.get_value2::<StateDeck>().deck[&id];
        let is_in_range = card_index < my_deck.hand_persistent.len() as i32;
        if !is_in_range {
            println!("Card out of bounds");
            return false;
        }

        return true;
    }
    fn check_card_index_consumable(game_state: &mut GameState, id: i32, card_index: i32) -> bool {
        let my_deck = &game_state.get_value2::<StateDeck>().deck[&id];
        let is_in_range = card_index < my_deck.hand_consumable.len() as i32;
        if !is_in_range {
            println!("Card out of bounds");
            return false;
        }

        return true;
    }
    fn try_play_card(game_state: &mut GameState, event_queue: &mut EventQueue, card_instance: Arc<CardInstance>, data: &FilledCardResponse, id: &i32) {
        // let library = CardLibrary::new();
        // let card = &library.get_card(&card_instance.card_id);

        if !ECSSystemGameRequestManuever::check_energy(game_state, *id, card_instance.get_cost()) {
            return;
        }
        if !ECSSystemGameRequestManuever::check_dependencies_match(&card_instance, data) {
            return;
        }

        // spend
        game_state.edit::<StateEnergy>(|x| {
            x.all_players
                .insert(*id, (x.all_players[id].0 - card_instance.get_cost(), x.all_players[id].1));
        });

        // consume
        game_state.edit::<StateDeck>(|x| {
            // get this deck
            let Some(deck) = x.deck.get_mut(id) else {
                return;
            };

            deck.discard(card_instance.clone());
        });

        // play the card
        event_queue.enqueue_event(GameEvents::PlayCard(*id, card_instance.clone(), data.clone()));
    }
}
