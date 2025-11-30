use crate::{ai_resolver::CardEvents, state::state_position_ball::StatePositionBall};
use core::collections::game_state::GameState;

pub struct EventReciever {}
impl EventReciever {
    pub fn recieve(event: &CardEvents, game_state: &mut GameState) -> Vec<CardEvents> {
        match event {
            CardEvents::ApplyEventMoveBall(wrapped_tiles) => {
                //
                let tile_ids = wrapped_tiles.as_tiles();
                if tile_ids.len() == 0 {
                    println!("Does not support len of 0 tile");
                    return vec![];
                }
                //
                if tile_ids.len() > 1 {
                    println!("Does not support more than 1 tile");
                    return vec![];
                }

                // note: all distance modifiers shoule be applied at the targeting phase

                // edit ball position
                game_state.edit::<StatePositionBall>(|x| {
                    x.column = tile_ids[0].x;
                    x.row = tile_ids[0].y;
                })
            }
            _ => {}
        }
        vec![]
    }
}
