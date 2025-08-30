use built_in_state::state_input::InputState;
use ecs_system::global_ecs_system;
use hecs::World;

use core::extensions::extensions_i32::ExtensionsI32;
use core::{
    collections::{event_queue::EventQueue, game_state::GameState},
    gameplay::ecs::traits::ecs_system::ECSSystemEventless,
};

use crate::state::state_ball_mode::{BallModes, StateBallMode};
use crate::state::state_deck::Card;
use crate::state::state_energy::StateEnergy;
use crate::state::state_position_ball::StatePositionBall;
use crate::state::state_position_player::StatePositionPlayer;
use crate::state::{state_deck::StateDeck, state_turn::StateTurn};

#[global_ecs_system]
pub struct ECSSystemTurnManuever {
    card_index: i32,
}
impl ECSSystemEventless for ECSSystemTurnManuever {
    fn is_enabled(&mut self, game_state: &mut GameState, _: &mut World) -> bool {
        game_state.get_value2::<StateTurn>().active_instance_id == game_state.instance_id
    }
    fn tick(&mut self, game_state: &mut GameState, _: &mut World, _: &mut EventQueue) {
        let state_input = game_state.get_value2::<InputState>();
        let state_deck = game_state.get_value2::<StateDeck>();
        let state_position_ball = game_state.get_value2::<StatePositionBall>();
        let state_position_player = game_state.get_value2::<StatePositionPlayer>();
        let state_energy = game_state.get_value2::<StateEnergy>();

        let input_card_left = state_input.mapped[0]
            .get_button_or_default("card_left")
            .went_up;
        let input_card_right = state_input.mapped[0]
            .get_button_or_default("card_right")
            .went_up;
        let input_card_submit = state_input.mapped[0]
            .get_button_or_default("card_submit")
            .went_up;

        // move left
        if input_card_left {
            let bounds_min = 0;
            let bounds_max = (state_deck.deck.hand_persistent.len() + state_deck.deck.hand_consumable.len()) as i32;

            self.card_index = (self.card_index - 1).repeat(bounds_min, bounds_max);

            let mut list: Vec<&Card> = vec![];
            list.extend(&state_deck.deck.hand_persistent);
            list.extend(&state_deck.deck.hand_consumable);

            println!("card leeft -> change card : {}", list[self.card_index as usize].title);
        }

        // move right
        if input_card_right {
            let bounds_min = 0;
            let bounds_max = (state_deck.deck.hand_persistent.len() + state_deck.deck.hand_consumable.len()) as i32;

            self.card_index = (self.card_index + 1).repeat(bounds_min, bounds_max);

            let mut list: Vec<&Card> = vec![];
            list.extend(&state_deck.deck.hand_persistent);
            list.extend(&state_deck.deck.hand_consumable);

            println!("card right -> change card : {}", list[self.card_index as usize].title);
        }

        // try to submit card
        if input_card_submit {
            let mut list: Vec<&Card> = vec![];
            list.extend(&state_deck.deck.hand_persistent);
            list.extend(&state_deck.deck.hand_consumable);
            let card = list[self.card_index as usize];

            let has_energy = state_energy.cur_energy >= card.cost;
            if !has_energy {
                println!("Not enough Energy");
                return;
            }

            let has_ball = state_position_ball.row == state_position_player.row && state_position_ball.collun == state_position_player.collun;
            if !has_ball {
                match card.card_type {
                    crate::state::state_deck::CardTypes::Set => {
                        println!("Not in position");
                        return;
                    }
                    crate::state::state_deck::CardTypes::Bump => {
                        println!("Not in position");
                        return;
                    }
                    crate::state::state_deck::CardTypes::Spike => {
                        println!("Not in position");
                        return;
                    }
                    _ => {}
                }
            }

            game_state.edit::<StateEnergy>(|x| {
                x.cur_energy = x.cur_energy - card.cost;
                println!("Energy Set to {} of {}", x.cur_energy, x.max_energy);
            });
            game_state.edit::<StateBallMode>(|x| {
                match card.card_type {
                    crate::state::state_deck::CardTypes::Set => x.mode = BallModes::Set,
                    crate::state::state_deck::CardTypes::Bump => x.mode = BallModes::Bump,
                    crate::state::state_deck::CardTypes::Spike => x.mode = BallModes::Spike,
                    _ => {}
                }
                println!("Balltype Set to {}", x.mode);
            });
            game_state.edit::<StatePositionBall>(|x| {
                match card.card_type {
                    crate::state::state_deck::CardTypes::Set => x.row += 0,
                    crate::state::state_deck::CardTypes::Bump => x.row += 1,
                    crate::state::state_deck::CardTypes::Spike => x.row += 2,
                    _ => {}
                }
                println!("Ball Move to {}, {}", x.row, x.collun);
            });

            // if not persistent card we remove from deck
            let is_persistent_card = self.card_index < (state_deck.deck.hand_persistent.len() as i32);
            if !is_persistent_card {
                game_state.edit::<StateDeck>(|x| {
                    x.deck
                        .hand_consumable
                        .remove(self.card_index as usize - state_deck.deck.hand_persistent.len());
                });
            }

            // remove the cur card
            let state_deck = game_state.get_value2::<StateDeck>();

            let bounds_min = 0;
            let bounds_max = (state_deck.deck.hand_persistent.len() + state_deck.deck.hand_consumable.len()) as i32;

            self.card_index = self.card_index.repeat(bounds_min, bounds_max);

            println!("New hand size {}", bounds_max);
        }
    }
}
