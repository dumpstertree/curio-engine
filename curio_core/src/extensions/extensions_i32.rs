pub trait ExtensionsI32 {
    fn repeat(&self, min: i32, max: i32) -> i32;
}

impl ExtensionsI32 for i32 {
    /// impl for  Hash for f32
    fn repeat(&self, min: i32, max: i32) -> i32 {
        let range = max - min;
        if range == 0 {
            return min; // avoid division by zero
        }
        ((*self - min).rem_euclid(range)) + min
    }
}
