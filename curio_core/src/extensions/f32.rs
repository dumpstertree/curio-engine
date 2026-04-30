pub trait Map {
    fn map(s: f32, a1: f32, a2: f32, b1: f32, b2: f32) -> f32;
}
pub trait RoundToInt {
    fn round_to_int(x: f32) -> i32;
}
impl Map for f32 {
    fn map(s: f32, a1: f32, a2: f32, b1: f32, b2: f32) -> f32 {
        return b1 + (s - a1) * (b2 - b1) / (a2 - a1);
    }
}
impl RoundToInt for f32 {
    fn round_to_int(x: f32) -> i32 {
        x.round() as i32
    }
}

pub struct F32Extensions {}
impl F32Extensions {
    pub fn map(s: f32, a1: f32, a2: f32, b1: f32, b2: f32) -> f32 {
        return b1 + (s - a1) * (b2 - b1) / (a2 - a1);
    }
}

pub trait FloatExtras {
    fn repeat(&self, modulus: f32) -> f32;
}

impl FloatExtras for f32 {
    fn repeat(&self, modulus: f32) -> f32 {
        if modulus == 0.0 {
            return *self; // avoid division by zero
        }
        self - (*self / modulus).floor() * modulus
    }
}
