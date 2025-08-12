use crate::collections::vector2::Vector2;

#[derive(Clone)]
pub struct InputAxisState {
    pub position: Vector2,
    pub delta: Vector2,
}

impl InputAxisState {
    pub fn default() -> InputAxisState {
        InputAxisState {
            position: Vector2::zero(),
            delta: Vector2::zero(),
        }
    }

    pub fn update(&mut self, axis: Vector2) {
        self.delta = axis - self.position;
        self.position = axis;
    }
}
