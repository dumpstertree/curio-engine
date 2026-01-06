use core::{
    collections::{event_queue::EventQueue, game_state::GameState},
    dumpster_engine::NetworkModes,
};
use ecs_event::global_ecs_system_event_reciever;
use serde::{Deserialize, Serialize};
use system_component_default_gameplay::{
    traits::{impulse::Impulse, scope::Scope},
    world_context::WorldContext,
};

use crate::game_events::GameEvents;

#[derive(Default)]
#[global_ecs_system_event_reciever(GameEvents)]
pub struct Listener {
    uimode: UITypes,
}

// Impl - Instance
impl Scope for Listener {
    fn is_enabled(&mut self, _game_state: &mut GameState) -> bool {
        true
    }
    fn run_on_instance(&mut self, _game_state: &mut GameState) -> Vec<NetworkModes> {
        NetworkModes::all()
    }
}
// Impl - Listener
impl Impulse<GameEvents> for Listener {
    fn dequeue_event(&mut self, _game_state: &mut GameState, _: &mut WorldContext, event_queue: &mut EventQueue, event: &GameEvents) {
        match event {
            GameEvents::SetUIMode(ui) => {
                println!("set ui");
                match self.uimode {
                    UITypes::Encounter => event_queue.enqueue_event(GameEvents::DisableUICombat),
                    UITypes::Heal => event_queue.enqueue_event(GameEvents::DisableUIHealing),
                    UITypes::Shop => event_queue.enqueue_event(GameEvents::DisableUIShop),
                    _ => {}
                }

                self.uimode = ui.clone();

                match self.uimode {
                    UITypes::Encounter => event_queue.enqueue_event(GameEvents::EnableUICombat),
                    UITypes::Heal => event_queue.enqueue_event(GameEvents::EnableUIHealing),
                    UITypes::Shop => event_queue.enqueue_event(GameEvents::EnableUIShop),
                    _ => {}
                }
            }
            _ => {}
        }
    }
}

#[derive(PartialEq, Eq, Hash, Default, Clone, Deserialize, Serialize)]
pub enum UITypes {
    #[default]
    None,
    Shop,
    Heal,
    Encounter,
}
