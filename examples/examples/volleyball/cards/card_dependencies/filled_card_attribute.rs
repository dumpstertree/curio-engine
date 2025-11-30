use serde::{Deserialize, Serialize};

use crate::cards::card_dependencies::data_dep_filled::DataDepsFilled;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct FilledCardAttribute {
    pub filled: Vec<DataDepsFilled>,
}
impl FilledCardAttribute {
    pub fn new(filled: Vec<DataDepsFilled>) -> FilledCardAttribute {
        FilledCardAttribute { filled }
    }
}
