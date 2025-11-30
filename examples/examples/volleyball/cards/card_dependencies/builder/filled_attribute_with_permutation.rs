use crate::cards::card_dependencies::builder::data_dep_filled_all_permutations::DataDepsFilledAllPermutations;

pub struct FilledAttributeWithPermutation {
    pub filled: Vec<DataDepsFilledAllPermutations>,
}
impl FilledAttributeWithPermutation {
    pub fn new(filled: Vec<DataDepsFilledAllPermutations>) -> FilledAttributeWithPermutation {
        FilledAttributeWithPermutation { filled }
    }
}
