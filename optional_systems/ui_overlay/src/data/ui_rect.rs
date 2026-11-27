use curio_core::Vector2;

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
    pub fn center(&self) -> Vector2 {
        Vector2::new(self.x + self.width * 0.5, self.y + self.height * 0.5)
    }
    /// Is a screen-space point inside this box? Used for hover/click routing.
    pub fn contains(&self, x: f32, y: f32) -> bool {
        x >= self.x && x <= self.x + self.width && y >= self.y && y <= self.y + self.height
    }
    /// Where a point of this upright box lands once the box is scaled, then turned
    /// `rotation` radians counter-clockwise on screen, both about its centre.
    pub fn turn_point(&self, x: f32, y: f32, rotation: f32, scale: Vector2) -> Vector2 {
        let center = self.center();
        let dx = (x - center.x) * scale.x;
        let dy = (y - center.y) * scale.y;
        let (sin, cos) = rotation.sin_cos();
        // y grows down, so counter-clockwise on screen is a negative turn in these axes
        Vector2::new(center.x + dx * cos + dy * sin, center.y - dx * sin + dy * cos)
    }
    /// The four corners of the turned box: top-left, top-right, bottom-right, bottom-left.
    pub fn turned_corners(&self, rotation: f32, scale: Vector2) -> [Vector2; 4] {
        let right = self.x + self.width;
        let bottom = self.y + self.height;
        [
            self.turn_point(self.x, self.y, rotation, scale),
            self.turn_point(right, self.y, rotation, scale),
            self.turn_point(right, bottom, rotation, scale),
            self.turn_point(self.x, bottom, rotation, scale),
        ]
    }
    /// Is a screen-space point inside the box as `turn_point` draws it?
    pub fn contains_turned(&self, x: f32, y: f32, rotation: f32, scale: Vector2) -> bool {
        if scale.x == 0.0 || scale.y == 0.0 {
            return false;
        }
        let center = self.center();
        let dx = x - center.x;
        let dy = y - center.y;
        let (sin, cos) = rotation.sin_cos();
        let local_x = (dx * cos - dy * sin) / scale.x;
        let local_y = (dx * sin + dy * cos) / scale.y;
        self.contains(center.x + local_x, center.y + local_y)
    }
}
