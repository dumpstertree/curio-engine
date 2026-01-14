use std::hash::{Hash, Hasher};

pub trait ExtensionsF64 {
    fn hash<H: Hasher>(&self, state: &mut H);
}

impl ExtensionsF64 for f64 {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.to_bits().hash(state)
    }
}
