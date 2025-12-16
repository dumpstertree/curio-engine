use crate::cards::card_attribute_fillers::attribute_filler_player::CardAttributeFillerPlayer;
use crate::cards::card_dependencies::filled_card_attribute::FilledCardAttribute;
use crate::cards::card_dependencies::filled_card_response::FilledCardResponse;
use crate::cards::card_instance::CardInstance;
use crate::exploration::exploration_path::RoomTypes;
use crate::game_events::GameEvents;
use crate::state::host::state_exploration::StateExploration;
use crate::state::peer::state_peer_input_mode::{InputModes, StatePeerInputMode};
use crate::state::peer::state_peer_selected_card::StatePeerSelectedCards;
use crate::state::state_deck::CardTypes;
use crate::state::state_teams::StateTeamAssignments;
use crate::state::{state_deck::StateDeck, state_turn::StateTurn};
use built_in_state::state_input::InputState;
use core::dumpster_engine::NetworkModes;
use core::extensions::extensions_i32::ExtensionsI32;
use core::gameplay::world_context::WorldContext;
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
    fn is_enabled(&mut self, game_state: &mut GameState, _: &mut WorldContext) -> bool {
        // let is_turn = game_state.get::<StateTurn>().active_instance_id
        //     == game_state
        //         .get::<StateTeamAssignments>()
        //         .team_for(&game_state.instance_id)
        //         .unwrap();

        // is_turn &&
        game_state
            .get::<StateExploration>()
            .exploration
            .get_cur_room()
            .room_type
            == RoomTypes::Combat
            && game_state.get::<StatePeerInputMode>().mode == InputModes::Manuever
            && !game_state.get::<StateExploration>().is_selecting_next
    }
    fn tick(&mut self, game_state: &mut GameState, _: &mut WorldContext, event_queue: &mut EventQueue) {
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

        let state_team = game_state.get::<StateTeamAssignments>();
        let Some(_) = state_team.team_for(&game_state.instance_id) else {
            return;
        };
        // my deck
        let my_deck = &state_deck.deck[&game_state.instance_id];
        let my_cards_in_hand = my_deck.get_cards_from_hand(|x| x.get_manuever_type() != CardTypes::Move);

        // new bounds for looping
        let bounds_min = 0;
        let bounds_max = my_cards_in_hand.len() as i32;
        // let bounds_max = (my_deck.hand_persistent.len() + my_deck.hand_consumable.len()) as i32;

        // move left or right
        if input_card_left || input_card_right {
            // edit the selected cards
            game_state.edit::<StatePeerSelectedCards>(|x| {
                // move left
                if input_card_left {
                    x.index = (x.index - 1).repeat(bounds_min, bounds_max);
                }

                // move right
                if input_card_right {
                    x.index = (x.index + 1).repeat(bounds_min, bounds_max);
                }
            });
        }

        // edit the selected cards
        game_state.edit::<StatePeerSelectedCards>(|x| {
            // incase its out of bounds clamp it
            x.index = x.index.clamp(bounds_min, bounds_max);
        });

        // try to submit card
        if input_card_submit {
            let index = game_state.get::<StatePeerSelectedCards>().index;

            let is_met = my_cards_in_hand[index as usize].has_statement(&game_state, game_state.instance_id);
            if !is_met {
                println!("Requirements not met");
                return;
            }

            let mut evnt_filled = vec![];
            for evnt in &my_cards_in_hand[index as usize].get_attributes_events(&game_state, game_state.instance_id) {
                evnt_filled.push(FilledCardAttribute::new(CardAttributeFillerPlayer::fill_events(game_state, &game_state.instance_id, &evnt.get_data_dependencies_empty())));
            }
            let mut mod_filled = vec![];
            for evnt in &my_cards_in_hand[index as usize].get_attributes_modifiers(&game_state, game_state.instance_id) {
                mod_filled.push(FilledCardAttribute::new(CardAttributeFillerPlayer::fill_events(game_state, &game_state.instance_id, &evnt.get_data_dependencies_empty())));
            }

            event_queue.enqueue_event(GameEvents::RequestUseManeuverConsumable(game_state.instance_id, my_cards_in_hand[index as usize].instance_id, FilledCardResponse::new(mod_filled, evnt_filled)));
            // }
        }
    }
}
