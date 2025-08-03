pub trait Map {
    fn map(s: f32, a1: f32, a2: f32, b1: f32, b2: f32) -> f32;
}
impl Map for f32 {
    fn map(s: f32, a1: f32, a2: f32, b1: f32, b2: f32) -> f32 {
        return b1 + (s - a1) * (b2 - b1) / (a2 - a1);
    }
}

pub struct f32Extensions {}
impl f32Extensions {
    pub fn map(s: f32, a1: f32, a2: f32, b1: f32, b2: f32) -> f32 {
        return b1 + (s - a1) * (b2 - b1) / (a2 - a1);
    }
}
