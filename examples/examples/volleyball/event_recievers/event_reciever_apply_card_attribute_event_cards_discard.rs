use crate::{ai_resolver::CardEvents, game_board, game_events::GameEvents, state::state_deck::StateDeck};
use core::{
    collections::{
        event_queue::EventQueue,
        game_state::{self, GameState},
    },
    dumpster_engine::NetworkModes,
    gameplay::ecs::traits::{ecs_event_reciever, ecs_system::ECSSystemEventless},
};
use ecs_event::global_ecs_system_event_reciever;
use ecs_system::global_ecs_system;
use hecs::World;

#[global_ecs_system]
pub struct EventReciever {}
impl ECSSystemEventless for EventReciever {
    fn is_enabled(&mut self, _: &mut GameState, _: &mut World) -> bool {
        true
    }
    fn run_on_instance(&mut self, game_state: &mut GameState) -> Vec<core::dumpster_engine::NetworkModes> {
        vec![NetworkModes::LocalHost, NetworkModes::OnlineHost]
    }
}
#[global_ecs_system_event_reciever(GameEvents)]
impl ecs_event_reciever::EventReciever<GameEvents> for EventReciever {
    fn dequeue_event(&mut self, game_state: &mut GameState, _: &mut World, _: &mut EventQueue, event: &GameEvents) {
        match event {
            GameEvents::ApplyCardAttributeEventDiscardCards(card_ids) => {
                game_state.edit::<StateDeck>(|x| {
                    for y in x.deck.iter_mut() {
                        for i in (0..y.1.hand_consumable.len()).rev() {
                            let remove = card_ids.contains(&y.1.hand_consumable[i].instance_id);
                            if remove {
                                y.1.pile_discard.push(y.1.hand_consumable[i].clone());
                                y.1.hand_consumable.remove(i);
                            }
                        }
                    }
                });
            }
            _ => {}
        }
    }
}
impl EventReciever {
    pub fn recieve(event: CardEvents, game_state: GameState) -> Vec<CardEvents> {
        vec![]
    }
}
