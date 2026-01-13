use std::sync::Arc;

use macro_component::global_component;
use system_component_default_gameplay::traits::field_override::FieldOverride;

use crate::cards::card_instance::CardInstance;

#[global_component]
pub struct ComponentCard {
    pub card_instance: Option<Arc<CardInstance>>,
}
impl ComponentCard {
    pub fn set_instance(mut self, card_instance: Arc<CardInstance>) -> Self {
        self.card_instance = Some(card_instance);
        self
    }
}
impl FieldOverride for ComponentCard {
    fn apply(&mut self, _field: &str, _val: &str) {}
}
