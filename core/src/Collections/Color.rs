#[derive(Clone)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}
impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "r:{}, g: {}, b:{}, a:{}", self.r, self.g, self.b, self.a)
    }
}
impl Color {
    pub fn r_0255(&self) -> u8 {
        (self.r * 255.0).round() as u8
    }
    pub fn g_0255(&self) -> u8 {
        (self.g * 255.0).round() as u8
    }
    pub fn b_0255(&self) -> u8 {
        (self.b * 255.0).round() as u8
    }
    pub fn a_0255(&self) -> u8 {
        (self.a * 255.0).round() as u8
    }

    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Color {
        Color { r: r, g: g, b: b, a: a }
    }
    pub fn get_clear() -> Color {
        Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.0,
        }
    }
    pub fn get_black() -> Color {
        Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }
    }
    pub fn get_white() -> Color {
        Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        }
    }
    pub fn get_green() -> Color {
        Color {
            r: 0.0,
            g: 1.0,
            b: 0.0,
            a: 1.0,
        }
    }
    pub fn get_blue() -> Color {
        Color {
            r: 0.0,
            g: 0.0,
            b: 1.0,
            a: 1.0,
        }
    }
    pub fn get_red() -> Color {
        Color {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }
    }
}
