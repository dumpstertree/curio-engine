// use register_macro::RegisterComponent;
// use serde::Serialize;

// use std::f32;
// #[derive(Debug, Clone, Serialize, RegisterComponent)]
// struct Position(f32, f32);

use core::Collections::vector3::Vector3;

// #[derive(Debug, Clone, Serialize, RegisterComponent)]
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
