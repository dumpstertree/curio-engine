use curio_core::{collections::state_ownerships::StateOwnerships, system::system_game_state::IState};
use record::record;
use std::{collections::HashMap, hash::Hash};

use gameplay::form::Form;

#[record]
pub struct StateEntityIDs {
    ids: HashMap<EntityIDTypes, Vec<Form>>,
}
impl StateEntityIDs {
    pub fn add(&mut self, id_type: EntityIDTypes, id: Form) {
        if !self.ids.contains_key(&id_type) {
            self.ids.insert(id_type.clone(), Vec::new());
        }

        if let Some(val) = self.ids.get_mut(&id_type) {
            val.push(id);
        };
    }
    pub fn get(&self, id_type: EntityIDTypes) -> Vec<Form> {
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
