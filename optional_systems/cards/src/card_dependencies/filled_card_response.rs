use serde::{Deserialize, Serialize};

use crate::cards::card_dependencies::filled_card_attribute::FilledCardAttribute;

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct FilledCardResponse {
    pub modifiers: Vec<FilledCardAttribute>,
    pub event: Vec<FilledCardAttribute>,
}
impl FilledCardResponse {
    pub fn new(state: Vec<FilledCardAttribute>, event: Vec<FilledCardAttribute>) -> FilledCardResponse {
        FilledCardResponse { modifiers: state, event }
    }
}
