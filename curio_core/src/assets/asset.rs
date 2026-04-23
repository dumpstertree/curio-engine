use std::any::Any;

// asset
pub trait AssetCommon: Any {}
pub trait AssetCommonFromBits<T>: AssetCommon {
    fn from_bits(bits: &Vec<u8>) -> T;
}
