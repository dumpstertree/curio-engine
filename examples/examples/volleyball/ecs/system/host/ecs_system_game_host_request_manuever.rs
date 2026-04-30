use crate::{
    cards::{card_dependencies::filled_card_response::FilledCardResponse, card_instance::CardInstance},
    game_events::GameEvents,
    state::{state_deck::StateDeck, state_energy::StateEnergy, state_teams::StateTeamAssignments, state_turn::StateTurn},
};
use curio_core::{
    collections::{event_queue::EventQueue, ledger::Ledger},
    network_modes::NetworkModes,
};
use gameplay::{
    context_3d::Context3D,
    traits::{impulse::Impulse, scope::Scope},
};
use impulse::impulse;
use std::sync::Arc;

#[derive(Default)]
#[impulse(GameEvents)]
pub struct ECsystemGameRequestManuever {}
impl Scope for ECsystemGameRequestManuever {
    fn is_enabled(&mut self, _: &mut Ledger) -> bool {
        true
    }
    fn run_on_instance(&mut self, _: &mut Ledger) -> Vec<NetworkModes> {
        vec![NetworkModes::LocalHost, NetworkModes::OnlineHost]
    }
}
impl Impulse<GameEvents> for ECsystemGameRequestManuever {
    fn dequeue_event(&mut self, ledger: &mut Ledger, _: &mut Context3D, event_queue: &mut EventQueue, event: &GameEvents) {
        match event {
            GameEvents::RequestUseManeuverPersistent(id, card_instance, data) => {
                // make sure the correct player is sending an event
                println!("----------------------------use manuever!!! for {} with id {}", ledger.name, id);

                if !ECsystemGameRequestManuever::check_player_id(ledger, *id) {
                    println!("mismatch player id");
                    return;
                }
                // make sure the card they sent is a valid index
                // if !ECsystemGameRequestManuever::check_card_index_persistent(ledger, *id, *card_instance) {
                //     return;
                // }

                // get state
                // let state_deck = &ledger.get_value2::<StateDeck>();

                // get the deck for play id
                // let Some(deck) = &state_deck.deck.get(id) else {
                //     return;
                // };

                // get the card instance
                // let card_instance = &deck.hand_persistent[*card_instance as usize];

                let state_deck = &ledger.read::<StateDeck>();
                let Some(deck) = state_deck.deck.get(id) else {
                    println!("card not found in deck");
                    return;
                };

                let card_instance = deck.get_instance(*card_instance);

                // play
                ECsystemGameRequestManuever::try_play_card(ledger, event_queue, card_instance, data, id);
                println!("-----------------------------------did use manuever!!! for {} with id {}", ledger.name, id);
            }
            GameEvents::RequestUseManeuverConsumable(id, card_instance, data) => {
                // make sure the correct player is sending an event

                println!("----------------------------use manuever!!!");

                if !ECsystemGameRequestManuever::check_player_id(ledger, *id) {
                    println!("mismatch player id");
                    return;
                }
                // make sure the card they sent is a valid index
                // if !ECsystemGameRequestManuever::check_card_index_consumable(ledger, *id, *card_index) {
                //     return;
                // }

                // let state_deck = &ledger.get_value2::<StateDeck>();

                // get the deck for play id
                // let Some(deck) = &state_deck.deck.get(id) else {
                //     return;
                // };

                // get the card instance
                // let card_instance = &deck.hand_consumable[*card_index as usize];

                let state_deck = &ledger.read::<StateDeck>();
                let Some(deck) = state_deck.deck.get(id) else {
                    println!("card not found in deck");
                    return;
                };
                println!("-----------------------------------did use manuever!!! for {}", ledger.name);

                let card_instance = deck.get_instance(*card_instance);
                // play
                ECsystemGameRequestManuever::try_play_card(ledger, event_queue, card_instance, data, id);
            }

            _ => {}
        }
    }
}
impl ECsystemGameRequestManuever {
    fn check_dependencies_match(card: &Arc<CardInstance>, response: &FilledCardResponse, ledger: &Ledger, id: &i32) -> bool {
        // get the full list of dependencies
        let deps_empty = card.get_attributes_events(ledger, *id);
        let deps_filled = &response.event;

        // check that they have the same length
        if deps_empty.len() != deps_filled.len() {
            println!("Event dependency length mismatch {} to {}", deps_empty.len(), deps_filled.len());
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
            if dep_empty.len() != dep_filled.filled.len() {
                println!("Event Dependency '{}' length mismatch:, {}, {}", i, dep_empty.len(), dep_filled.filled.len());
                return false;
            }
            // get the count of one because they should be the same
            let count = dep_empty.len();

            // iterate over the range
            for i in 0..count {
                // check if the dependencies are aligned
                if !dep_filled.filled[i].is_aligned(&dep_empty[i]) {
                    return false;
                }
            }
        }
        let deps_empty = card.get_attributes_modifiers(ledger, *id);
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
            if dep_empty.len() != dep_filled.filled.len() {
                println!("Modifier Dependency '{}' length mismatch:, {}, {}", i, dep_empty.len(), dep_filled.filled.len());
                return false;
            }
            // get the count of one because they should be the same
            let count = dep_empty.len();

            // iterate over the range
            for i in 0..count {
                // check if the dependencies are aligned
                if !dep_filled.filled[i].is_aligned(&dep_empty[i]) {
                    return false;
                }
            }
        }

        return true;
    }
    fn check_energy(ledger: &mut Ledger, id: i32, cost: i32) -> bool {
        let has_energy = ledger.read::<StateEnergy>().all_players[&id].0 - cost >= 0;
        if !has_energy {
            println!("Requested manuever for not enough energy cur: ({}) cost: ({})", ledger.read::<StateEnergy>().all_players[&id].0, cost);
            return false;
        }

        return true;
    }
    fn check_player_id(ledger: &mut Ledger, id: i32) -> bool {
        let state_teams = ledger.read::<StateTeamAssignments>().team_for(&id).unwrap();
        let is_active_player = ledger.read::<StateTurn>().active_instance_id == state_teams;
        if !is_active_player {
            println!("Requested for non-active player");
            return false;
        }

        return true;
    }
    // fn check_card_index_persistent(ledger: &mut GameState, id: i32, card_index: i32) -> bool {
    //     let my_deck = &ledger.get::<StateDeck>().deck[&id];
    //     let is_in_range = card_index < my_deck.hand_persistent.len() as i32;
    //     if !is_in_range {
    //         println!("Card out of bounds");
    //         return false;
    //     }

    //     return true;
    // }
    fn check_card_index_consumable(ledger: &mut Ledger, id: i32, card_index: i32) -> bool {
        let my_deck = &ledger.read::<StateDeck>().deck[&id];
        let is_in_range = card_index < my_deck.hand_consumable.len() as i32;
        if !is_in_range {
            println!("Card out of bounds");
            return false;
        }

        return true;
    }
    fn try_play_card(ledger: &mut Ledger, event_queue: &mut EventQueue, card_instance: Arc<CardInstance>, data: &FilledCardResponse, id: &i32) {
        // let library = CardLibrary::new();
        // let card = &library.get_card(&card_instance.card_id);

        let cost = card_instance.get_cost(&ledger, *id);
        if !ECsystemGameRequestManuever::check_energy(ledger, *id, cost) {
            return;
        }
        if !ECsystemGameRequestManuever::check_dependencies_match(&card_instance, data, &ledger, id) {
            return;
        }

        if !card_instance
            .get_statement(ledger, *id)
            .requirements
            .iter()
            .all(|x| x.is_met(ledger, *id))
        {
            return;
        }
        // spend
        ledger.write::<StateEnergy>(|x| {
            x.all_players
                .insert(*id, (x.all_players[id].0 - cost, x.all_players[id].1));
        });

        // consume
        ledger.write::<StateDeck>(|x| {
            // get this deck
            let Some(deck) = x.deck.get_mut(id) else {
                return;
            };

            deck.play(card_instance.clone());
        });

        // play the card
        event_queue.enqueue_event(GameEvents::PlayCard(*id, card_instance.clone(), data.clone()));
    }
}
