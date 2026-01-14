use crate::{cards::enums::card_events::CardEvents, state::host::state_heat::StateHeat};
use curio_core::collections::game_state::GameState;

pub struct EventReciever {}
impl EventReciever {
    pub fn recieve(event: &CardEvents, game_state: &mut GameState) -> Vec<CardEvents> {
        match event {
            CardEvents::EventHeatDrain(wrapped_entities) => {
                // edit the energy
                game_state.edit::<StateHeat>(|x| {
                    // unwrap entityes
                    let entity_ids = wrapped_entities.as_entities();
                    // iterate over each entity
                    for entity_id in &entity_ids {
                        // set the heat to zero
                        x.all_players.insert(*entity_id, 0);
                    }
                });
            }
            _ => {}
        }
        vec![]
    }
}
