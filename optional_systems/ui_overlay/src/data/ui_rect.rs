/// A resolved box in screen pixels. x/y is the top-left corner, y grows down.
#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub struct UiRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}
impl UiRect {
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> UiRect {
        UiRect { x, y, width, height }
    }
    pub const fn zero() -> UiRect {
        UiRect::new(0.0, 0.0, 0.0, 0.0)
    }
    pub fn area(&self) -> f32 {
        self.width * self.height
    }
    /// Is a screen-space point inside this box? Used for hover/click routing.
    pub fn contains(&self, x: f32, y: f32) -> bool {
        x >= self.x && x <= self.x + self.width && y >= self.y && y <= self.y + self.height
    }
}
