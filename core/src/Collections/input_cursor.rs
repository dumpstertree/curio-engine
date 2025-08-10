use crate::collections::vector3::Vector3;

#[derive(Clone)]
pub struct InputAxisState {
    pub position: Vector3,
    pub delta: Vector3,
}

impl InputAxisState {
    pub fn default() -> InputAxisState {
        InputAxisState {
            position: Vector3::zero(),
            delta: Vector3::zero(),
        }
    }

    pub fn update(&mut self, axis: Vector3) {
        self.delta = axis - self.position;
        self.position = axis;
    }
}
