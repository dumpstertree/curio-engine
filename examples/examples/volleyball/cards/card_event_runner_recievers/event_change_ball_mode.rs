use crate::{cards::enums::card_events::CardEvents, state::state_ball_mode::StateBallMode};
use core::collections::game_state::GameState;

pub struct EventReciever {}
impl EventReciever {
    pub fn recieve(event: &CardEvents, game_state: &mut GameState) -> Vec<CardEvents> {
        match event {
            CardEvents::EventChangeBallMode(ball_mode) => {
                game_state.edit::<StateBallMode>(|x| {
                    x.mode = ball_mode.clone();
                });
            }
            _ => {}
        }

        vec![]
    }
}
