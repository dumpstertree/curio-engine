use crate::{
    card_parser::{DataDepsEmpty, DataDepsFilled},
    game_events::GameEvents,
    state::{
        state_deck::{CardLibrary, StateDeck},
        state_energy::StateEnergy,
        state_position_player::StatePositionPlayer,
        state_turn::StateTurn,
    },
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
    fn dependencies_match(empty: Vec<DataDepsEmpty>, filled: Vec<DataDepsFilled>) -> bool {
        if empty.len() != filled.len() {
            println!("incorrect lengths :, {}, {}", empty.len(), filled.len());
            return false;
        }

        for i in 0..empty.len() {
            let a = &empty[i];
            let b = &filled[i];

            match a {
                DataDepsEmpty::Players(target_types_players) => println!(" a player"),
                DataDepsEmpty::Entities(target_types_entities) => println!(" a entity"),
                DataDepsEmpty::Cards(target_types_cards) => println!(" a card"),
                DataDepsEmpty::Tiles(target_types_tiles) => println!(" a tile"),
            }
            match b {
                DataDepsFilled::Players(target_types_players) => println!(" b player"),
                DataDepsFilled::Entities(target_types_entities) => println!(" b entity"),
                DataDepsFilled::Cards(target_types_cards) => println!(" b card"),
                DataDepsFilled::Tiles(target_types_tiles) => println!(" b tile"),
            }
            match a {
                DataDepsEmpty::Entities(_) => match b {
                    DataDepsFilled::Entities(_) => continue,
                    _ => return false,
                },
                DataDepsEmpty::Cards(_) => match b {
                    DataDepsFilled::Cards(_) => continue,
                    _ => return false,
                },
                DataDepsEmpty::Tiles(_) => match b {
                    DataDepsFilled::Tiles(_) => continue,
                    _ => return false,
                },
                DataDepsEmpty::Players(_) => match b {
                    DataDepsFilled::Players(_) => continue,
                    _ => return false,
                },
            };
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
            GameEvents::RequestUseManeuverPersistent(id, card_index, data) => {
                if !ECSSystemGameRequestManuever::check_player_id(game_state, *id) {
                    return;
                }
                if !ECSSystemGameRequestManuever::check_card_index_persistent(game_state, *id, *card_index) {
                    return;
                }

                let state_deck = &game_state.get_value2::<StateDeck>();
                let deck = &state_deck.deck[id];
                let card = &deck.hand_persistent[*card_index as usize];
                let library = CardLibrary::new();
                let card = library.get_card(&card.card_id);

                if !ECSSystemGameRequestManuever::check_energy(game_state, *id, card.cost) {
                    return;
                }

                // pull card from library
                let card_events = &card.get_events();
                if card_events.len() != data.event.len() {
                    println!("incorrecte attributes and data events{}, {}", card_events.len(), data.event.len());
                    return;
                }
                for i in 0..card_events.len() {
                    let a = card.get_events()[i].get_data_dependencies();
                    let b: Vec<DataDepsFilled> = data.event[i].clone();
                    let do_match = ECSSystemGameRequestManuever::dependencies_match(a.clone(), b.clone());
                    if !do_match {
                        println!("incorrecte attributes and data events at index {} : {}, {}", i, a.len(), b.len());
                        return;
                    }
                }

                // spend
                game_state.edit::<StateEnergy>(|x| {
                    x.all_players
                        .insert(*id, (x.all_players[id].0 - card.cost, x.all_players[id].1));
                });

                // play the card
                event_queue.enqueue_event(GameEvents::PlayCard(*id, card.title.clone(), data.clone()));
            }
            GameEvents::RequestUseManeuverConsumable(id, card_index, data) => {
                if !ECSSystemGameRequestManuever::check_player_id(game_state, *id) {
                    return;
                }
                if !ECSSystemGameRequestManuever::check_card_index_consumable(game_state, *id, *card_index) {
                    return;
                }

                let state_deck = &game_state.get_value2::<StateDeck>();
                let deck = &state_deck.deck[id];
                let card = &deck.hand_consumable[*card_index as usize];
                let library = CardLibrary::new();
                let card = library.get_card(&card.card_id);

                if !ECSSystemGameRequestManuever::check_energy(game_state, *id, card.cost) {
                    return;
                }

                // pull card from library
                let card_events = &card.get_events();
                if card_events.len() != data.event.len() {
                    println!("incorrecte attributes and data events{}, {}", card_events.len(), data.event.len());
                    return;
                }
                for i in 0..card_events.len() {
                    let a = card.get_events()[i].get_data_dependencies();
                    let b: Vec<DataDepsFilled> = data.event[i].clone();
                    let do_match = ECSSystemGameRequestManuever::dependencies_match(a.clone(), b.clone());
                    if !do_match {
                        println!("incorrecte attributes and data events at index {} : {}, {}", i, a.len(), b.len());
                        return;
                    }
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
                event_queue.enqueue_event(GameEvents::PlayCard(*id, card.title.clone(), data.clone()));
            }

            _ => {}
        }
    }
}
