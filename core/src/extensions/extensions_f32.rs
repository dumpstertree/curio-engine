// use std::hash::{Hash, Hasher};

// /// Wrapper type that enables hashing of f32 values by bit pattern.
// #[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
// pub struct HashableF32(pub f32);

// impl Eq for HashableF32 {} // safe because we treat equal bits as equal

// impl Hash for HashableF32 {
//     #[inline]
//     fn hash<H: Hasher>(&self, state: &mut H) {
//         // Use the raw bit pattern of the float.
//         // This distinguishes +0.0 vs -0.0 and different NaNs.
//         self.0.to_bits().hash(state);
//     }
// }

// impl From<f32> for HashableF32 {
//     fn from(v: f32) -> Self {
//         HashableF32(v)
//     }
// }

// impl From<HashableF32> for f32 {
//     fn from(v: HashableF32) -> Self {
//         v.0
//     }
// }

// impl HashableF32 for f32 {}

use std::hash::{Hash, Hasher};

pub trait ExtensionsF32 {
    fn hash<H: Hasher>(&self, state: &mut H);
}

impl ExtensionsF32 for f32 {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.to_bits().hash(state)
    }
}
