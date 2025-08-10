use core::collections::vector3::Vector3;

pub struct Spin {
    pub speed: f32,
    pub axis: Vector3,
}
impl Spin {
    #[allow(unused)]
    pub fn set_speed(mut self, speed: f32) -> Spin {
        self.speed = speed;
        self
    }
    #[allow(unused)]
    pub fn set_axis(mut self, axis: Vector3) -> Spin {
        self.axis = axis;
        self
    }
}
impl Default for Spin {
    fn default() -> Self {
        Self {
            speed: 1.0,
            axis: Vector3::up(),
        }
    }
}
