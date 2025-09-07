use built_in_state::state_input::InputState;
use ecs_system::global_ecs_system;
use hecs::World;

use core::dumpster_engine::NetworkModes;
use core::extensions::extensions_i32::ExtensionsI32;
use core::{
    collections::{event_queue::EventQueue, game_state::GameState},
    gameplay::ecs::traits::ecs_system::ECSSystemEventless,
};

use crate::game_events::GameEvents;
use crate::state::state_deck::Card;
use crate::state::{state_deck::StateDeck, state_turn::StateTurn};

#[global_ecs_system]
pub struct ECSSystemTurnManuever {
    card_index: i32,
}
impl ECSSystemEventless for ECSSystemTurnManuever {
    fn run_on_instance(&mut self, game_state: &mut GameState) -> Vec<core::dumpster_engine::NetworkModes> {
        vec![NetworkModes::LocalPeer, NetworkModes::OnlinePeer]
    }
    fn is_enabled(&mut self, game_state: &mut GameState, _: &mut World) -> bool {
        game_state.get_value2::<StateTurn>().active_instance_id == game_state.instance_id
    }
    fn tick(&mut self, game_state: &mut GameState, _: &mut World, event_queue: &mut EventQueue) {
        let state_input = game_state.get_value2::<InputState>();
        let state_deck = game_state.get_value2::<StateDeck>();

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

        // incase its out of bounds clamp it
        self.card_index = self.card_index.clamp(bounds_min, bounds_max);

        // move left or right
        if input_card_left || input_card_right {
            // generate the list of cards using
            let mut list: Vec<&Card> = vec![];
            list.extend(&my_deck.hand_persistent);
            list.extend(&my_deck.hand_consumable);

            // move left
            if input_card_left {
                self.card_index = (self.card_index - 1).repeat(bounds_min, bounds_max);
                println!("card leeft -> change card : {}", list[self.card_index as usize].title);
            }

            // move right
            if input_card_right {
                self.card_index = (self.card_index + 1).repeat(bounds_min, bounds_max);
                println!("card right -> change card : {}", list[self.card_index as usize].title);
            }
        }

        // try to submit card
        if input_card_submit {
            //
            let persistent_len = my_deck.hand_persistent.len() as i32;
            let is_persistent = self.card_index < persistent_len;
            if is_persistent {
                event_queue.enqueue_event(GameEvents::RequestUseManeuverPersistent(game_state.instance_id, self.card_index));
            } else {
                event_queue.enqueue_event(GameEvents::RequestUseManeuverConsumable(game_state.instance_id, self.card_index - persistent_len));
            }
        }
    }
}
