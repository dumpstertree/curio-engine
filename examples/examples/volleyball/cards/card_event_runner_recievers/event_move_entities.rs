use crate::{cards::enums::card_events::CardEvents, state::state_position_player::StatePositionEntities};
use core::collections::game_state::GameState;

pub struct EventReciever {}
impl EventReciever {
    pub fn recieve(event: &CardEvents, game_state: &mut GameState) -> Vec<CardEvents> {
        match event {
            CardEvents::EventMoveEntities(entities, tiles) => {
                // unwrap
                let tile_ids = tiles.as_tiles();
                let entity_ids = entities.as_entities();

                if tile_ids.len() != 1 {
                    panic!("only supports one tile");
                }

                game_state.edit::<StatePositionEntities>(|y| {
                    for x in &entity_ids {
                        let Some(position) = y.positions.get_mut(x) else { return };
                        position.0 = tile_ids[0].x;
                        position.1 = tile_ids[0].y;
                    }
                });
            }
            _ => {}
        }
        vec![]
    }
}
