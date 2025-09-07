use crate::{
    game_events::GameEvents,
    state::{state_deck::StateDeck, state_energy::StateEnergy, state_position_player::StatePositionPlayer, state_turn::StateTurn},
};
use core::{
    collections::{event_queue::EventQueue, game_state::GameState},
    dumpster_engine::NetworkModes,
    gameplay::ecs::traits::{ecs_event_reciever, ecs_system::ECSSystemEventless},
};
use ecs_event::global_ecs_system_event_reciever;
use ecs_system::global_ecs_system;
use hecs::World;

#[global_ecs_system]
pub struct ECSSystemGameRequestManuever {}
impl ECSSystemGameRequestManuever {
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
}
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
            GameEvents::RequestUseManeuverPersistent(id, card_index) => {
                if !ECSSystemGameRequestManuever::check_player_id(game_state, *id) {
                    return;
                }
                if !ECSSystemGameRequestManuever::check_card_index_persistent(game_state, *id, *card_index) {
                    return;
                }

                let state_deck = &game_state.get_value2::<StateDeck>();
                let deck = &state_deck.deck[id];
                let card = &deck.hand_persistent[*card_index as usize];

                if !ECSSystemGameRequestManuever::check_energy(game_state, *id, card.cost) {
                    return;
                }

                // spend
                game_state.edit::<StateEnergy>(|x| {
                    x.all_players
                        .insert(*id, (x.all_players[id].0 - card.cost, x.all_players[id].1));
                });

                // play the card
                event_queue.enqueue_event(GameEvents::PlayCard(*id, card.clone()));
            }
            GameEvents::RequestUseManeuverConsumable(id, card_index) => {
                if !ECSSystemGameRequestManuever::check_player_id(game_state, *id) {
                    return;
                }
                if !ECSSystemGameRequestManuever::check_card_index_consumable(game_state, *id, *card_index) {
                    return;
                }

                let state_deck = &game_state.get_value2::<StateDeck>();
                let deck = &state_deck.deck[id];
                let card = &deck.hand_consumable[*card_index as usize];

                if !ECSSystemGameRequestManuever::check_energy(game_state, *id, card.cost) {
                    return;
                }

                // spend
                game_state.edit::<StateEnergy>(|x| {
                    x.all_players
                        .insert(*id, (x.all_players[id].0 - card.cost, x.all_players[id].1));
                });

                // consume
                game_state.edit::<StateDeck>(|x| {
                    let Some(deck) = x.deck.get_mut(id) else {
                        return;
                    };
                    deck.hand_consumable.remove(*card_index as usize);
                });

                // play the card
                event_queue.enqueue_event(GameEvents::PlayCard(*id, card.clone()));
            }

            _ => {}
        }
    }
}
