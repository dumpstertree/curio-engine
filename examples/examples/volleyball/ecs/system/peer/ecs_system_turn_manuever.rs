use crate::cards::card_instance::CardInstance;
use crate::dependency_filler::DependencyFiller;
use crate::game_events::{FilledAttribute, FilledCardResponse, GameEvents};
use crate::state::peer::state_peer_input_mode::{InputModes, StatePeerInputMode};
use crate::state::peer::state_peer_selected_card::StatePeerSelectedCards;
use crate::state::{state_deck::StateDeck, state_turn::StateTurn};
use built_in_state::state_input::InputState;
use core::dumpster_engine::NetworkModes;
use core::extensions::extensions_i32::ExtensionsI32;
use core::{
    collections::{event_queue::EventQueue, game_state::GameState},
    gameplay::ecs::traits::ecs_system::ECSSystemEventless,
};
use ecs_system::global_ecs_system;
use hecs::World;
use std::sync::Arc;

#[global_ecs_system]
pub struct ECSSystemTurnManuever {
    // card_index: i32,
}
impl ECSSystemTurnManuever {}
impl ECSSystemEventless for ECSSystemTurnManuever {
    fn run_on_instance(&mut self, game_state: &mut GameState) -> Vec<core::dumpster_engine::NetworkModes> {
        vec![NetworkModes::LocalPeer, NetworkModes::OnlinePeer]
    }
    fn is_enabled(&mut self, game_state: &mut GameState, _: &mut World) -> bool {
        game_state.get::<StateTurn>().active_instance_id == game_state.instance_id && game_state.get::<StatePeerInputMode>().mode == InputModes::Manuever
    }
    fn tick(&mut self, game_state: &mut GameState, _: &mut World, event_queue: &mut EventQueue) {
        let state_input = game_state.get::<InputState>();
        let state_deck = game_state.get::<StateDeck>();

        let input_card_left = state_input.mapped[0]
            .get_button_or_default("card_left")
            .went_up;
        let input_card_right = state_input.mapped[0]
            .get_button_or_default("card_right")
            .went_up;
        let input_card_submit = state_input.mapped[0]
            .get_button_or_default("card_submit")
            .went_up;

        // my deck
        let my_deck = &state_deck.deck[&game_state.instance_id];

        // new bounds for looping
        let bounds_min = 0;
        let bounds_max = (my_deck.hand_persistent.len() + my_deck.hand_consumable.len()) as i32;

        // move left or right
        if input_card_left || input_card_right {
            // edit the selected cards
            game_state.edit::<StatePeerSelectedCards>(|x| {
                // clamp any old value
                x.index = x.index.clamp(bounds_min, bounds_max);

                // generate the list of cards using
                let mut list: Vec<Arc<CardInstance>> = vec![];
                for x in my_deck.hand_persistent.clone() {
                    list.push(x);
                }
                for x in my_deck.hand_consumable.clone() {
                    list.push(x);
                }

                // move left
                if input_card_left {
                    x.index = (x.index - 1).repeat(bounds_min, bounds_max);
                    println!("card leeft -> change card : {}", list[x.index as usize].get_title());
                }

                // move right
                if input_card_right {
                    x.index = (x.index + 1).repeat(bounds_min, bounds_max);
                    println!("card right -> change card : {}", list[x.index as usize].get_title());
                }
            });
        }

        // try to submit card
        if input_card_submit {
            // edit the selected cards
            game_state.edit::<StatePeerSelectedCards>(|x| {
                // incase its out of bounds clamp it
                x.index = x.index.clamp(bounds_min, bounds_max);
            });

            //
            let persistent_len = my_deck.hand_persistent.len() as i32;
            let index = game_state.get::<StatePeerSelectedCards>().index;
            let is_persistent = index < persistent_len;
            if is_persistent {
                let index = game_state.get::<StatePeerSelectedCards>().index;
                // generate the list of cards using
                let mut list0 = vec![];
                for x in my_deck.hand_persistent.clone() {
                    list0.push(x);
                }
                for x in my_deck.hand_consumable.clone() {
                    list0.push(x);
                }

                let is_met = list0[index as usize].has_statement(&game_state, game_state.instance_id);
                if !is_met {
                    println!("Requirements not met");
                    return;
                }

                let mut evnt_filled = vec![];
                for evnt in &list0[index as usize].get_attributes_events(game_state, game_state.instance_id) {
                    evnt_filled.push(FilledAttribute::new(DependencyFiller::fill_events(game_state, &&evnt.get_data_dependencies_empty())));
                }
                let mut mod_filled = vec![];
                for evnt in &list0[index as usize].get_attributes_modifiers(game_state, game_state.instance_id) {
                    mod_filled.push(FilledAttribute::new(DependencyFiller::fill_events(game_state, &evnt.get_data_dependencies_empty())));
                }

                event_queue.enqueue_event(GameEvents::RequestUseManeuverPersistent(game_state.instance_id, list0[index as usize].instance_id, FilledCardResponse::new(mod_filled, evnt_filled)));
            } else {
                let index = game_state.get::<StatePeerSelectedCards>().index;
                // generate the list of cards using
                let mut list0 = vec![];
                for x in my_deck.hand_persistent.clone() {
                    list0.push(x);
                }
                for x in my_deck.hand_consumable.clone() {
                    list0.push(x);
                }

                let is_met = list0[index as usize].has_statement(&game_state, game_state.instance_id);
                if !is_met {
                    println!("Requirements not met");
                    return;
                }
                if !is_met {
                    println!("Requirements not met");
                    return;
                }

                let mut evnt_filled = vec![];
                for evnt in &list0[index as usize].get_attributes_events(&game_state, game_state.instance_id) {
                    evnt_filled.push(FilledAttribute::new(DependencyFiller::fill_events(game_state, &evnt.get_data_dependencies_empty())));
                }
                let mut mod_filled = vec![];
                for evnt in &list0[index as usize].get_attributes_modifiers(&game_state, game_state.instance_id) {
                    mod_filled.push(FilledAttribute::new(DependencyFiller::fill_events(game_state, &evnt.get_data_dependencies_empty())));
                }

                event_queue.enqueue_event(GameEvents::RequestUseManeuverConsumable(game_state.instance_id, list0[index as usize].instance_id, FilledCardResponse::new(mod_filled, evnt_filled)));
            }
        }
    }
}
