use crate::{cards::enums::card_events::CardEvents, state::state_ball_mode::StateBallMode};
use curio_core::collections::ledger::Ledger;

pub struct EventReciever {}
impl EventReciever {
    pub fn recieve(event: &CardEvents, ledger: &mut Ledger) -> Vec<CardEvents> {
        match event {
            CardEvents::EventChangeBallMode(ball_mode) => {
                ledger.write::<StateBallMode>(|x| {
                    x.mode = ball_mode.clone();
                });
            }
            _ => {}
        }

        vec![]
    }
}
