use std::f32;

use crate::Collections::vector3::Vector3;

pub struct ComponentBall {
    pub direction: Vector3,
    pub speed: f32,
}
impl ComponentBall {
    pub fn default() -> ComponentBall {
        ComponentBall {
            speed: 1.0,
            direction: Vector3::up(),
        }
    }
    pub fn set_speed(mut self, speed: f32) -> ComponentBall {
        self.speed = speed;
        self
    }
    pub fn set_axis(mut self, direction: Vector3) -> ComponentBall {
        self.direction = direction;
        self
    }
}
