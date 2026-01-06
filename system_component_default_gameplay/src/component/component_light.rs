use core::collections::{color::Color, light_uniform::LightType, vector3::Vector3};

// use macro_component::global_component;
use serde::Deserialize;

use crate::field_override::FieldDeserialize;

#[derive(Default, Deserialize)]
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
impl FieldDeserialize for ComponentLight {
    fn override_field(&mut self, field: &str, value: &str) {
        match field {
            // "type" => self.asset = value.parse().unwrap_or_default(),
            "direction" => self.direction = value.parse().unwrap_or_default(),
            "color" => self.color = value.parse().unwrap_or_default(),
            "radius" => self.radius = value.parse().unwrap_or_default(),
            "intensity" => self.intensity = value.parse().unwrap_or_default(),
            _ => {}
        }
    }
}
