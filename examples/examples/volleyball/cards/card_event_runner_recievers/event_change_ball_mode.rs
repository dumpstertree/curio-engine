use crate::{cards::enums::card_events::CardEvents, state::state_ball_mode::StateBallMode};
use curio_core::collections::game_state::Ledger;

pub struct EventReciever {}
impl EventReciever {
    pub fn recieve(event: &CardEvents, game_state: &mut Ledger) -> Vec<CardEvents> {
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
