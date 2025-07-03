pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}
impl Color {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Color {
        Color { r: r, g: g, b: b, a: a }
    }
    fn get_clear() -> Color {
        Color { r: 0.0, g: 0.0, b: 0.0, a: 0.0 }
    }
    fn get_black() -> Color {
        Color { r: 0.0, g: 0.0, b: 0.0, a: 0.0 }
    }
    fn get_white() -> Color {
        Color { r: 0.0, g: 0.0, b: 0.0, a: 0.0 }
    }
    fn get_green() -> Color {
        Color { r: 0.0, g: 0.0, b: 0.0, a: 0.0 }
    }
    fn get_blue() -> Color {
        Color { r: 0.0, g: 0.0, b: 0.0, a: 0.0 }
    }
    fn get_red() -> Color {
        Color { r: 0.0, g: 0.0, b: 0.0, a: 0.0 }
    }
}
