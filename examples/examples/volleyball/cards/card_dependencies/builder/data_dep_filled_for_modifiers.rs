use crate::{
    cards::card_dependencies::builder::filled_attribute_with_permutation::FilledAttributeWithPermutation,
    game_events::{FilledAttribute, FilledCardResponse},
};

use std::vec;

pub struct DataDepsFilledForModifiers {
    modifiers_atts: Vec<FilledAttributeWithPermutation>,
    modifiers_events: Vec<FilledAttributeWithPermutation>,
}
impl DataDepsFilledForModifiers {
    pub fn new() -> DataDepsFilledForModifiers {
        DataDepsFilledForModifiers { modifiers_atts: vec![], modifiers_events: vec![] }
    }

    pub fn add_modifier_atts(&mut self, permutation: FilledAttributeWithPermutation) {
        self.modifiers_atts.push(permutation);
    }

    pub fn add_modifier_event(&mut self, permutation: FilledAttributeWithPermutation) {
        self.modifiers_events.push(permutation);
    }
}
impl DataDepsFilledForModifiers {
    pub fn get_data_stack_permutations(&self) -> Vec<FilledCardResponse> {
        // this is incomplete
        let mut output_mods = Vec::new();
        for x in &self.modifiers_atts {
            let mut filled_att = Vec::new();
            for att in &x.filled {
                filled_att.push(att.permutations[0].clone());
            }
            output_mods.push(FilledAttribute::new(filled_att));
        }
        let mut output_events = Vec::new();
        for x in &self.modifiers_events {
            let mut filled_att = Vec::new();
            for att in &x.filled {
                filled_att.push(att.permutations[0].clone());
            }
            output_events.push(FilledAttribute::new(filled_att));
        }

        vec![FilledCardResponse::new(output_mods, output_events)]
    }
}
