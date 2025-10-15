use core::collections::{
        color::Color,
        light_uniform::LightType,
        vector3::Vector3,
    };

pub struct ComponentLight {
    pub asset: LightType,
    pub direction: Vector3,
    pub color: Color,
    pub radius: f32,
    pub intensity: f32,
}

impl ComponentLight {
    pub fn default() -> ComponentLight {
        ComponentLight {
            asset: LightType::Point,
            direction: Vector3::zero(),
            color: Color::white(),
            radius: 10.0,
            intensity: 1.0,
        }
    }
}
