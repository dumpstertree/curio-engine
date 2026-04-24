use curio_core::{
    collections::network_modes::NetworkModes,
    collections::{event_queue::EventQueue, ledger::Ledger},
};
use gameplay::{
    context_3d::Context3D,
    traits::{impulse::Impulse, scope::Scope},
};
use impulse::impulse;
use serde::{Deserialize, Serialize};

use crate::game_events::GameEvents;

#[derive(Default)]
#[impulse(GameEvents)]
pub struct Listener {
    uimode: UITypes,
}

// Impl - Instance
impl Scope for Listener {
    fn is_enabled(&mut self, _ledger: &mut Ledger) -> bool {
        true
    }
    fn run_on_instance(&mut self, _ledger: &mut Ledger) -> Vec<NetworkModes> {
        NetworkModes::all()
    }
}
// Impl - Listener
impl Impulse<GameEvents> for Listener {
    fn dequeue_event(&mut self, _ledger: &mut Ledger, _: &mut Context3D, event_queue: &mut EventQueue, event: &GameEvents) {
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
