use core::{collections::state_ownerships::StateOwnerships, system::system_game_state::IState};
use std::{collections::HashMap, hash::Hash};

use hecs::Entity;
use macro_state::global_state;
use system_component_default_gameplay::world_context::GameObject;

#[global_state]
pub struct StateEntityIDs {
    ids: HashMap<EntityIDTypes, Vec<GameObject>>,
}
impl StateEntityIDs {
    pub fn add(&mut self, id_type: EntityIDTypes, id: GameObject) {
        if !self.ids.contains_key(&id_type) {
            self.ids.insert(id_type.clone(), Vec::new());
        }

        if let Some(val) = self.ids.get_mut(&id_type) {
            val.push(id);
        };
    }
    pub fn get(&self, id_type: EntityIDTypes) -> Vec<GameObject> {
        if let Some(val) = self.ids.get(&id_type) {
            return val.clone();
        };

        vec![]
    }
    pub fn clear(&mut self, id_type: EntityIDTypes) {
        if let Some(val) = self.ids.get_mut(&id_type) {
            val.clear();
        };
    }
}
impl IState for StateEntityIDs {
    fn id() -> i32 {
        3873473
    }
    fn ownership() -> StateOwnerships {
        StateOwnerships::Instance
    }
}
impl Hash for StateEntityIDs {
    fn hash<H: std::hash::Hasher>(&self, _: &mut H) {}
}

#[derive(Hash, Default, Clone, PartialEq, Eq)]
pub enum EntityIDTypes {
    #[default]
    Background,
    Entities,
    Ball,
    UICards,
    UIBallMode,
    UIScore,
    UIEnergy,
    UITurn,

    UIPanelHealing,
}
