use crate::{cards::enums::card_events::CardEvents, state::state_position_ball::StatePositionBall};
use curio_core::collections::game_state::GameState;

pub struct EventReciever {}
impl EventReciever {
    pub fn recieve(event: &CardEvents, game_state: &mut GameState) -> Vec<CardEvents> {
        match event {
            CardEvents::EventMoveBall(wrapped_tiles) => {
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
                    let pos_x = tile_ids[0].x.clamp(0, 3);
                    let pos_y = tile_ids[0].y.clamp(0, 3);
                    // x.column = tile_ids[0].x;
                    // x.row = tile_ids[0].y;
                    x.column = pos_x;
                    x.row = pos_y;
                })
            }
            _ => {}
        }
        vec![]
    }
}
