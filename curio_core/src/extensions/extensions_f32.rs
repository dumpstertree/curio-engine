use std::hash::{Hash, Hasher};

pub trait ExtensionsF32 {
    fn hash<H: Hasher>(&self, state: &mut H);
    fn map(s: f32, a1: f32, a2: f32, b1: f32, b2: f32) -> f32;
    fn round_to_int(x: f32) -> i32;
    fn repeat(&self, modulus: f32) -> f32;
}

impl ExtensionsF32 for f32 {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.to_bits().hash(state)
    }
    fn map(s: f32, a1: f32, a2: f32, b1: f32, b2: f32) -> f32 {
        return b1 + (s - a1) * (b2 - b1) / (a2 - a1);
    }
    fn round_to_int(x: f32) -> i32 {
        x.round() as i32
    }
    fn repeat(&self, modulus: f32) -> f32 {
        if modulus == 0.0 {
            return *self; // avoid division by zero
        }
        self - (*self / modulus).floor() * modulus
    }
}
