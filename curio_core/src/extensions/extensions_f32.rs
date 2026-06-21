use std::hash::{Hash, Hasher};

pub trait ExtensionsF32 {
    fn hash<H: Hasher>(&self, state: &mut H);
    fn map(&self, s: f32, a1: f32, a2: f32, b1: f32, b2: f32) -> f32;
    fn round(&self, x: f32) -> i32;
    fn repeat(&self, modulus: f32) -> f32;
}

impl ExtensionsF32 for f32 {
    /// impl for  Hash for f32
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.to_bits().hash(state)
    }

    /// remap the value from rang a to range b
    fn map(&self, s: f32, a1: f32, a2: f32, b1: f32, b2: f32) -> f32 {
        return b1 + (s - a1) * (b2 - b1) / (a2 - a1);
    }

    /// round to the nearest i32
    fn round(&self, x: f32) -> i32 {
        x.round() as i32
    }

    /// wrap the value around the min-max
    fn repeat(&self, max: f32) -> f32 {
        // avoid division by 0
        if max == 0.0 {
            return *self;
        }

        // return value
        self - (*self / max).floor() * max
    }
}
