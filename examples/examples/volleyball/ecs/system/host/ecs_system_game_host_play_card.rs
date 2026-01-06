use crate::{
    cards::{card_event_runner::CardEventRunner, enums::attribute_clear_flag::ModifierClearFlag},
    game_events::GameEvents,
    state::host::state_play_history::StatePlayHistory,
};
use core::{
    collections::{event_queue::EventQueue, game_state::GameState},
    dumpster_engine::NetworkModes,
};
use ecs_event::global_ecs_system_event_reciever;
use ecs_system::global_ecs_system;
use std::vec;
use system_component_default_gameplay::{
    traits::{ecs_system::ECSSystemEventless, event_reciever::EventReciever, instance_scope::InstanceLimiter},
    world_context::WorldContext,
};

#[global_ecs_system]
#[global_ecs_system_event_reciever(GameEvents)]
pub struct ECSSystemGameRequestManuever {}

impl ECSSystemEventless for ECSSystemGameRequestManuever {
    fn is_enabled(&mut self, _: &mut GameState, _: &mut WorldContext) -> bool {
        true
    }
    fn run_on_instance(&mut self, _: &mut GameState) -> Vec<NetworkModes> {
        vec![NetworkModes::LocalHost, NetworkModes::OnlineHost]
    }
}
impl InstanceLimiter for ECSSystemGameRequestManuever {
    fn is_enabled(&mut self, _game_state: &mut GameState) -> bool {
        true
    }
    fn run_on_instance(&mut self, _game_state: &mut GameState) -> Vec<NetworkModes> {
        NetworkModes::all_host()
    }
}
impl EventReciever<GameEvents> for ECSSystemGameRequestManuever {
    fn dequeue_event(&mut self, game_state: &mut GameState, _: &mut WorldContext, event_queue: &mut EventQueue, event: &GameEvents) {
        match event {
            GameEvents::PlayCard(id, card_instance, data) => {
                // creates an event runner to all the events on
                let mut event_runner = CardEventRunner::new();

                // get the attributes out of this card
                let atts_mods = card_instance.get_attributes_modifiers(&game_state, *id);
                let atts_evnt = card_instance.get_attributes_events(&game_state, *id);

                println!("PLAYED CARD: {}", card_instance.card_id);
                // iterate over each mod and add it and its data to the runner
                for i in 0..atts_mods.len() {
                    event_runner.enqueue_modifier(&atts_mods[i], &data.modifiers[i]);
                }

                // iterate over each event and add it and its data to the runner
                for i in 0..atts_evnt.len() {
                    event_runner.enqueue_event(&atts_evnt[i], &data.event[i]);
                }

                // enqueue the clear flag
                event_runner.enqueue_clear_modifiers(&ModifierClearFlag::Play);

                // run all inside runner
                event_runner.post_and_drain(game_state);

                // add play to history
                game_state.edit::<StatePlayHistory>(|x| {
                    x.history.push((*id, card_instance.clone(), data.clone()));
                });

                // send event that we did play the card
                event_queue.enqueue_event(GameEvents::DidPlayCard(*id, card_instance.clone(), data.clone()));
            }

            _ => {}
        }
    }
}
