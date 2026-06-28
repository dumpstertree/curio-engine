use std::any::Any;

/// Base for an Asset that can be loaded from the AssetLoader
pub trait AssetCommon<T>: Any {
    /// Convert raw bits into an instance of this asset
    fn from_bits(bits: &Vec<u8>) -> T;
}
