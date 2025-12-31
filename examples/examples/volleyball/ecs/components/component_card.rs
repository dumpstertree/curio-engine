use std::sync::Arc;

use crate::cards::card_instance::CardInstance;

// #[derive(Debug, Clone, Serialize, RegisterComponent)]
#[derive(Clone)]
pub struct ComponentCard {
    pub card_instance: Option<Arc<CardInstance>>,
}
impl ComponentCard {
    pub fn default() -> ComponentCard {
        ComponentCard { card_instance: None }
    }
    pub fn set_instance(mut self, card_instance: Arc<CardInstance>) -> Self {
        self.card_instance = Some(card_instance);
        self
    }
}
