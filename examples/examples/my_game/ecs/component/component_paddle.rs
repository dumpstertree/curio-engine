use core::collections::vector3::Vector3;

#[derive(Debug, Clone)]
pub struct ComponentPaddle {
    pub speed: f32,
    pub axis: Vector3,
}
impl ComponentPaddle {
    pub fn default() -> ComponentPaddle {
        ComponentPaddle {
            speed: 1.0,
            axis: Vector3::up(),
        }
    }
    pub fn set_speed(mut self, speed: f32) -> ComponentPaddle {
        self.speed = speed;
        self
    }
    pub fn set_axis(mut self, axis: Vector3) -> ComponentPaddle {
        self.axis = axis;
        self
    }
}
