use crate::{ cards::card_dependencies::data_dep_filled::DataDepsFilled};

use std::vec;
pub struct DataDepsFilledAllPermutations {
    pub permutations: Vec<DataDepsFilled>,
}
impl DataDepsFilledAllPermutations {
    pub fn new() -> DataDepsFilledAllPermutations {
        DataDepsFilledAllPermutations { permutations: vec![] }
    }
    pub fn add_permutation(&mut self, permutation: DataDepsFilled) {
        self.permutations.push(permutation);
    }
}
