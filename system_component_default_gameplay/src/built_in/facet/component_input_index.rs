#[derive(Clone)]
pub struct InputIndex {
    pub index: usize,
}

impl InputIndex {
    pub fn set_index(mut self, index: usize) -> InputIndex {
        self.index = index;
        self
    }
    pub fn default() -> InputIndex {
        InputIndex { index: 0 }
    }
}
